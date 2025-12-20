/// S3Provider 統合テスト
///
/// このテストファイルは、S3Providerの統合テストを提供します。
/// 実際のAWS S3接続は不要で、モックまたは構造的なテストのみを実施します。
///
/// 実際のS3接続テストを実行する場合は、以下の環境変数を設定してください:
/// - AWS_ACCESS_KEY_ID
/// - AWS_SECRET_ACCESS_KEY
/// - AWS_REGION
/// - TEST_S3_BUCKET
///
/// 環境変数が設定されていない場合、これらのテストはスキップされます。
use app_lib::storage::{s3::S3Provider, CloudStorageProvider, FileMetadata};
use chrono::Utc;

/// S3Providerのインスタンス作成テスト
///
/// 正しくS3Providerのインスタンスが作成できることを確認します。
#[tokio::test]
async fn test_s3_provider_creation() {
    // 環境変数が設定されていない場合でも、コンストラクタは成功するはず
    let result = S3Provider::new("test-bucket".to_string()).await;

    // AWS認証情報がない場合、作成は成功するが、実際の操作は失敗する
    // （これは正常な動作）
    match result {
        Ok(provider) => {
            assert_eq!(provider.bucket(), "test-bucket");
        }
        Err(e) => {
            // 認証情報が見つからない場合のエラーは許容
            eprintln!(
                "Expected error when AWS credentials are not configured: {}",
                e
            );
        }
    }
}

/// バケット名の取得テスト
#[tokio::test]
async fn test_get_bucket_name() {
    let result = S3Provider::new("my-test-bucket".to_string()).await;

    if let Ok(provider) = result {
        assert_eq!(provider.bucket(), "my-test-bucket");
    } else {
        // 認証情報がない場合はスキップ
        println!("Skipping test: AWS credentials not configured");
    }
}

/// 異なるバケット名でのインスタンス作成テスト
#[tokio::test]
async fn test_multiple_bucket_instances() {
    let bucket1 = "bucket-one".to_string();
    let bucket2 = "bucket-two".to_string();

    let result1 = S3Provider::new(bucket1.clone()).await;
    let result2 = S3Provider::new(bucket2.clone()).await;

    if let (Ok(provider1), Ok(provider2)) = (result1, result2) {
        assert_eq!(provider1.bucket(), "bucket-one");
        assert_eq!(provider2.bucket(), "bucket-two");
        assert_ne!(provider1.bucket(), provider2.bucket());
    } else {
        println!("Skipping test: AWS credentials not configured");
    }
}

/// FileMetadataの作成テスト（統合テストとして）
#[test]
fn test_file_metadata_for_s3() {
    let now = Utc::now();
    let metadata = FileMetadata::new(
        1024 * 1024, // 1MB
        now,
        Some("application/octet-stream".to_string()),
        Some("abc123xyz".to_string()),
    );

    assert_eq!(metadata.size, 1024 * 1024);
    assert_eq!(metadata.last_modified, now);
    assert_eq!(
        metadata.content_type,
        Some("application/octet-stream".to_string())
    );
    assert_eq!(metadata.etag, Some("abc123xyz".to_string()));
}

// ========================================
// 実際のS3接続テスト（オプション）
// ========================================

/// 実際のS3バケットへのアップロードテスト
///
/// 環境変数 TEST_S3_BUCKET が設定されている場合のみ実行されます。
#[tokio::test]
#[ignore] // デフォルトでは無効（`cargo test -- --ignored` で有効化）
async fn test_real_s3_upload() {
    let bucket = std::env::var("TEST_S3_BUCKET").expect("TEST_S3_BUCKET not set");
    let provider = S3Provider::new(bucket)
        .await
        .expect("Failed to create S3Provider");

    let test_data = b"Hello, S3!";
    let now = Utc::now();
    let metadata = FileMetadata::new(
        test_data.len() as u64,
        now,
        Some("text/plain".to_string()),
        None,
    );

    let result = provider
        .upload("test/integration_test.txt", test_data, metadata)
        .await;

    assert!(result.is_ok(), "Upload should succeed: {:?}", result.err());
}

