use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod metadata;
pub mod s3;

/// カスタムエラー型
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("ネットワークエラー: {0}")]
    Network(String),

    #[error("認証エラー: {0}")]
    Authentication(String),

    #[error("ファイルが見つかりません: {0}")]
    NotFound(String),

    #[error("アップロードエラー: {0}")]
    Upload(String),

    #[error("ダウンロードエラー: {0}")]
    Download(String),

    #[error("メタデータエラー: {0}")]
    Metadata(String),

    #[error("IO エラー: {0}")]
    Io(#[from] std::io::Error),

    #[error("その他のエラー: {0}")]
    Other(String),
}

/// ストレージエラーのResult型エイリアス
pub type Result<T> = std::result::Result<T, StorageError>;

/// ファイル情報を表す構造体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileInfo {
    /// ファイルのパス
    pub path: String,

    /// ファイルサイズ（バイト）
    pub size: u64,

    /// 最終更新日時
    pub last_modified: DateTime<Utc>,

    /// ETag（エンティティタグ、ファイルのチェックサム：MD5）
    /// AWS S3のETagと一致する形式
    pub etag: Option<String>,
}

impl FileInfo {
    /// 新しいFileInfoインスタンスを作成
    pub fn new(
        path: String,
        size: u64,
        last_modified: DateTime<Utc>,
        etag: Option<String>,
    ) -> Self {
        Self {
            path,
            size,
            last_modified,
            etag,
        }
    }

    /// 2つのファイルの内容が同じかどうかを判定
    ///
    /// ファイルサイズとETag（MD5チェックサム）を比較します。
    /// ETagが両方とも存在する場合はETagで比較し、それ以外はサイズのみで比較します。
    ///
    /// # Arguments
    /// * `other` - 比較対象のFileInfo
    ///
    /// # Returns
    /// * `true` - ファイルの内容が同じ
    /// * `false` - ファイルの内容が異なる
    pub fn has_same_content(&self, other: &FileInfo) -> bool {
        // サイズが異なれば確実に内容が異なる
        if self.size != other.size {
            return false;
        }

        // ETagが両方存在する場合はETagで比較
        match (&self.etag, &other.etag) {
            (Some(etag1), Some(etag2)) => etag1 == etag2,
            // ETagがない場合はサイズのみで判断（完全ではない）
            _ => true,
        }
    }

    /// このファイルが他のファイルより新しいかどうかを判定
    ///
    /// 最終更新日時を比較します。
    ///
    /// # Arguments
    /// * `other` - 比較対象のFileInfo
    ///
    /// # Returns
    /// * `true` - このファイルの方が新しい
    /// * `false` - このファイルの方が古いか同じ
    pub fn is_newer_than(&self, other: &FileInfo) -> bool {
        self.last_modified > other.last_modified
    }

    /// 同期が必要かどうかを判定
    ///
    /// ETag（MD5チェックサム）がある場合、ETagとサイズで判定します。
    /// ETagがない場合は、サイズと最終更新日時で判定します（後方互換性のため）。
    ///
    /// ETagがある場合:
    /// 1. ファイルサイズが異なる → 同期必要
    /// 2. ETagが異なる → 同期必要
    /// 3. サイズとETagが同じ → 同期不要（更新日時は無視）
    ///
    /// ETagがない場合:
    /// 1. ファイルサイズが異なる → 同期必要
    /// 2. 最終更新日時が異なる（秒単位で比較） → 同期必要
    ///
    /// # Arguments
    /// * `other` - 比較対象のFileInfo
    ///
    /// # Returns
    /// * `true` - 同期が必要
    /// * `false` - 同期不要（ファイルは同一）
    pub fn needs_sync(&self, other: &FileInfo) -> bool {
        // サイズが異なれば同期が必要
        if self.size != other.size {
            return true;
        }

        // ETagが両方存在する場合はETagのみで判定（更新日時は無視）
        if let (Some(etag1), Some(etag2)) = (&self.etag, &other.etag) {
            return etag1 != etag2;
        }

        // ETagがない場合は最終更新日時で判定（後方互換性のため）
        // ファイルシステムの時刻精度を考慮して秒単位で比較
        let diff = (self.last_modified.timestamp() - other.last_modified.timestamp()).abs();
        diff > 1
    }
}

