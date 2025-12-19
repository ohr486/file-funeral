//! Synchronization engine module
//!
//! This module handles file synchronization logic between local storage and cloud providers.
//! It provides:
//! - File comparison to determine sync state
//! - Conflict detection when both local and remote files are modified
//! - Conflict resolution using the "both-save" approach (Dropbox-style)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

use crate::storage::FileInfo;

/// Errors that can occur during synchronization
#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    #[error("Failed to get hostname")]
    HostnameError,

    #[error("Conflict resolution failed: {0}")]
    ConflictResolutionError(String),
}

/// Synchronization state of a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncState {
    /// File is synchronized (local and remote are identical)
    InSync,
    /// Local file is newer than remote (upload needed)
    LocalNewer,
    /// Remote file is newer than local (download needed)
    RemoteNewer,
    /// Both files have been modified (conflict)
    Conflict,
    /// File exists only locally (upload needed)
    LocalOnly,
    /// File exists only remotely (download needed)
    RemoteOnly,
}

/// Result of comparing local and remote files
#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonResult {
    /// Path of the file
    pub path: String,
    /// Sync state determined by comparison
    pub state: SyncState,
    /// Local file info (if exists)
    pub local_info: Option<FileInfo>,
    /// Remote file info (if exists)
    pub remote_info: Option<FileInfo>,
}

/// Conflict resolution result
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictResolution {
    /// Original file path (will keep remote version)
    pub original_path: String,
    /// Conflicted copy path (will save local version with new name)
    pub conflicted_copy_path: String,
}

/// Compare local and remote file information to determine sync state
///
/// # Arguments
/// * `local_info` - Local file information (None if file doesn't exist locally)
/// * `remote_info` - Remote file information (None if file doesn't exist remotely)
/// * `last_sync_time` - Last time this file was synchronized (None if never synced)
///
/// # Returns
/// ComparisonResult indicating the sync state and file information
pub fn compare_files(
    local_info: Option<&FileInfo>,
    remote_info: Option<&FileInfo>,
    last_sync_time: Option<DateTime<Utc>>,
) -> ComparisonResult {
    let path = local_info
        .or(remote_info)
        .map(|info| info.path.clone())
        .unwrap_or_default();

    let state = match (local_info, remote_info, last_sync_time) {
        // Both exist - need to compare
        (Some(local), Some(remote), Some(last_sync)) => {
            let local_modified_after_sync = local.last_modified > last_sync;
            let remote_modified_after_sync = remote.last_modified > last_sync;

            if local_modified_after_sync && remote_modified_after_sync {
                // Both modified since last sync - conflict
                SyncState::Conflict
            } else if local_modified_after_sync {
                // Only local modified
                SyncState::LocalNewer
            } else if remote_modified_after_sync {
                // Only remote modified
                SyncState::RemoteNewer
            } else if files_are_identical(local, remote) {
                // Neither modified, files identical
                SyncState::InSync
            } else {
                // Files differ but neither modified since sync - treat as conflict
                SyncState::Conflict
            }
        }
        // Both exist but never synced before
        (Some(local), Some(remote), None) => {
            if files_are_identical(local, remote) {
                SyncState::InSync
            } else {
                // Different files, never synced - conflict
                SyncState::Conflict
            }
        }
        // Only local exists
        (Some(_), None, _) => SyncState::LocalOnly,
        // Only remote exists
        (None, Some(_), _) => SyncState::RemoteOnly,
        // Neither exists (shouldn't happen in practice)
        (None, None, _) => SyncState::InSync,
    };

    ComparisonResult {
        path,
        state,
        local_info: local_info.cloned(),
        remote_info: remote_info.cloned(),
    }
}

/// Check if two files are identical based on metadata
///
/// Files are considered identical if they have the same size and either:
/// - Same ETag, or
/// - Same last modified time (within 1 second tolerance for filesystem precision)
fn files_are_identical(local: &FileInfo, remote: &FileInfo) -> bool {
    // Size must match
    if local.size != remote.size {
        return false;
    }

    // If both have ETags, compare them
    if let (Some(local_etag), Some(remote_etag)) = (&local.etag, &remote.etag) {
        return local_etag == remote_etag;
    }

    // Fall back to timestamp comparison (with 1 second tolerance)
    let time_diff = if local.last_modified > remote.last_modified {
        local.last_modified - remote.last_modified
    } else {
        remote.last_modified - local.last_modified
    };

    time_diff.num_seconds() <= 1
}

