//! Tauri commands module
//!
//! This module provides the bridge between the frontend (React) and backend (Rust).
//! All functions here are exposed to the frontend via the Tauri command system.

use serde::{Deserialize, Serialize};

use crate::auth::{AwsCredentials, CredentialError, CredentialManager};
use crate::storage::{FileInfo, StorageError};
use crate::sync::{ComparisonResult, SyncError, SyncState};

/// Custom error type for Tauri commands
/// This error type is serializable and can be sent to the frontend
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Credential error: {0}")]
    Credential(#[from] CredentialError),

    #[error("Sync error: {0}")]
    Sync(#[from] SyncError),

    #[error("Not configured: {0}")]
    NotConfigured(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

// Implement Serialize for CommandError to make it compatible with Tauri
impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Result type for Tauri commands
pub type CommandResult<T> = Result<T, CommandError>;

/// Request structure for setting credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCredentialsRequest {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub bucket_name: String,
}

/// Response structure for credential operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialsResponse {
    pub success: bool,
    pub message: String,
}

/// Response structure for getting credentials (with masked secret)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCredentialsResponse {
    pub has_credentials: bool,
    pub access_key_id: Option<String>,
    pub region: Option<String>,
    pub bucket_name: Option<String>,
    // Secret access key is never returned for security reasons
}

/// Response structure for connection test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResponse {
    pub connected: bool,
    pub message: String,
    pub region: Option<String>,
    pub bucket_name: Option<String>,
}

/// Request structure for sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFilesRequest {
    pub local_path: String,
    pub remote_prefix: String,
}

/// Response structure for sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFilesResponse {
    pub success: bool,
    pub files_uploaded: usize,
    pub files_downloaded: usize,
    pub conflicts_resolved: usize,
    pub message: String,
}

/// Request structure for listing files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesRequest {
    pub prefix: String,
}

/// Response structure for listing files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesResponse {
    pub files: Vec<FileInfo>,
    pub total_count: usize,
}

/// Response structure for getting sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    pub comparisons: Vec<ComparisonResultDto>,
    pub in_sync_count: usize,
    pub needs_upload_count: usize,
    pub needs_download_count: usize,
    pub conflict_count: usize,
}

/// DTO for ComparisonResult (simplified for frontend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResultDto {
    pub path: String,
    pub state: SyncState,
    pub local_size: Option<u64>,
    pub remote_size: Option<u64>,
    pub local_modified: Option<String>,
    pub remote_modified: Option<String>,
}

impl From<ComparisonResult> for ComparisonResultDto {
    fn from(result: ComparisonResult) -> Self {
        Self {
            path: result.path,
            state: result.state,
            local_size: result.local_info.as_ref().map(|info| info.size),
            remote_size: result.remote_info.as_ref().map(|info| info.size),
            local_modified: result
                .local_info
                .as_ref()
                .map(|info| info.last_modified.to_rfc3339()),
            remote_modified: result
                .remote_info
                .as_ref()
                .map(|info| info.last_modified.to_rfc3339()),
        }
    }
}

/// Set AWS credentials
///
/// This command stores the AWS credentials in the OS keychain.
/// The credentials will be used for all subsequent S3 operations.
///
/// # Arguments
/// * `request` - The credentials to store
///
/// # Returns
/// A response indicating success or failure
#[tauri::command]
pub async fn set_credentials(
    request: SetCredentialsRequest,
) -> CommandResult<CredentialsResponse> {
    log::info!("set_credentials called");

    // Validate input
    if request.access_key_id.is_empty() {
        log::error!("Access key ID is empty");
        return Err(CommandError::InvalidInput(
            "Access key ID cannot be empty".to_string(),
        ));
    }
    if request.region.is_empty() {
        log::error!("Region is empty");
        return Err(CommandError::InvalidInput(
            "Region cannot be empty".to_string(),
        ));
    }
    if request.bucket_name.is_empty() {
        log::error!("Bucket name is empty");
        return Err(CommandError::InvalidInput(
            "Bucket name cannot be empty".to_string(),
        ));
    }

    // If secret key is empty, try to load existing credentials and keep the existing secret key
    let secret_access_key = if request.secret_access_key.is_empty() {
        log::info!("Secret access key is empty, attempting to load existing value");
        let manager = CredentialManager::new();
        match manager.load_aws_credentials() {
            Ok(existing) => {
                log::info!("Using existing secret access key");
                existing.secret_access_key
            }
            Err(_) => {
                log::error!("No existing credentials found and secret key is empty");
                return Err(CommandError::InvalidInput(
                    "Secret access key cannot be empty".to_string(),
                ));
            }
        }
    } else {
        request.secret_access_key
    };

    let credentials = AwsCredentials {
        access_key_id: request.access_key_id,
        secret_access_key,
        region: request.region,
        bucket_name: request.bucket_name,
    };

    log::info!("Attempting to save credentials to keychain");
    let manager = CredentialManager::new();
    match manager.save_aws_credentials(&credentials) {
        Ok(_) => {
            log::info!("Credentials saved successfully");
            Ok(CredentialsResponse {
                success: true,
                message: "Credentials saved successfully".to_string(),
            })
        }
        Err(e) => {
            log::error!("Failed to save credentials: {}", e);
            Err(CommandError::Credential(e))
        }
    }
}