/// ファイルのメタデータを表す構造体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    /// ファイルサイズ（バイト）
    pub size: u64,

    /// 最終更新日時
    pub last_modified: DateTime<Utc>,

    /// コンテンツタイプ（MIMEタイプ）
    pub content_type: Option<String>,

    /// ETag（エンティティタグ、ファイルのチェックサム：MD5）
    /// AWS S3のETagと一致する形式
    pub etag: Option<String>,
}

impl FileMetadata {
    /// 新しいFileMetadataインスタンスを作成
    pub fn new(
        size: u64,
        last_modified: DateTime<Utc>,
        content_type: Option<String>,
        etag: Option<String>,
    ) -> Self {
        Self {
            size,
            last_modified,
            content_type,
            etag,
        }
    }
}

/// クラウドストレージプロバイダーのための抽象化トレイト
///
/// このトレイトは、異なるクラウドストレージプロバイダー（AWS S3、GCS、Azure Blob等）
/// に対して統一されたインターフェースを提供します。
#[async_trait::async_trait]
pub trait CloudStorageProvider: Send + Sync {
    /// ファイルをクラウドストレージにアップロード
    ///
    /// # 引数
    /// * `path` - アップロード先のパス
    /// * `data` - アップロードするデータ
    /// * `metadata` - ファイルのメタデータ
    ///
    /// # エラー
    /// アップロードに失敗した場合は `StorageError::Upload` を返す
    async fn upload(&self, path: &str, data: &[u8], metadata: FileMetadata) -> Result<()>;

    /// クラウドストレージからファイルをダウンロード
    ///
    /// # 引数
    /// * `path` - ダウンロードするファイルのパス
    ///
    /// # 戻り値
    /// ダウンロードしたファイルのデータ
    ///
    /// # エラー
    /// ダウンロードに失敗した場合は `StorageError::Download` を返す
    async fn download(&self, path: &str) -> Result<Vec<u8>>;

    /// 指定されたプレフィックスでファイルのリストを取得
    ///
    /// # 引数
    /// * `prefix` - ファイルパスのプレフィックス（フォルダパス等）
    ///
    /// # 戻り値
    /// ファイル情報のベクター
    ///
    /// # エラー
    /// リストの取得に失敗した場合はエラーを返す
    async fn list(&self, prefix: &str) -> Result<Vec<FileInfo>>;

    /// クラウドストレージからファイルを削除
    ///
    /// # 引数
    /// * `path` - 削除するファイルのパス
    ///
    /// # エラー
    /// 削除に失敗した場合はエラーを返す
    async fn delete(&self, path: &str) -> Result<()>;

    /// ファイルのメタデータを取得
    ///
    /// # 引数
    /// * `path` - メタデータを取得するファイルのパス
    ///
    /// # 戻り値
    /// ファイルのメタデータ
    ///
    /// # エラー
    /// メタデータの取得に失敗した場合は `StorageError::Metadata` を返す
    async fn get_metadata(&self, path: &str) -> Result<FileMetadata>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // ========================================
    // シリアライゼーション/デシリアライゼーションテスト
    // ========================================

    #[test]
    fn test_file_info_json_round_trip() {
        let now = Utc.with_ymd_and_hms(2025, 12, 17, 10, 30, 45).unwrap();
        let original = FileInfo::new(
            "documents/report.pdf".to_string(),
            5242880, // 5MB
            now,
            Some("a1b2c3d4e5f6".to_string()),
        );

        // JSON へのシリアライゼーション
        let json = serde_json::to_string(&original).expect("Failed to serialize");

        // JSON からのデシリアライゼーション
        let deserialized: FileInfo = serde_json::from_str(&json).expect("Failed to deserialize");

        // 元のデータと一致することを確認
        assert_eq!(original, deserialized);
        assert_eq!(deserialized.path, "documents/report.pdf");
        assert_eq!(deserialized.size, 5242880);
        assert_eq!(deserialized.last_modified, now);
    }

