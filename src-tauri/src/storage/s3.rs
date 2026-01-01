use super::{CloudStorageProvider, FileInfo, FileMetadata, Result, StorageError};
use crate::retry::{is_network_error, retry_with_backoff};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{primitives::ByteStream, Client};
use chrono::Utc;

/// AWS S3 ストレージプロバイダー
///
/// このプロバイダーは、AWS S3 APIを使用してファイルの
/// アップロード、ダウンロード、リスト、削除などの操作を提供します。
#[derive(Clone)]
pub struct S3Provider {
    client: Client,
    bucket: String,
}

impl S3Provider {
    /// 新しいS3Providerインスタンスを作成
    ///
    /// # 引数
    /// * `bucket` - 使用するS3バケット名
    ///
    /// # 戻り値
    /// 設定されたS3Providerインスタンス
    ///
    /// # エラー
    /// AWS認証情報の読み込みに失敗した場合はエラーを返す
    pub async fn new(bucket: String) -> Result<Self> {
        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
        let client = Client::new(&config);

        Ok(Self { client, bucket })
    }

    /// カスタム設定でS3Providerインスタンスを作成
    ///
    /// # 引数
    /// * `bucket` - 使用するS3バケット名
    /// * `config` - AWS SDK設定
    ///
    /// # 戻り値
    /// 設定されたS3Providerインスタンス
    pub fn with_config(bucket: String, config: &aws_config::SdkConfig) -> Self {
        let client = Client::new(config);
        Self { client, bucket }
    }

    /// バケット名を取得
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[async_trait::async_trait]
impl CloudStorageProvider for S3Provider {
    async fn upload(&self, path: &str, data: &[u8], metadata: FileMetadata) -> Result<()> {
        let path = path.to_string();
        let bucket = self.bucket.clone();
        let client = self.client.clone();
        let data_vec = data.to_vec();
        let metadata_clone = metadata.clone();

        // Retry with exponential backoff for network errors
        retry_with_backoff(|| async {
            let body = ByteStream::from(data_vec.clone());

            let mut request = client.put_object().bucket(&bucket).key(&path).body(body);

            // コンテンツタイプを設定（存在する場合）
            if let Some(ref content_type) = metadata_clone.content_type {
                request = request.content_type(content_type.clone());
            }

            // メタデータを設定
            // Note: S3 automatically calculates MD5 hash and sets it as ETag
            // No need to set custom metadata for etag
            request = request
                .metadata("last-modified", metadata_clone.last_modified.to_rfc3339())
                .metadata("size", metadata_clone.size.to_string());

            // アップロードを実行
            request.send().await.map_err(|e| {
                let error_msg = format!("S3へのアップロードに失敗しました: {}", e);
                if is_network_error(&error_msg) {
                    log::warn!("Network error during upload, will retry: {}", error_msg);
                }
                StorageError::Upload(error_msg)
            })?;

            Ok(())
        })
        .await
    }

    async fn download(&self, path: &str) -> Result<Vec<u8>> {
        let path = path.to_string();
        let bucket = self.bucket.clone();
        let client = self.client.clone();

        // Retry with exponential backoff for network errors
        retry_with_backoff(|| async {
            let response = client
                .get_object()
                .bucket(&bucket)
                .key(&path)
                .send()
                .await
                .map_err(|e| {
                    let error_msg = e.to_string();
                    // NotFoundエラーを特別に処理（リトライしない）
                    if error_msg.contains("NoSuchKey") {
                        StorageError::NotFound(path.clone())
                    } else {
                        let download_error =
                            format!("S3からのダウンロードに失敗しました: {}", error_msg);
                        if is_network_error(&download_error) {
                            log::warn!(
                                "Network error during download, will retry: {}",
                                download_error
                            );
                        }
                        StorageError::Download(download_error)
                    }
                })?;

            // ボディをバイト配列に変換
            let body_bytes = response
                .body
                .collect()
                .await
                .map_err(|e| {
                    let error_msg = format!("レスポンスボディの読み取りに失敗しました: {}", e);
                    if is_network_error(&error_msg) {
                        log::warn!(
                            "Network error while reading body, will retry: {}",
                            error_msg
                        );
                    }
                    StorageError::Download(error_msg)
                })?
                .into_bytes();

            Ok(body_bytes.to_vec())
        })
        .await
    }