/// Test connection to AWS S3
///
/// This command verifies that the stored credentials are valid by attempting
/// to connect to S3 and access the specified bucket.
///
/// # Returns
/// A response indicating whether the connection was successful
#[tauri::command]
pub async fn test_connection() -> CommandResult<ConnectionTestResponse> {
    log::info!("test_connection called");

    // Load credentials
    let manager = CredentialManager::new();
    log::info!("Attempting to load credentials from keychain");

    let credentials = manager.load_aws_credentials().map_err(|e| {
        log::error!("Failed to load credentials: {}", e);
        CommandError::NotConfigured("AWS credentials not found. Please set credentials first.".to_string())
    })?;

    log::info!("Credentials loaded successfully");

    // Test actual connection to S3
    log::info!("Testing connection to S3 bucket: {}", credentials.bucket_name);

    // Create AWS config with behavior version
    let region = aws_sdk_s3::config::Region::new(credentials.region.clone());
    let creds = aws_sdk_s3::config::Credentials::new(
        credentials.access_key_id.clone(),
        credentials.secret_access_key.clone(),
        None,
        None,
        "file-funeral",
    );

    let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region)
        .credentials_provider(creds)
        .load()
        .await;

    let client = aws_sdk_s3::Client::new(&sdk_config);

    // Try to access the bucket (head_bucket is a lightweight operation)
    match client.head_bucket()
        .bucket(&credentials.bucket_name)
        .send()
        .await
    {
        Ok(_) => {
            log::info!("Successfully connected to S3 bucket: {}", credentials.bucket_name);
            Ok(ConnectionTestResponse {
                connected: true,
                message: format!(
                    "Successfully connected to bucket '{}' in region '{}'",
                    credentials.bucket_name, credentials.region
                ),
                region: Some(credentials.region),
                bucket_name: Some(credentials.bucket_name),
            })
        }
        Err(e) => {
            log::error!("Failed to connect to S3: {:?}", e);
            let error_message = format!(
                "Failed to connect to S3: {}. Please check your credentials and bucket name.",
                e
            );
            Ok(ConnectionTestResponse {
                connected: false,
                message: error_message,
                region: Some(credentials.region),
                bucket_name: Some(credentials.bucket_name),
            })
        }
    }
}

/// Get stored AWS credentials (without secret key for security)
///
/// This command retrieves the stored credentials and returns them without the secret access key.
/// Used to populate the settings form when the user navigates back to the settings page.
///
/// # Returns
/// A response containing the non-sensitive credential information
#[tauri::command]
pub async fn get_credentials() -> CommandResult<GetCredentialsResponse> {
    log::info!("get_credentials called");

    let manager = CredentialManager::new();

    match manager.load_aws_credentials() {
        Ok(credentials) => {
            log::info!("Credentials loaded successfully");
            Ok(GetCredentialsResponse {
                has_credentials: true,
                access_key_id: Some(credentials.access_key_id),
                region: Some(credentials.region),
                bucket_name: Some(credentials.bucket_name),
            })
        }
        Err(e) => {
            log::info!("No credentials found: {}", e);
            Ok(GetCredentialsResponse {
                has_credentials: false,
                access_key_id: None,
                region: None,
                bucket_name: None,
            })
        }
    }
}

/// List files from cloud storage
///
/// This command retrieves a list of files from the cloud storage provider
/// matching the given prefix.
///
/// # Arguments
/// * `request` - The prefix to filter files
///
/// # Returns
/// A list of files matching the prefix
#[tauri::command]
pub async fn list_files(_request: ListFilesRequest) -> CommandResult<ListFilesResponse> {
    // Load credentials
    let manager = CredentialManager::new();
    let _credentials = manager.load_aws_credentials().map_err(|_| {
        CommandError::NotConfigured("AWS credentials not found. Please set credentials first.".to_string())
    })?;

    // TODO: Create S3Provider and list files
    // For now, return empty list
    // This will be implemented when we integrate the actual S3Provider

    Ok(ListFilesResponse {
        files: Vec::new(),
        total_count: 0,
    })
}