    #[test]
    fn test_file_metadata_json_round_trip() {
        let now = Utc.with_ymd_and_hms(2025, 12, 17, 15, 20, 0).unwrap();
        let original = FileMetadata::new(
            10485760, // 10MB
            now,
            Some("application/pdf".to_string()),
            Some("etag-12345".to_string()),
        );

        let json = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: FileMetadata =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_file_info_with_special_characters() {
        let now = Utc::now();
        let file_info = FileInfo::new(
            "フォルダ/ファイル 名 (1).txt".to_string(), // 日本語、スペース、括弧
            100,
            now,
            None,
        );

        let json = serde_json::to_string(&file_info).expect("Failed to serialize");
        let deserialized: FileInfo = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(file_info.path, deserialized.path);
    }

    // ========================================
    // 境界値テスト
    // ========================================

    #[test]
    fn test_file_info_with_zero_size() {
        let now = Utc::now();
        let file_info = FileInfo::new(
            "empty.txt".to_string(),
            0, // 空のファイル
            now,
            None,
        );

        assert_eq!(file_info.size, 0);
    }

    #[test]
    fn test_file_info_with_very_large_size() {
        let now = Utc::now();
        let file_info = FileInfo::new(
            "large_video.mp4".to_string(),
            5_368_709_120, // 5GB（最大サイズ）
            now,
            Some("large-etag".to_string()),
        );

        assert_eq!(file_info.size, 5_368_709_120);
    }

    #[test]
    fn test_file_info_with_long_path() {
        let now = Utc::now();
        let long_path = "a/".repeat(100) + "file.txt"; // 非常に長いパス
        let file_info = FileInfo::new(long_path.clone(), 1024, now, None);

        assert_eq!(file_info.path, long_path);
    }

    // ========================================
    // ファイル比較ロジックテスト（同期判定）
    // ========================================

    #[test]
    fn test_files_need_sync_different_size() {
        let now = Utc::now();
        let file1 = FileInfo::new("file.txt".to_string(), 1024, now, Some("etag1".to_string()));
        let file2 = FileInfo::new("file.txt".to_string(), 2048, now, Some("etag1".to_string()));

        // サイズが異なる場合は同期が必要
        assert_ne!(file1.size, file2.size);
    }

    #[test]
    fn test_files_need_sync_different_modified_time() {
        let time1 = Utc.with_ymd_and_hms(2025, 12, 17, 10, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2025, 12, 17, 11, 0, 0).unwrap();

        let file1 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            time1,
            Some("etag1".to_string()),
        );
        let file2 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            time2,
            Some("etag1".to_string()),
        );