    async fn list(&self, prefix: &str) -> Result<Vec<FileInfo>> {
        let prefix = prefix.to_string();
        let bucket = self.bucket.clone();
        let client = self.client.clone();

        // Retry with exponential backoff for network errors
        retry_with_backoff(|| async {
            let mut file_list = Vec::new();
            let mut continuation_token: Option<String> = None;

            // ページネーションを使用してすべてのオブジェクトを取得
            loop {
                let mut request = client
                    .list_objects_v2()
                    .bucket(&bucket)
                    .prefix(&prefix)
                    .max_keys(1000);

                if let Some(ref token) = continuation_token {
                    request = request.continuation_token(token.clone());
                }

                let response = request.send().await.map_err(|e| {
                    let error_msg = format!("S3オブジェクトのリスト取得に失敗しました: {}", e);
                    if is_network_error(&error_msg) {
                        log::warn!("Network error during list, will retry: {}", error_msg);
                    }
                    StorageError::Other(error_msg)
                })?;

                // 次のページがあるかチェック（先にチェックしてムーブ問題を回避）
                let is_truncated = response.is_truncated().unwrap_or(false);
                let next_token = response.next_continuation_token;

                // オブジェクトを処理
                if let Some(contents) = response.contents {
                    for object in contents {
                        if let Some(key) = object.key() {
                            let size = object.size().unwrap_or(0) as u64;
                            let last_modified = object
                                .last_modified()
                                .and_then(|dt| {
                                    // AWS DateTimeをUNIXタイムスタンプ経由で変換
                                    let secs = dt.secs();
                                    let nanos = dt.subsec_nanos();
                                    chrono::DateTime::from_timestamp(secs, nanos)
                                })
                                .unwrap_or_else(Utc::now);
                            // AWS S3 ETag is surrounded by quotes, remove them to match local MD5 hash format
                            let etag = object.e_tag().map(|s| s.trim_matches('"').to_string());

                            file_list.push(FileInfo::new(
                                key.to_string(),
                                size,
                                last_modified,
                                etag,
                            ));
                        }
                    }
                }

                if !is_truncated {
                    break;
                }
                continuation_token = next_token;
            }

            Ok(file_list)
        })
        .await
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let path = path.to_string();
        let bucket = self.bucket.clone();
        let client = self.client.clone();

        // Retry with exponential backoff for network errors
        retry_with_backoff(|| async {
            client
                .delete_object()
                .bucket(&bucket)
                .key(&path)
                .send()
                .await
                .map_err(|e| {
                    let error_msg = format!("S3オブジェクトの削除に失敗しました: {}", e);
                    if is_network_error(&error_msg) {
                        log::warn!("Network error during delete, will retry: {}", error_msg);
                    }
                    StorageError::Other(error_msg)
                })?;

            Ok(())
        })
        .await
    }

