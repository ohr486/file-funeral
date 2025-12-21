//! Tauri commands module
//!
//! This module provides the bridge between the frontend (React) and backend (Rust).
//! All functions here are exposed to the frontend via the Tauri command system.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::auth::{AwsCredentials, CredentialError, CredentialManager};
use crate::storage::{s3::S3Provider, CloudStorageProvider, FileInfo, FileMetadata, StorageError};
use crate::sync::{self, ComparisonResult, SyncError, SyncState};

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
pub async fn set_credentials(request: SetCredentialsRequest) -> CommandResult<CredentialsResponse> {
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
        CommandError::NotConfigured(
            "AWS credentials not found. Please set credentials first.".to_string(),
        )
    })?;

    log::info!("Credentials loaded successfully");

    // Test actual connection to S3
    log::info!(
        "Testing connection to S3 bucket: {}",
        credentials.bucket_name
    );

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
    match client
        .head_bucket()
        .bucket(&credentials.bucket_name)
        .send()
        .await
    {
        Ok(_) => {
            log::info!(
                "Successfully connected to S3 bucket: {}",
                credentials.bucket_name
            );
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

// ============================================================================
// Local File System Helper Functions
// ============================================================================

/// List all files in a local directory recursively
///
/// # Arguments
/// * `dir_path` - Path to the directory to scan
///
/// # Returns
/// A vector of FileInfo for all files found in the directory
fn list_local_files(dir_path: &Path) -> CommandResult<Vec<FileInfo>> {
    let mut files = Vec::new();

    if !dir_path.exists() {
        log::warn!("Directory does not exist: {:?}", dir_path);
        return Ok(files);
    }

    if !dir_path.is_dir() {
        return Err(CommandError::InvalidInput(format!(
            "Path is not a directory: {:?}",
            dir_path
        )));
    }

    // Recursively walk the directory
    walk_directory(dir_path, dir_path, &mut files)?;

    log::info!("Found {} local files in {:?}", files.len(), dir_path);
    Ok(files)
}

/// Recursively walk a directory and collect file information
fn walk_directory(
    base_path: &Path,
    current_path: &Path,
    files: &mut Vec<FileInfo>,
) -> CommandResult<()> {
    let entries = fs::read_dir(current_path).map_err(|e| {
        CommandError::OperationFailed(format!(
            "Failed to read directory {:?}: {}",
            current_path, e
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            CommandError::OperationFailed(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Skip hidden files and system files
        if should_skip_file(&file_name_str) {
            log::debug!("Skipping file: {:?}", file_name_str);
            continue;
        }

        let metadata = entry.metadata().map_err(|e| {
            CommandError::OperationFailed(format!("Failed to read metadata for {:?}: {}", path, e))
        })?;

        if metadata.is_file() {
            // Get relative path from base
            let relative_path = path.strip_prefix(base_path).map_err(|e| {
                CommandError::OperationFailed(format!("Failed to get relative path: {}", e))
            })?;

            let path_str = relative_path.to_string_lossy().to_string();

            // Convert modified time to DateTime<Utc>
            let modified = metadata.modified().map_err(|e| {
                CommandError::OperationFailed(format!("Failed to get modified time: {}", e))
            })?;

            let modified_utc = chrono::DateTime::<chrono::Utc>::from(modified);

            let file_info = FileInfo::new(
                path_str,
                metadata.len(),
                modified_utc,
                None, // ETag is not available for local files
            );

            files.push(file_info);
        } else if metadata.is_dir() {
            // Recursively process subdirectory
            walk_directory(base_path, &path, files)?;
        }
        // Skip symbolic links and other special files
    }

    Ok(())
}

/// Check if a file should be skipped during sync
fn should_skip_file(file_name: &str) -> bool {
    // Default exclusion patterns
    let skip_patterns = [".DS_Store", "Thumbs.db", ".tmp", ".swp", "~"];

    // Skip hidden files (starting with .)
    if file_name.starts_with('.') {
        return true;
    }

    // Skip files matching patterns
    for pattern in &skip_patterns {
        if file_name.ends_with(pattern) {
            return true;
        }
    }

    false
}

/// Read a local file's contents
fn read_local_file(file_path: &Path) -> CommandResult<Vec<u8>> {
    fs::read(file_path).map_err(|e| {
        CommandError::OperationFailed(format!("Failed to read file {:?}: {}", file_path, e))
    })
}

/// Write data to a local file
fn write_local_file(file_path: &Path, data: &[u8]) -> CommandResult<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            CommandError::OperationFailed(format!(
                "Failed to create directories {:?}: {}",
                parent, e
            ))
        })?;
    }

    fs::write(file_path, data).map_err(|e| {
        CommandError::OperationFailed(format!("Failed to write file {:?}: {}", file_path, e))
    })
}

/// Create an S3Provider instance from stored credentials
async fn create_s3_provider() -> CommandResult<S3Provider> {
    let manager = CredentialManager::new();
    let credentials = manager.load_aws_credentials().map_err(|_| {
        CommandError::NotConfigured(
            "AWS credentials not found. Please set credentials first.".to_string(),
        )
    })?;

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

    Ok(S3Provider::with_config(
        credentials.bucket_name,
        &sdk_config,
    ))
}

// ============================================================================
// Tauri Commands
// ============================================================================

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
pub async fn list_files(request: ListFilesRequest) -> CommandResult<ListFilesResponse> {
    log::info!("list_files called with prefix: {}", request.prefix);

    // Create S3Provider
    let provider = create_s3_provider().await?;

    // List files with the given prefix
    let files = provider.list(&request.prefix).await?;

    log::info!(
        "Found {} remote files with prefix '{}'",
        files.len(),
        request.prefix
    );

    Ok(ListFilesResponse {
        total_count: files.len(),
        files,
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
    remote_prefix: String,
) -> CommandResult<SyncStatusResponse> {
    log::info!(
        "get_sync_status called: local={}, remote={}",
        local_path,
        remote_prefix
    );

    // Validate input
    if local_path.is_empty() {
        return Err(CommandError::InvalidInput(
            "Local path cannot be empty".to_string(),
        ));
    }

    let local_dir = PathBuf::from(&local_path);

    // 1. List local files
    let local_files = list_local_files(&local_dir)?;
    log::info!("Found {} local files", local_files.len());

    // 2. List remote files
    let provider = create_s3_provider().await?;
    let remote_files = provider.list(&remote_prefix).await?;
    log::info!("Found {} remote files", remote_files.len());

    // 3. Build a map of all unique file paths
    let mut all_paths = HashSet::new();
    for file in &local_files {
        all_paths.insert(file.path.clone());
    }
    for file in &remote_files {
        all_paths.insert(file.path.clone());
    }

    log::info!("Comparing {} unique file paths", all_paths.len());

    // 4. Compare each file
    let mut comparisons = Vec::new();
    let mut in_sync_count = 0;
    let mut needs_upload_count = 0;
    let mut needs_download_count = 0;
    let mut conflict_count = 0;

    for path in all_paths {
        let local_info = local_files.iter().find(|f| f.path == path);
        let remote_info = remote_files.iter().find(|f| f.path == path);

        // For v1.0, we don't track last_sync_time, so pass None
        let comparison = sync::compare_files(local_info, remote_info, None);

        // Count by state
        match comparison.state {
            SyncState::InSync => in_sync_count += 1,
            SyncState::NeedsUpload => needs_upload_count += 1,
            SyncState::NeedsDownload => needs_download_count += 1,
            SyncState::Conflict => conflict_count += 1,
        }

        comparisons.push(comparison.into());
    }

    log::info!(
        "Sync status: in_sync={}, upload={}, download={}, conflict={}",
        in_sync_count,
        needs_upload_count,
        needs_download_count,
        conflict_count
    );

    Ok(SyncStatusResponse {
        comparisons,
        in_sync_count,
        needs_upload_count,
        needs_download_count,
        conflict_count,
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
    log::info!(
        "sync_files called: local={}, remote={}",
        request.local_path,
        request.remote_prefix
    );

    // Validate input
    if request.local_path.is_empty() {
        return Err(CommandError::InvalidInput(
            "Local path cannot be empty".to_string(),
        ));
    }

    let local_dir = PathBuf::from(&request.local_path);

    // 1. Get sync status (compare files)
    let status = get_sync_status(request.local_path.clone(), request.remote_prefix.clone()).await?;
    log::info!(
        "Sync analysis: {} comparisons, {} upload, {} download, {} conflicts",
        status.comparisons.len(),
        status.needs_upload_count,
        status.needs_download_count,
        status.conflict_count
    );

    // 2. Create S3Provider
    let provider = create_s3_provider().await?;

    // 3. Sync each file based on its state
    let mut files_uploaded = 0;
    let mut files_downloaded = 0;
    let mut conflicts_resolved = 0;
    let mut errors = Vec::new();

    for comparison in status.comparisons {
        let result =
            sync_single_file(&comparison, &local_dir, &request.remote_prefix, &provider).await;

        match result {
            Ok(action) => {
                match action {
                    SyncAction::Uploaded => files_uploaded += 1,
                    SyncAction::Downloaded => files_downloaded += 1,
                    SyncAction::ConflictResolved => {
                        conflicts_resolved += 1;
                        // Conflict resolution involves both upload and download
                        files_uploaded += 1;
                        files_downloaded += 1;
                    }
                    SyncAction::Skipped => {}
                }
            }
            Err(e) => {
                log::error!("Failed to sync file {}: {}", comparison.path, e);
                errors.push(format!("{}: {}", comparison.path, e));
            }
        }
    }

    let message = if errors.is_empty() {
        format!(
            "Sync completed successfully. Uploaded: {}, Downloaded: {}, Conflicts: {}",
            files_uploaded, files_downloaded, conflicts_resolved
        )
    } else {
        format!(
            "Sync completed with {} errors. Uploaded: {}, Downloaded: {}, Conflicts: {}. Errors: {}",
            errors.len(),
            files_uploaded,
            files_downloaded,
            conflicts_resolved,
            errors.join("; ")
        )
    };

    log::info!("{}", message);

    Ok(SyncFilesResponse {
        success: errors.is_empty(),
        files_uploaded,
        files_downloaded,
        conflicts_resolved,
        message,
    })
}

/// Action taken during sync
enum SyncAction {
    Uploaded,
    Downloaded,
    ConflictResolved,
    Skipped,
}

/// Sync a single file based on its comparison result
async fn sync_single_file(
    comparison: &ComparisonResultDto,
    local_dir: &Path,
    remote_prefix: &str,
    provider: &S3Provider,
) -> CommandResult<SyncAction> {
    let local_path = local_dir.join(&comparison.path);
    let remote_path = if remote_prefix.is_empty() {
        comparison.path.clone()
    } else {
        format!(
            "{}/{}",
            remote_prefix.trim_end_matches('/'),
            comparison.path
        )
    };

    match comparison.state {
        SyncState::InSync => {
            log::debug!("File already in sync: {}", comparison.path);
            Ok(SyncAction::Skipped)
        }

        SyncState::NeedsUpload => {
            log::info!("Uploading file: {} -> {}", comparison.path, remote_path);

            // Read local file
            let data = read_local_file(&local_path)?;

            // Create metadata
            let metadata = FileMetadata::new(
                data.len() as u64,
                chrono::Utc::now(),
                None, // Content type detection could be added later
                None, // ETag will be generated by S3
            );

            // Upload to S3
            provider.upload(&remote_path, &data, metadata).await?;

            log::info!("Successfully uploaded: {}", comparison.path);
            Ok(SyncAction::Uploaded)
        }

        SyncState::NeedsDownload => {
            log::info!("Downloading file: {} <- {}", comparison.path, remote_path);

            // Download from S3
            let data = provider.download(&remote_path).await?;

            // Write to local file
            write_local_file(&local_path, &data)?;

            log::info!("Successfully downloaded: {}", comparison.path);
            Ok(SyncAction::Downloaded)
        }

        SyncState::Conflict => {
            log::warn!("Conflict detected for file: {}", comparison.path);

            // Resolve conflict using the "both-save" approach
            let resolution = sync::resolve_conflict(&comparison.path)?;

            log::info!(
                "Resolving conflict: original={}, conflicted={}",
                resolution.original_path,
                resolution.conflicted_copy_path
            );

            // Step 1: Read current local version BEFORE overwriting it
            let local_data = read_local_file(&local_path)?;

            // Step 2: Download remote version and save to original path
            let remote_data = provider.download(&remote_path).await?;
            write_local_file(&local_path, &remote_data)?;

            // Step 3: Save local version to conflicted copy path
            let conflicted_path = local_dir.join(&resolution.conflicted_copy_path);
            write_local_file(&conflicted_path, &local_data)?;

            // Step 4: Upload the conflicted copy to S3 as well
            let conflicted_remote_path = if remote_prefix.is_empty() {
                resolution.conflicted_copy_path.clone()
            } else {
                format!(
                    "{}/{}",
                    remote_prefix.trim_end_matches('/'),
                    resolution.conflicted_copy_path
                )
            };

            let conflicted_metadata =
                FileMetadata::new(local_data.len() as u64, chrono::Utc::now(), None, None);

            provider
                .upload(&conflicted_remote_path, &local_data, conflicted_metadata)
                .await?;

            log::info!("Successfully resolved conflict for: {}", comparison.path);
            Ok(SyncAction::ConflictResolved)
        }
    }
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
        let file_info = FileInfo::new(
            "test.txt".to_string(),
            100,
            now,
            Some("etag123".to_string()),
        );

        let comparison = ComparisonResult {
            path: "test.txt".to_string(),
            state: SyncState::NeedsUpload,
            local_info: Some(file_info),
            remote_info: None,
        };

        let dto: ComparisonResultDto = comparison.into();
        assert_eq!(dto.path, "test.txt");
        assert_eq!(dto.state, SyncState::NeedsUpload);
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
            ("access", "", "region", "bucket", "Secret access key"), // Now fails because no existing credentials
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

    // ============================================================================
    // Tests for Local File System Helper Functions (Phase 3.1)
    // ============================================================================

    #[test]
    fn test_should_skip_file() {
        // Hidden files (starting with .)
        assert!(should_skip_file(".hidden"));
        assert!(should_skip_file(".git"));
        assert!(should_skip_file(".DS_Store"));

        // System files
        assert!(should_skip_file("Thumbs.db"));
        assert!(should_skip_file("file.tmp"));
        assert!(should_skip_file("file.swp"));
        assert!(should_skip_file("file~"));

        // Normal files should not be skipped
        assert!(!should_skip_file("normal.txt"));
        assert!(!should_skip_file("document.pdf"));
        assert!(!should_skip_file("README.md"));
    }

    #[test]
    fn test_read_write_local_file() {
        use tempfile::TempDir;

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Test data
        let test_data = b"Hello, World!";

        // Write file
        let write_result = write_local_file(&file_path, test_data);
        assert!(write_result.is_ok());

        // Read file
        let read_result = read_local_file(&file_path);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_data);

        // Cleanup is automatic with TempDir
    }

    #[test]
    fn test_write_local_file_creates_directories() {
        use tempfile::TempDir;

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir
            .path()
            .join("subdir1")
            .join("subdir2")
            .join("test.txt");

        // Test data
        let test_data = b"Test content";

        // Write file (should create parent directories)
        let write_result = write_local_file(&file_path, test_data);
        assert!(write_result.is_ok());

        // Verify file exists
        assert!(file_path.exists());

        // Verify content
        let read_result = read_local_file(&file_path);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_data);
    }

    #[test]
    fn test_read_local_file_nonexistent() {
        let file_path = PathBuf::from("/nonexistent/path/to/file.txt");
        let result = read_local_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_local_files_empty_directory() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let result = list_local_files(temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_list_local_files_with_files() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create some test files
        fs::write(temp_dir.path().join("file1.txt"), b"content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), b"content2").unwrap();

        // Create a subdirectory with a file
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file3.txt"), b"content3").unwrap();

        let result = list_local_files(temp_dir.path());
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), 3);

        // Check that all files are found
        let paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
        assert!(paths.contains(&"file1.txt".to_string()));
        assert!(paths.contains(&"file2.txt".to_string()));
        assert!(
            paths.contains(&"subdir/file3.txt".to_string())
                || paths.contains(&"subdir\\file3.txt".to_string())
        ); // Windows compatibility
    }

    #[test]
    fn test_list_local_files_skips_hidden_files() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create normal and hidden files
        fs::write(temp_dir.path().join("normal.txt"), b"content").unwrap();
        fs::write(temp_dir.path().join(".hidden"), b"hidden").unwrap();
        fs::write(temp_dir.path().join(".DS_Store"), b"ds").unwrap();

        let result = list_local_files(temp_dir.path());
        assert!(result.is_ok());

        let files = result.unwrap();
        // Should only find the normal file
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "normal.txt");
    }

    #[test]
    fn test_list_local_files_nonexistent_directory() {
        let path = PathBuf::from("/nonexistent/directory");
        let result = list_local_files(&path);

        // Should return empty list for nonexistent directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_list_local_files_not_a_directory() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, b"content").unwrap();

        let result = list_local_files(&file_path);
        assert!(result.is_err());

        match result {
            Err(CommandError::InvalidInput(msg)) => {
                assert!(msg.contains("not a directory"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_sync_action_enum() {
        // Just verify the enum variants exist and can be created
        let _uploaded = SyncAction::Uploaded;
        let _downloaded = SyncAction::Downloaded;
        let _conflict = SyncAction::ConflictResolved;
        let _skipped = SyncAction::Skipped;
    }

    // ============================================================================
    // Integration Tests for Commands (Phase 3.1)
    // ============================================================================

    #[tokio::test]
    async fn test_list_files_without_credentials() {
        // Clean up any existing credentials
        let manager = CredentialManager::new();
        let _ = manager.delete_aws_credentials();

        let request = ListFilesRequest {
            prefix: "test/".to_string(),
        };

        let result = list_files(request).await;

        // Should fail with NotConfigured error
        assert!(result.is_err());
        match result {
            Err(CommandError::NotConfigured(_)) => {
                // Expected
            }
            _ => panic!("Expected NotConfigured error"),
        }

        // Cleanup
        let _ = manager.delete_aws_credentials();
    }

    #[tokio::test]
    async fn test_get_sync_status_empty_path() {
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
    async fn test_get_sync_status_without_credentials() {
        use tempfile::TempDir;

        // Clean up credentials
        let manager = CredentialManager::new();
        let _ = manager.delete_aws_credentials();

        let temp_dir = TempDir::new().unwrap();
        let local_path = temp_dir.path().to_string_lossy().to_string();

        let result = get_sync_status(local_path, "remote/".to_string()).await;

        // Should fail with NotConfigured error
        assert!(result.is_err());
        match result {
            Err(CommandError::NotConfigured(_)) => {
                // Expected
            }
            _ => panic!("Expected NotConfigured error"),
        }

        // Cleanup
        let _ = manager.delete_aws_credentials();
    }

    #[tokio::test]
    async fn test_sync_files_empty_path() {
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

    #[tokio::test]
    async fn test_sync_files_without_credentials() {
        use tempfile::TempDir;

        // Clean up credentials
        let manager = CredentialManager::new();
        let _ = manager.delete_aws_credentials();

        let temp_dir = TempDir::new().unwrap();
        let request = SyncFilesRequest {
            local_path: temp_dir.path().to_string_lossy().to_string(),
            remote_prefix: "remote/".to_string(),
        };

        let result = sync_files(request).await;

        // Should fail with NotConfigured error
        assert!(result.is_err());
        match result {
            Err(CommandError::NotConfigured(_)) => {
                // Expected
            }
            _ => panic!("Expected NotConfigured error"),
        }

        // Cleanup
        let _ = manager.delete_aws_credentials();
    }
}