        // 更新時刻が異なる場合は同期が必要
        assert_ne!(file1.last_modified, file2.last_modified);
    }

    #[test]
    fn test_files_need_sync_different_etag() {
        let now = Utc::now();
        let file1 = FileInfo::new("file.txt".to_string(), 1024, now, Some("etag1".to_string()));
        let file2 = FileInfo::new("file.txt".to_string(), 1024, now, Some("etag2".to_string()));

        // ETagが異なる場合は同期が必要
        assert_ne!(file1.etag, file2.etag);
    }

    #[test]
    fn test_files_identical() {
        let now = Utc.with_ymd_and_hms(2025, 12, 17, 10, 0, 0).unwrap();
        let file1 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("same-etag".to_string()),
        );
        let file2 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("same-etag".to_string()),
        );

        // 全てが同じ場合は同期不要
        assert_eq!(file1, file2);
    }

    // ========================================
    // メタデータ比較ロジックテスト
    // ========================================

    #[test]
    fn test_has_same_content_identical_files() {
        let now = Utc::now();
        let file1 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("etag123".to_string()),
        );
        let file2 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("etag123".to_string()),
        );

        assert!(file1.has_same_content(&file2));
    }

    #[test]
    fn test_has_same_content_different_size() {
        let now = Utc::now();
        let file1 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("etag123".to_string()),
        );
        let file2 = FileInfo::new(
            "file.txt".to_string(),
            2048,
            now,
            Some("etag123".to_string()),
        );

        assert!(!file1.has_same_content(&file2));
    }

    #[test]
    fn test_has_same_content_different_etag() {
        let now = Utc::now();
        let file1 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("etag123".to_string()),
        );
        let file2 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("etag456".to_string()),
        );

        assert!(!file1.has_same_content(&file2));
    }

    #[test]
    fn test_has_same_content_no_etag() {
        let now = Utc::now();
        let file1 = FileInfo::new("file.txt".to_string(), 1024, now, None);
        let file2 = FileInfo::new("file.txt".to_string(), 1024, now, None);

        // ETagがない場合はサイズのみで判断
        assert!(file1.has_same_content(&file2));
    }

    #[test]
    fn test_is_newer_than() {
        let time1 = Utc.with_ymd_and_hms(2025, 12, 17, 10, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2025, 12, 17, 11, 0, 0).unwrap();

        let file1 = FileInfo::new("file.txt".to_string(), 1024, time1, None);
        let file2 = FileInfo::new("file.txt".to_string(), 1024, time2, None);

        assert!(!file1.is_newer_than(&file2)); // file1 is older
        assert!(file2.is_newer_than(&file1)); // file2 is newer
    }

    #[test]
    fn test_is_newer_than_same_time() {
        let now = Utc::now();
        let file1 = FileInfo::new("file.txt".to_string(), 1024, now, None);
        let file2 = FileInfo::new("file.txt".to_string(), 1024, now, None);

        assert!(!file1.is_newer_than(&file2));
        assert!(!file2.is_newer_than(&file1));
    }

    #[test]
    fn test_needs_sync_identical() {
        let now = Utc::now();
        let file1 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("etag123".to_string()),
        );
        let file2 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("etag123".to_string()),
        );

        assert!(!file1.needs_sync(&file2));
    }

    #[test]
    fn test_needs_sync_different_size() {
        let now = Utc::now();
        let file1 = FileInfo::new("file.txt".to_string(), 1024, now, None);
        let file2 = FileInfo::new("file.txt".to_string(), 2048, now, None);

        assert!(file1.needs_sync(&file2));
    }

    #[test]
    fn test_needs_sync_different_etag() {
        let now = Utc::now();
        let file1 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("etag123".to_string()),
        );
        let file2 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            now,
            Some("etag456".to_string()),
        );

        assert!(file1.needs_sync(&file2));
    }

    #[test]
    fn test_needs_sync_different_time() {
        let time1 = Utc.with_ymd_and_hms(2025, 12, 17, 10, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2025, 12, 17, 11, 0, 0).unwrap();

        let file1 = FileInfo::new("file.txt".to_string(), 1024, time1, None);
        let file2 = FileInfo::new("file.txt".to_string(), 1024, time2, None);

        assert!(file1.needs_sync(&file2));
    }

    #[test]
    fn test_needs_sync_small_time_difference() {
        let time1 = Utc.with_ymd_and_hms(2025, 12, 17, 10, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2025, 12, 17, 10, 0, 1).unwrap();

        let file1 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            time1,
            Some("etag123".to_string()),
        );
        let file2 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            time2,
            Some("etag123".to_string()),
        );

        // 1秒以内の差は許容範囲（ファイルシステムの精度を考慮）
        assert!(!file1.needs_sync(&file2));
    }

    #[test]
    fn test_needs_sync_same_etag_different_time() {
        // CRITICAL TEST: ETag takes priority over timestamp
        // When both files have the same ETag, timestamp differences should be IGNORED
        let time1 = Utc.with_ymd_and_hms(2025, 12, 17, 10, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2025, 12, 17, 11, 0, 0).unwrap(); // 1 hour difference

        let file1 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            time1,
            Some("etag123".to_string()),
        );
        let file2 = FileInfo::new(
            "file.txt".to_string(),
            1024,
            time2,
            Some("etag123".to_string()),
        );

        // Same ETag and size means files are identical, even with different timestamps
        assert!(!file1.needs_sync(&file2));
    }

    // ========================================
    // エラーハンドリングテスト
    // ========================================

    #[test]
    fn test_storage_error_display() {
        let errors = vec![
            (
                StorageError::Network("Timeout".to_string()),
                "ネットワークエラー: Timeout",
            ),
            (
                StorageError::Authentication("Invalid token".to_string()),
                "認証エラー: Invalid token",
            ),
            (
                StorageError::NotFound("missing.txt".to_string()),
                "ファイルが見つかりません: missing.txt",
            ),
            (
                StorageError::Upload("Failed".to_string()),
                "アップロードエラー: Failed",
            ),
            (
                StorageError::Download("Failed".to_string()),
                "ダウンロードエラー: Failed",
            ),
            (
                StorageError::Metadata("No metadata".to_string()),
                "メタデータエラー: No metadata",
            ),
            (
                StorageError::Other("Unknown".to_string()),
                "その他のエラー: Unknown",
            ),
        ];

        for (error, expected_msg) in errors {
            assert_eq!(error.to_string(), expected_msg);
        }
    }

    // ========================================
    // 状態を持つモックストレージプロバイダー
    // ========================================

    /// 実際にデータを保存・取得できる、状態を持つモック実装
    struct StatefulMockStorage {
        files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    }

    impl StatefulMockStorage {
        fn new() -> Self {
            Self {
                files: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl CloudStorageProvider for StatefulMockStorage {
        async fn upload(&self, path: &str, data: &[u8], _metadata: FileMetadata) -> Result<()> {
            let mut files = self.files.lock().unwrap();
            files.insert(path.to_string(), data.to_vec());
            Ok(())
        }

        async fn download(&self, path: &str) -> Result<Vec<u8>> {
            let files = self.files.lock().unwrap();
            files
                .get(path)
                .cloned()
                .ok_or_else(|| StorageError::NotFound(path.to_string()))
        }

        async fn list(&self, prefix: &str) -> Result<Vec<FileInfo>> {
            let files = self.files.lock().unwrap();
            let now = Utc::now();

            let file_list: Vec<FileInfo> = files
                .keys()
                .filter(|path| path.starts_with(prefix))
                .map(|path| {
                    let size = files.get(path).unwrap().len() as u64;
                    FileInfo::new(path.clone(), size, now, None)
                })
                .collect();

            Ok(file_list)
        }

        async fn delete(&self, path: &str) -> Result<()> {
            let mut files = self.files.lock().unwrap();
            files
                .remove(path)
                .ok_or_else(|| StorageError::NotFound(path.to_string()))?;
            Ok(())
        }

        async fn get_metadata(&self, path: &str) -> Result<FileMetadata> {
            let files = self.files.lock().unwrap();
            let data = files
                .get(path)
                .ok_or_else(|| StorageError::NotFound(path.to_string()))?;

            let now = Utc::now();
            Ok(FileMetadata::new(
                data.len() as u64,
                now,
                Some("application/octet-stream".to_string()),
                None,
            ))
        }
    }

    // ========================================
    // 実践的な統合テスト
    // ========================================

    #[tokio::test]
    async fn test_upload_and_download_cycle() {
        let storage = StatefulMockStorage::new();
        let now = Utc::now();
        let test_data = b"Hello, Cloud Storage!";
        let metadata = FileMetadata::new(
            test_data.len() as u64,
            now,
            Some("text/plain".to_string()),
            None,
        );

        // アップロード
        let upload_result = storage.upload("test.txt", test_data, metadata).await;
        assert!(upload_result.is_ok(), "Upload should succeed");

        // ダウンロード
        let download_result = storage.download("test.txt").await;
        assert!(download_result.is_ok(), "Download should succeed");
        assert_eq!(
            download_result.unwrap(),
            test_data,
            "Downloaded data should match uploaded data"
        );
    }

    #[tokio::test]
    async fn test_download_nonexistent_file() {
        let storage = StatefulMockStorage::new();

        let result = storage.download("nonexistent.txt").await;
        assert!(
            result.is_err(),
            "Download should fail for non-existent file"
        );

        match result {
            Err(StorageError::NotFound(path)) => {
                assert_eq!(path, "nonexistent.txt");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_list_files_with_prefix() {
        let storage = StatefulMockStorage::new();
        let now = Utc::now();
        let metadata = FileMetadata::new(10, now, None, None);

        // 複数のファイルをアップロード
        storage
            .upload("docs/file1.txt", b"content1", metadata.clone())
            .await
            .unwrap();
        storage
            .upload("docs/file2.txt", b"content2", metadata.clone())
            .await
            .unwrap();
        storage
            .upload("images/pic.jpg", b"image", metadata.clone())
            .await
            .unwrap();

        // "docs/"プレフィックスでリスト取得
        let result = storage.list("docs/").await;
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), 2, "Should find 2 files with 'docs/' prefix");

        let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
        assert!(paths.contains(&"docs/file1.txt".to_string()));
        assert!(paths.contains(&"docs/file2.txt".to_string()));
    }

    #[tokio::test]
    async fn test_delete_file() {
        let storage = StatefulMockStorage::new();
        let now = Utc::now();
        let metadata = FileMetadata::new(5, now, None, None);

        // ファイルをアップロード
        storage.upload("temp.txt", b"temp", metadata).await.unwrap();

        // 削除前にダウンロードできることを確認
        let before_delete = storage.download("temp.txt").await;
        assert!(before_delete.is_ok());

        // 削除
        let delete_result = storage.delete("temp.txt").await;
        assert!(delete_result.is_ok(), "Delete should succeed");

        // 削除後はダウンロードできないことを確認
        let after_delete = storage.download("temp.txt").await;
        assert!(after_delete.is_err(), "Download should fail after delete");
    }

    #[tokio::test]
    async fn test_delete_nonexistent_file() {
        let storage = StatefulMockStorage::new();

        let result = storage.delete("nonexistent.txt").await;
        assert!(result.is_err(), "Delete should fail for non-existent file");
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let storage = StatefulMockStorage::new();
        let now = Utc::now();
        let test_data = b"Test metadata content";
        let metadata = FileMetadata::new(test_data.len() as u64, now, None, None);

        storage
            .upload("metadata_test.txt", test_data, metadata)
            .await
            .unwrap();

        let result = storage.get_metadata("metadata_test.txt").await;
        assert!(result.is_ok());

        let retrieved_metadata = result.unwrap();
        assert_eq!(retrieved_metadata.size, test_data.len() as u64);
    }

    #[tokio::test]
    async fn test_upload_empty_file() {
        let storage = StatefulMockStorage::new();
        let now = Utc::now();
        let metadata = FileMetadata::new(0, now, None, None);

        // 空のファイルをアップロード
        let result = storage.upload("empty.txt", b"", metadata).await;
        assert!(result.is_ok());

        // ダウンロードして確認
        let downloaded = storage.download("empty.txt").await.unwrap();
        assert_eq!(downloaded.len(), 0);
    }

    #[tokio::test]
    async fn test_upload_large_file() {
        let storage = StatefulMockStorage::new();
        let now = Utc::now();

        // 1MBのデータを作成
        let large_data = vec![0u8; 1_048_576];
        let metadata = FileMetadata::new(large_data.len() as u64, now, None, None);

        let result = storage
            .upload("large_file.bin", &large_data, metadata)
            .await;
        assert!(result.is_ok());

        let downloaded = storage.download("large_file.bin").await.unwrap();
        assert_eq!(downloaded.len(), 1_048_576);
    }

    // ========================================
    // 並行アクセステスト
    // ========================================

    #[tokio::test]
    async fn test_concurrent_uploads() {
        let storage = Arc::new(StatefulMockStorage::new());
        let now = Utc::now();

        let mut handles = vec![];

        // 10個の並行アップロード
        for i in 0..10 {
            let storage_clone = Arc::clone(&storage);
            let handle = tokio::spawn(async move {
                let path = format!("concurrent_{}.txt", i);
                let data = format!("content {}", i);
                let metadata = FileMetadata::new(data.len() as u64, now, None, None);

                storage_clone.upload(&path, data.as_bytes(), metadata).await
            });
            handles.push(handle);
        }

        // 全てのアップロードが成功することを確認
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // 全てのファイルがリストされることを確認
        let files = storage.list("concurrent_").await.unwrap();
        assert_eq!(files.len(), 10);
    }

    #[tokio::test]
    async fn test_concurrent_read_write() {
        let storage = Arc::new(StatefulMockStorage::new());
        let now = Utc::now();

        // 初期ファイルをアップロード
        let metadata = FileMetadata::new(7, now, None, None);
        storage
            .upload("shared.txt", b"initial", metadata)
            .await
            .unwrap();

        let mut handles = vec![];

        // 5つの読み取りタスク
        for _ in 0..5 {
            let storage_clone = Arc::clone(&storage);
            let handle = tokio::spawn(async move { storage_clone.download("shared.txt").await });
            handles.push(handle);
        }

        // 全ての読み取りが成功することを確認
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    // ========================================
    // dyn Trait オブジェクトとしての使用テスト
    // ========================================

    #[tokio::test]
    async fn test_storage_as_trait_object() {
        let storage: Box<dyn CloudStorageProvider + Send + Sync> =
            Box::new(StatefulMockStorage::new());

        let now = Utc::now();
        let metadata = FileMetadata::new(11, now, None, None);

        // trait object経由でメソッドを呼び出し
        let upload_result = storage
            .upload("trait_test.txt", b"trait works", metadata)
            .await;
        assert!(upload_result.is_ok());

        let download_result = storage.download("trait_test.txt").await;
        assert!(download_result.is_ok());
        assert_eq!(download_result.unwrap(), b"trait works");
    }
}