    async fn get_metadata(&self, path: &str) -> Result<FileMetadata> {
        let path = path.to_string();
        let bucket = self.bucket.clone();
        let client = self.client.clone();

        // Retry with exponential backoff for network errors
        retry_with_backoff(|| async {
            let response = client
                .head_object()
                .bucket(&bucket)
                .key(&path)
                .send()
                .await
                .map_err(|e| {
                    let error_msg = e.to_string();
                    // NotFoundエラーを特別に処理（リトライしない）
                    if error_msg.contains("NotFound") {
                        StorageError::NotFound(path.clone())
                    } else {
                        let metadata_error = format!(
                            "S3オブジェクトのメタデータ取得に失敗しました: {}",
                            error_msg
                        );
                        if is_network_error(&metadata_error) {
                            log::warn!(
                                "Network error during get_metadata, will retry: {}",
                                metadata_error
                            );
                        }
                        StorageError::Metadata(metadata_error)
                    }
                })?;

            let size = response.content_length().unwrap_or(0) as u64;
            let last_modified = response
                .last_modified()
                .and_then(|dt| {
                    // AWS DateTimeをUNIXタイムスタンプ経由で変換
                    let secs = dt.secs();
                    let nanos = dt.subsec_nanos();
                    chrono::DateTime::from_timestamp(secs, nanos)
                })
                .unwrap_or_else(Utc::now);
            let content_type = response.content_type().map(|s| s.to_string());
            // AWS S3 ETag is surrounded by quotes, remove them to match local MD5 hash format
            let etag = response.e_tag().map(|s| s.trim_matches('"').to_string());

            Ok(FileMetadata::new(size, last_modified, content_type, etag))
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // ========================================
    // 構造体とコンストラクタのテスト
    // ========================================

    #[test]
    fn test_s3_provider_with_config() {
        // モック用の設定を作成（実際のAWS接続は不要）
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
            let provider = S3Provider::with_config("test-bucket".to_string(), &config);

            assert_eq!(provider.bucket(), "test-bucket");
        });
    }

    #[test]
    fn test_s3_provider_bucket_name() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
            let provider = S3Provider::with_config("my-storage-bucket".to_string(), &config);

            assert_eq!(provider.bucket(), "my-storage-bucket");
        });
    }

    // ========================================
    // メタデータ処理のテスト
    // ========================================

    #[test]
    fn test_metadata_creation() {
        let now = Utc.with_ymd_and_hms(2025, 12, 17, 10, 0, 0).unwrap();
        let metadata = FileMetadata::new(
            1024,
            now,
            Some("application/pdf".to_string()),
            Some("etag123".to_string()),
        );

        assert_eq!(metadata.size, 1024);
        assert_eq!(metadata.last_modified, now);
        assert_eq!(metadata.content_type, Some("application/pdf".to_string()));
        assert_eq!(metadata.etag, Some("etag123".to_string()));
    }

    #[test]
    fn test_file_info_creation() {
        let now = Utc::now();
        let file_info = FileInfo::new(
            "documents/test.pdf".to_string(),
            2048,
            now,
            Some("abc123".to_string()),
        );

        assert_eq!(file_info.path, "documents/test.pdf");
        assert_eq!(file_info.size, 2048);
        assert_eq!(file_info.last_modified, now);
        assert_eq!(file_info.etag, Some("abc123".to_string()));
    }

    // ========================================
    // エラーハンドリングテスト
    // ========================================

    #[test]
    fn test_storage_error_display() {
        // NotFoundエラー
        let not_found = StorageError::NotFound("missing.txt".to_string());
        assert_eq!(
            not_found.to_string(),
            "ファイルが見つかりません: missing.txt"
        );

        // Uploadエラー
        let upload_err = StorageError::Upload("Network error".to_string());
        assert_eq!(upload_err.to_string(), "アップロードエラー: Network error");

        // Downloadエラー
        let download_err = StorageError::Download("Connection lost".to_string());
        assert_eq!(
            download_err.to_string(),
            "ダウンロードエラー: Connection lost"
        );
    }

    #[test]
    fn test_storage_error_variants() {
        // 各エラーバリアントが正しく作成できることを確認
        let network_err = StorageError::Network("Timeout".to_string());
        let auth_err = StorageError::Authentication("Invalid token".to_string());
        let metadata_err = StorageError::Metadata("Missing".to_string());

        assert!(network_err.to_string().contains("Timeout"));
        assert!(auth_err.to_string().contains("Invalid token"));
        assert!(metadata_err.to_string().contains("Missing"));
    }

    // ========================================
    // FileInfo と FileMetadata の実用的なテスト
    // ========================================

    #[test]
    fn test_file_info_path_normalization() {
        // パスが正しく保持されることを確認
        let now = Utc::now();
        let paths = vec![
            "documents/file.txt",
            "images/2025/photo.jpg",
            "backup/データ/ファイル.pdf",
        ];

        for path in paths {
            let file_info = FileInfo::new(path.to_string(), 1024, now, None);
            assert_eq!(file_info.path, path);
        }
    }

    #[test]
    fn test_file_metadata_etag_handling() {
        // ETagの有無での動作確認
        let now = Utc::now();

        // ETagありの場合
        let with_etag = FileMetadata::new(
            1024,
            now,
            Some("text/plain".to_string()),
            Some("abc123".to_string()),
        );
        assert!(with_etag.etag.is_some());
        assert_eq!(with_etag.etag.unwrap(), "abc123");

        // ETagなしの場合
        let without_etag = FileMetadata::new(1024, now, Some("text/plain".to_string()), None);
        assert!(without_etag.etag.is_none());
    }

    #[test]
    fn test_file_info_size_edge_cases() {
        // ファイルサイズのエッジケースをテスト
        let now = Utc::now();

        // ゼロバイト
        let zero_size = FileInfo::new("empty.txt".to_string(), 0, now, None);
        assert_eq!(zero_size.size, 0);

        // 最大サイズ（5GB）
        let max_size = FileInfo::new("large.bin".to_string(), 5_368_709_120, now, None);
        assert_eq!(max_size.size, 5_368_709_120);

        // 100MB（プログレスバー表示の境界）
        let progress_threshold = FileInfo::new("medium.zip".to_string(), 104_857_600, now, None);
        assert_eq!(progress_threshold.size, 104_857_600);
    }

    // 注意: 実際のS3接続テストは統合テストで実施します（tests/ディレクトリ）
    // ここではモックまたは構造体のテストのみを行います
}