/// Generate a conflict resolution path for a local file
///
/// Creates a path like: "filename (PCName's conflicted copy YYYY-MM-DD).txt"
///
/// # Arguments
/// * `original_path` - The original file path
///
/// # Returns
/// ConflictResolution with paths for both the original and conflicted copy
pub fn resolve_conflict(original_path: &str) -> Result<ConflictResolution, SyncError> {
    let path = Path::new(original_path);

    // Get filename components
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| SyncError::InvalidPath(original_path.to_string()))?;

    let extension = path.extension().and_then(|s| s.to_str());

    // Get parent directory
    let parent = path
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    // Get hostname (PC name)
    let hostname = get_hostname()?;

    // Get current date
    let today = Utc::now().format("%Y-%m-%d").to_string();

    // Build conflicted filename
    let conflicted_filename = if let Some(ext) = extension {
        format!(
            "{} ({}'s conflicted copy {}).{}",
            file_stem, hostname, today, ext
        )
    } else {
        format!(
            "{} ({}'s conflicted copy {})",
            file_stem, hostname, today
        )
    };

    // Build full path
    let conflicted_path = if parent.is_empty() {
        conflicted_filename
    } else {
        format!("{}/{}", parent, conflicted_filename)
    };

    Ok(ConflictResolution {
        original_path: original_path.to_string(),
        conflicted_copy_path: conflicted_path,
    })
}