/// Get sync status for files
///
/// This command compares local files with remote files and returns the sync state
/// for each file.
///
/// # Arguments
/// * `local_path` - The local directory path to check
/// * `remote_prefix` - The remote prefix to compare against
///
/// # Returns
/// A status report showing which files need syncing
#[tauri::command]
pub async fn get_sync_status(
    local_path: String,
    _remote_prefix: String,
) -> CommandResult<SyncStatusResponse> {
    // Validate input
    if local_path.is_empty() {
        return Err(CommandError::InvalidInput(
            "Local path cannot be empty".to_string(),
        ));
    }

    // Load credentials
    let manager = CredentialManager::new();
    let _credentials = manager.load_aws_credentials().map_err(|_| {
        CommandError::NotConfigured("AWS credentials not found. Please set credentials first.".to_string())
    })?;

    // TODO: Implement actual sync status checking
    // 1. List local files in local_path
    // 2. List remote files with remote_prefix
    // 3. Compare each file using sync::compare_files
    // 4. Return comparison results

    Ok(SyncStatusResponse {
        comparisons: Vec::new(),
        in_sync_count: 0,
        needs_upload_count: 0,
        needs_download_count: 0,
        conflict_count: 0,
    })
}

/// Synchronize files between local and remote storage
///
/// This command performs the actual file synchronization:
/// - Uploads files that exist only locally or are newer locally
/// - Downloads files that exist only remotely or are newer remotely
/// - Resolves conflicts using the "both-save" approach
///
/// # Arguments
/// * `request` - Sync request containing local path and remote prefix
///
/// # Returns
/// A summary of the sync operation
#[tauri::command]
pub async fn sync_files(request: SyncFilesRequest) -> CommandResult<SyncFilesResponse> {
    // Validate input
    if request.local_path.is_empty() {
        return Err(CommandError::InvalidInput(
            "Local path cannot be empty".to_string(),
        ));
    }

    // Load credentials
    let manager = CredentialManager::new();
    let _credentials = manager.load_aws_credentials().map_err(|_| {
        CommandError::NotConfigured("AWS credentials not found. Please set credentials first.".to_string())
    })?;

    // TODO: Implement actual sync logic
    // 1. Get sync status (compare files)
    // 2. For each file:
    //    - LocalOnly or LocalNewer: Upload
    //    - RemoteOnly or RemoteNewer: Download
    //    - Conflict: Resolve using sync::resolve_conflict
    // 3. Track progress and report back

    Ok(SyncFilesResponse {
        success: true,
        files_uploaded: 0,
        files_downloaded: 0,
        conflicts_resolved: 0,
        message: "Sync operation not yet implemented".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_credentials_request_validation() {
        // Valid request
        let valid_request = SetCredentialsRequest {
            access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            region: "us-west-2".to_string(),
            bucket_name: "my-bucket".to_string(),
        };
        assert!(!valid_request.access_key_id.is_empty());
        assert!(!valid_request.secret_access_key.is_empty());
    }

    #[test]
    fn test_comparison_result_dto_conversion() {
        use crate::storage::FileInfo;
        use crate::sync::ComparisonResult;
        use chrono::Utc;

        let now = Utc::now();
        let file_info = FileInfo::new("test.txt".to_string(), 100, now, Some("etag123".to_string()));

        let comparison = ComparisonResult {
            path: "test.txt".to_string(),
            state: SyncState::LocalOnly,
            local_info: Some(file_info),
            remote_info: None,
        };

        let dto: ComparisonResultDto = comparison.into();
        assert_eq!(dto.path, "test.txt");
        assert_eq!(dto.state, SyncState::LocalOnly);
        assert_eq!(dto.local_size, Some(100));
        assert!(dto.remote_size.is_none());
    }

    #[test]
    fn test_command_error_display() {
        let errors = vec![
            CommandError::NotConfigured("test".to_string()),
            CommandError::InvalidInput("empty field".to_string()),
            CommandError::OperationFailed("network error".to_string()),
        ];

        for error in errors {
            let msg = error.to_string();
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn test_sync_files_request_serialization() {
        let request = SyncFilesRequest {
            local_path: "/home/user/documents".to_string(),
            remote_prefix: "documents/".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: SyncFilesRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.local_path, deserialized.local_path);
        assert_eq!(request.remote_prefix, deserialized.remote_prefix);
    }

    #[test]
    fn test_list_files_response_serialization() {
        use crate::storage::FileInfo;
        use chrono::Utc;

        let now = Utc::now();
        let files = vec![
            FileInfo::new("file1.txt".to_string(), 100, now, None),
            FileInfo::new("file2.txt".to_string(), 200, now, None),
        ];

        let response = ListFilesResponse {
            total_count: files.len(),
            files,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ListFilesResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.total_count, deserialized.total_count);
        assert_eq!(response.files.len(), deserialized.files.len());
    }

    #[test]
    fn test_sync_status_response_counts() {
        let response = SyncStatusResponse {
            comparisons: Vec::new(),
            in_sync_count: 5,
            needs_upload_count: 3,
            needs_download_count: 2,
            conflict_count: 1,
        };

        assert_eq!(response.in_sync_count, 5);
        assert_eq!(response.needs_upload_count, 3);
        assert_eq!(response.needs_download_count, 2);
        assert_eq!(response.conflict_count, 1);
    }

    #[tokio::test]
    async fn test_set_credentials_validates_empty_fields() {
        let empty_access_key = SetCredentialsRequest {
            access_key_id: "".to_string(),
            secret_access_key: "secret".to_string(),
            region: "us-west-2".to_string(),
            bucket_name: "bucket".to_string(),
        };

        let result = set_credentials(empty_access_key).await;
        assert!(result.is_err());
        match result {
            Err(CommandError::InvalidInput(msg)) => {
                assert!(msg.contains("Access key ID"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_set_credentials_validates_all_fields() {
        // Clean up any existing credentials before testing
        let manager = crate::auth::CredentialManager::new();
        let _ = manager.delete_aws_credentials();

        let test_cases = vec![
            ("", "secret", "region", "bucket", "Access key ID"),
            ("access", "", "region", "bucket", "Secret access key"),  // Now fails because no existing credentials
            ("access", "secret", "", "bucket", "Region"),
            ("access", "secret", "region", "", "Bucket name"),
        ];

        for (access, secret, region, bucket, expected_error) in test_cases {
            let request = SetCredentialsRequest {
                access_key_id: access.to_string(),
                secret_access_key: secret.to_string(),
                region: region.to_string(),
                bucket_name: bucket.to_string(),
            };

            let result = set_credentials(request).await;
            assert!(result.is_err());
            match result {
                Err(CommandError::InvalidInput(msg)) => {
                    assert!(msg.contains(expected_error));
                }
                _ => panic!("Expected InvalidInput error for {}", expected_error),
            }
        }

        // Cleanup after test
        let _ = manager.delete_aws_credentials();
    }

    #[tokio::test]
    async fn test_list_files_empty_result() {
        // Clean up any existing credentials before testing
        let manager = crate::auth::CredentialManager::new();
        let _ = manager.delete_aws_credentials();

        let request = ListFilesRequest {
            prefix: "test/".to_string(),
        };

        // This will fail because credentials are not set in test environment
        // But we can test that the function exists and returns the correct error type
        let result = list_files(request).await;

        // Should return NotConfigured error since credentials aren't set
        assert!(result.is_err());
        match result {
            Err(CommandError::NotConfigured(_)) => {
                // Expected
            }
            _ => panic!("Expected NotConfigured error"),
        }

        // Cleanup after test
        let _ = manager.delete_aws_credentials();
    }

    #[tokio::test]
    async fn test_get_sync_status_validates_empty_path() {
        let result = get_sync_status("".to_string(), "remote/".to_string()).await;
        assert!(result.is_err());
        match result {
            Err(CommandError::InvalidInput(msg)) => {
                assert!(msg.contains("Local path"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[tokio::test]
    async fn test_sync_files_validates_empty_path() {
        let request = SyncFilesRequest {
            local_path: "".to_string(),
            remote_prefix: "remote/".to_string(),
        };

        let result = sync_files(request).await;
        assert!(result.is_err());
        match result {
            Err(CommandError::InvalidInput(msg)) => {
                assert!(msg.contains("Local path"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_connection_test_response_serialization() {
        let response = ConnectionTestResponse {
            connected: true,
            message: "Connection successful".to_string(),
            region: Some("us-west-2".to_string()),
            bucket_name: Some("my-bucket".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ConnectionTestResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.connected, deserialized.connected);
        assert_eq!(response.message, deserialized.message);
        assert_eq!(response.region, deserialized.region);
        assert_eq!(response.bucket_name, deserialized.bucket_name);
    }

    #[test]
    fn test_command_error_serialization() {
        let error = CommandError::NotConfigured("test error".to_string());
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Not configured"));
        assert!(json.contains("test error"));
    }
}