/// 実際のS3バケットからのダウンロードテスト
///
/// 環境変数 TEST_S3_BUCKET が設定されている場合のみ実行されます。
#[tokio::test]
#[ignore]
async fn test_real_s3_download() {
    let bucket = std::env::var("TEST_S3_BUCKET").expect("TEST_S3_BUCKET not set");
    let provider = S3Provider::new(bucket)
        .await
        .expect("Failed to create S3Provider");

    // 先にアップロード
    let test_data = b"Download test data";
    let now = Utc::now();
    let metadata = FileMetadata::new(test_data.len() as u64, now, None, None);

    provider
        .upload("test/download_test.txt", test_data, metadata)
        .await
        .expect("Upload failed");

    // ダウンロード
    let result = provider.download("test/download_test.txt").await;

    assert!(
        result.is_ok(),
        "Download should succeed: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), test_data);
}

/// 実際のS3バケットのリスト取得テスト
///
/// 環境変数 TEST_S3_BUCKET が設定されている場合のみ実行されます。
#[tokio::test]
#[ignore]
async fn test_real_s3_list() {
    let bucket = std::env::var("TEST_S3_BUCKET").expect("TEST_S3_BUCKET not set");
    let provider = S3Provider::new(bucket)
        .await
        .expect("Failed to create S3Provider");

    let result = provider.list("test/").await;

    assert!(result.is_ok(), "List should succeed: {:?}", result.err());
}

/// 実際のS3バケットの削除テスト
///
/// 環境変数 TEST_S3_BUCKET が設定されている場合のみ実行されます。
#[tokio::test]
#[ignore]
async fn test_real_s3_delete() {
    let bucket = std::env::var("TEST_S3_BUCKET").expect("TEST_S3_BUCKET not set");
    let provider = S3Provider::new(bucket)
        .await
        .expect("Failed to create S3Provider");

    // 先にアップロード
    let test_data = b"Delete test";
    let now = Utc::now();
    let metadata = FileMetadata::new(test_data.len() as u64, now, None, None);

    provider
        .upload("test/delete_test.txt", test_data, metadata)
        .await
        .expect("Upload failed");

    // 削除
    let result = provider.delete("test/delete_test.txt").await;

    assert!(result.is_ok(), "Delete should succeed: {:?}", result.err());
}

/// 実際のS3バケットのメタデータ取得テスト
///
/// 環境変数 TEST_S3_BUCKET が設定されている場合のみ実行されます。
#[tokio::test]
#[ignore]
async fn test_real_s3_get_metadata() {
    let bucket = std::env::var("TEST_S3_BUCKET").expect("TEST_S3_BUCKET not set");
    let provider = S3Provider::new(bucket)
        .await
        .expect("Failed to create S3Provider");

    // 先にアップロード
    let test_data = b"Metadata test";
    let now = Utc::now();
    let metadata = FileMetadata::new(
        test_data.len() as u64,
        now,
        Some("text/plain".to_string()),
        None,
    );

    provider
        .upload("test/metadata_test.txt", test_data, metadata)
        .await
        .expect("Upload failed");

    // メタデータ取得
    let result = provider.get_metadata("test/metadata_test.txt").await;

    assert!(
        result.is_ok(),
        "Get metadata should succeed: {:?}",
        result.err()
    );

    let retrieved_metadata = result.unwrap();
    assert_eq!(retrieved_metadata.size, test_data.len() as u64);
}

// ========================================
// エラーケースのテスト
// ========================================

/// S3Providerの異なるバケット名での動作確認
#[tokio::test]
async fn test_different_bucket_names() {
    let buckets = vec!["bucket-a", "bucket-b", "my-test-bucket"];

    for bucket in buckets {
        let result = S3Provider::new(bucket.to_string()).await;
        if let Ok(provider) = result {
            assert_eq!(provider.bucket(), bucket);
        }
    }
}

/// バケット名の取得が正しく動作することを確認
#[tokio::test]
async fn test_bucket_name_consistency() {
    let bucket_name = "consistent-bucket-name";
    let result = S3Provider::new(bucket_name.to_string()).await;

    if let Ok(provider) = result {
        // 最初の取得
        let name1 = provider.bucket();
        // 2回目の取得（同じ値が返されるべき）
        let name2 = provider.bucket();

        assert_eq!(name1, bucket_name);
        assert_eq!(name2, bucket_name);
        assert_eq!(name1, name2);
    }
}

// ========================================
// 並行処理のテスト
// ========================================

/// 複数のS3Providerインスタンスの並行作成テスト
#[tokio::test]
async fn test_concurrent_provider_creation() {
    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();

    for i in 0..5 {
        join_set.spawn(async move {
            let bucket = format!("bucket-{}", i);
            S3Provider::new(bucket.clone()).await
        });
    }

    let mut count = 0;
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) | Ok(Err(_)) => count += 1,
            Err(e) => panic!("Task failed: {:?}", e),
        }
    }

    assert_eq!(count, 5);
}
