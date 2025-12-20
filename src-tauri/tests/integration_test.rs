/// 統合テスト
///
/// storage モジュールの統合テストを実施します。
use app_lib::storage::{FileInfo, FileMetadata};
use chrono::Utc;

#[test]
fn test_file_info_serialization() {
    // FileInfoのシリアライゼーションとデシリアライゼーションが正しく動作するか確認
    let now = Utc::now();
    let file_info = FileInfo::new(
        "test/file.txt".to_string(),
        1024,
        now,
        Some("etag123".to_string()),
    );

    let json = serde_json::to_string(&file_info).expect("Failed to serialize");
    let deserialized: FileInfo = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(file_info, deserialized);
}

#[test]
fn test_file_metadata_content_type_handling() {
    // FileMetadataのコンテンツタイプ処理を確認
    let now = Utc::now();

    // コンテンツタイプありの場合
    let metadata_with_ct = FileMetadata::new(1024, now, Some("application/pdf".to_string()), None);
    assert_eq!(
        metadata_with_ct.content_type,
        Some("application/pdf".to_string())
    );

    // コンテンツタイプなしの場合
    let metadata_without_ct = FileMetadata::new(1024, now, None, None);
    assert_eq!(metadata_without_ct.content_type, None);
}

#[test]
fn test_file_info_comparison() {
    // FileInfoの比較が正しく動作するか確認
    let now = Utc::now();
    let file1 = FileInfo::new("test.txt".to_string(), 1024, now, Some("etag1".to_string()));
    let file2 = FileInfo::new("test.txt".to_string(), 1024, now, Some("etag1".to_string()));
    let file3 = FileInfo::new(
        "test.txt".to_string(),
        2048, // サイズが異なる
        now,
        Some("etag1".to_string()),
    );

    assert_eq!(file1, file2); // 同じファイル
    assert_ne!(file1, file3); // サイズが異なるファイル
}