/// Get the hostname of the current machine
fn get_hostname() -> Result<String, SyncError> {
    hostname::get()
        .map_err(|_| SyncError::HostnameError)?
        .into_string()
        .map_err(|_| SyncError::HostnameError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_file_info(path: &str, size: u64, modified: DateTime<Utc>, etag: Option<&str>) -> FileInfo {
        FileInfo {
            path: path.to_string(),
            size,
            last_modified: modified,
            etag: etag.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_sync_state_variants() {
        // Just verify all variants can be created
        let states = vec![
            SyncState::InSync,
            SyncState::LocalNewer,
            SyncState::RemoteNewer,
            SyncState::Conflict,
            SyncState::LocalOnly,
            SyncState::RemoteOnly,
        ];
        assert_eq!(states.len(), 6);
    }

    #[test]
    fn test_files_identical_same_etag() {
        let now = Utc::now();
        let file1 = create_file_info("test.txt", 100, now, Some("abc123"));
        let file2 = create_file_info("test.txt", 100, now, Some("abc123"));

        assert!(files_are_identical(&file1, &file2));
    }

    #[test]
    fn test_files_identical_different_etag() {
        let now = Utc::now();
        let file1 = create_file_info("test.txt", 100, now, Some("abc123"));
        let file2 = create_file_info("test.txt", 100, now, Some("def456"));

        assert!(!files_are_identical(&file1, &file2));
    }

    #[test]
    fn test_files_identical_different_size() {
        let now = Utc::now();
        let file1 = create_file_info("test.txt", 100, now, Some("abc123"));
        let file2 = create_file_info("test.txt", 200, now, Some("abc123"));

        assert!(!files_are_identical(&file1, &file2));
    }

    #[test]
    fn test_files_identical_no_etag_same_time() {
        let now = Utc::now();
        let file1 = create_file_info("test.txt", 100, now, None);
        let file2 = create_file_info("test.txt", 100, now, None);

        assert!(files_are_identical(&file1, &file2));
    }

    #[test]
    fn test_files_identical_no_etag_within_tolerance() {
        let now = Utc::now();
        let file1 = create_file_info("test.txt", 100, now, None);
        let file2 = create_file_info("test.txt", 100, now + Duration::milliseconds(500), None);

        assert!(files_are_identical(&file1, &file2));
    }

    #[test]
    fn test_files_not_identical_no_etag_different_time() {
        let now = Utc::now();
        let file1 = create_file_info("test.txt", 100, now, None);
        let file2 = create_file_info("test.txt", 100, now + Duration::seconds(5), None);

        assert!(!files_are_identical(&file1, &file2));
    }

    #[test]
    fn test_compare_files_both_in_sync() {
        let now = Utc::now();
        let last_sync = now - Duration::hours(1);

        let local = create_file_info("test.txt", 100, last_sync - Duration::minutes(30), Some("abc123"));
        let remote = create_file_info("test.txt", 100, last_sync - Duration::minutes(30), Some("abc123"));

        let result = compare_files(Some(&local), Some(&remote), Some(last_sync));

        assert_eq!(result.state, SyncState::InSync);
        assert_eq!(result.path, "test.txt");
    }

    #[test]
    fn test_compare_files_local_newer() {
        let now = Utc::now();
        let last_sync = now - Duration::hours(1);

        let local = create_file_info("test.txt", 100, now, Some("abc123"));
        let remote = create_file_info("test.txt", 100, last_sync - Duration::minutes(30), Some("def456"));

        let result = compare_files(Some(&local), Some(&remote), Some(last_sync));

        assert_eq!(result.state, SyncState::LocalNewer);
    }

    #[test]
    fn test_compare_files_remote_newer() {
        let now = Utc::now();
        let last_sync = now - Duration::hours(1);

        let local = create_file_info("test.txt", 100, last_sync - Duration::minutes(30), Some("abc123"));
        let remote = create_file_info("test.txt", 100, now, Some("def456"));

        let result = compare_files(Some(&local), Some(&remote), Some(last_sync));

        assert_eq!(result.state, SyncState::RemoteNewer);
    }

    #[test]
    fn test_compare_files_conflict() {
        let now = Utc::now();
        let last_sync = now - Duration::hours(1);

        let local = create_file_info("test.txt", 100, now - Duration::minutes(10), Some("abc123"));
        let remote = create_file_info("test.txt", 200, now - Duration::minutes(5), Some("def456"));

        let result = compare_files(Some(&local), Some(&remote), Some(last_sync));

        assert_eq!(result.state, SyncState::Conflict);
    }

    #[test]
    fn test_compare_files_local_only() {
        let now = Utc::now();
        let local = create_file_info("test.txt", 100, now, Some("abc123"));

        let result = compare_files(Some(&local), None, None);

        assert_eq!(result.state, SyncState::LocalOnly);
    }

    #[test]
    fn test_compare_files_remote_only() {
        let now = Utc::now();
        let remote = create_file_info("test.txt", 100, now, Some("abc123"));

        let result = compare_files(None, Some(&remote), None);

        assert_eq!(result.state, SyncState::RemoteOnly);
    }

    #[test]
    fn test_compare_files_never_synced_identical() {
        let now = Utc::now();
        let local = create_file_info("test.txt", 100, now, Some("abc123"));
        let remote = create_file_info("test.txt", 100, now, Some("abc123"));

        let result = compare_files(Some(&local), Some(&remote), None);

        assert_eq!(result.state, SyncState::InSync);
    }

    #[test]
    fn test_compare_files_never_synced_different() {
        let now = Utc::now();
        let local = create_file_info("test.txt", 100, now, Some("abc123"));
        let remote = create_file_info("test.txt", 200, now, Some("def456"));

        let result = compare_files(Some(&local), Some(&remote), None);

        assert_eq!(result.state, SyncState::Conflict);
    }

    #[test]
    fn test_resolve_conflict_with_extension() {
        let result = resolve_conflict("documents/report.txt").expect("Failed to resolve conflict");

        assert_eq!(result.original_path, "documents/report.txt");
        assert!(result.conflicted_copy_path.starts_with("documents/report ("));
        assert!(result.conflicted_copy_path.contains("'s conflicted copy "));
        assert!(result.conflicted_copy_path.ends_with(").txt"));
    }

    #[test]
    fn test_resolve_conflict_without_extension() {
        let result = resolve_conflict("documents/README").expect("Failed to resolve conflict");

        assert_eq!(result.original_path, "documents/README");
        assert!(result.conflicted_copy_path.starts_with("documents/README ("));
        assert!(result.conflicted_copy_path.contains("'s conflicted copy "));
        assert!(!result.conflicted_copy_path.contains("."));
    }

    #[test]
    fn test_resolve_conflict_no_parent() {
        let result = resolve_conflict("file.txt").expect("Failed to resolve conflict");

        assert_eq!(result.original_path, "file.txt");
        assert!(result.conflicted_copy_path.starts_with("file ("));
        assert!(result.conflicted_copy_path.ends_with(").txt"));
    }

    #[test]
    fn test_resolve_conflict_multiple_extensions() {
        let result = resolve_conflict("backup.tar.gz").expect("Failed to resolve conflict");

        assert_eq!(result.original_path, "backup.tar.gz");
        // Should preserve only the last extension
        assert!(result.conflicted_copy_path.ends_with(").gz"));
    }

    #[test]
    fn test_get_hostname() {
        let hostname = get_hostname().expect("Failed to get hostname");
        assert!(!hostname.is_empty());
    }

    #[test]
    fn test_comparison_result_cloning() {
        let now = Utc::now();
        let local = create_file_info("test.txt", 100, now, Some("abc123"));
        let result = ComparisonResult {
            path: "test.txt".to_string(),
            state: SyncState::LocalOnly,
            local_info: Some(local),
            remote_info: None,
        };

        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    #[test]
    fn test_conflict_resolution_equality() {
        let res1 = ConflictResolution {
            original_path: "test.txt".to_string(),
            conflicted_copy_path: "test (conflict).txt".to_string(),
        };
        let res2 = ConflictResolution {
            original_path: "test.txt".to_string(),
            conflicted_copy_path: "test (conflict).txt".to_string(),
        };

        assert_eq!(res1, res2);
    }
}
