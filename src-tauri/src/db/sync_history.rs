//! Sync history management
//!
//! This module provides APIs for saving and loading sync history.
//! It tracks when syncs occurred and which files were synced.

use chrono::{DateTime, Utc};

use super::{DbError, DbPool};

/// Sync history record
#[derive(Debug, Clone)]
pub struct SyncHistory {
    pub id: Option<i64>,
    pub sync_started_at: DateTime<Utc>,
    pub sync_completed_at: DateTime<Utc>,
    pub local_path: String,
    pub remote_prefix: String,
    pub files_uploaded: i32,
    pub files_downloaded: i32,
    pub files_deleted: i32,
    pub conflicts_resolved: i32,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Synced file record
#[derive(Debug, Clone)]
pub struct SyncedFile {
    pub file_path: String,
    pub file_size: u64,
    pub last_modified: DateTime<Utc>,
    pub etag: Option<String>,
    pub was_local: bool,
    pub was_remote: bool,
}

/// Save sync history and synced files to the database
///
/// This function saves a sync history record and the associated file list.
/// It's called after a successful sync operation.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `history` - Sync history record to save
/// * `files` - List of files that were synced
///
/// # Returns
/// * `Ok(i64)` - The ID of the inserted sync_history record
/// * `Err(DbError)` - If the save operation fails
pub fn save_sync_history(
    pool: &DbPool,
    history: &SyncHistory,
    files: &[SyncedFile],
) -> Result<i64, DbError> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    // Insert sync_history record
    tx.execute(
        "INSERT INTO sync_history
        (sync_started_at, sync_completed_at, local_path, remote_prefix,
         files_uploaded, files_downloaded, files_deleted, conflicts_resolved,
         success, error_message)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            &history.sync_started_at.to_rfc3339(),
            &history.sync_completed_at.to_rfc3339(),
            &history.local_path,
            &history.remote_prefix,
            &history.files_uploaded,
            &history.files_downloaded,
            &history.files_deleted,
            &history.conflicts_resolved,
            if history.success { 1 } else { 0 },
            &history.error_message,
        ),
    )?;

    let sync_history_id = tx.last_insert_rowid();

    // Insert synced_files records
    for file in files {
        tx.execute(
            "INSERT INTO synced_files
            (sync_history_id, file_path, file_size, last_modified, etag, was_local, was_remote)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                sync_history_id,
                &file.file_path,
                file.file_size as i64,
                &file.last_modified.to_rfc3339(),
                &file.etag,
                if file.was_local { 1 } else { 0 },
                if file.was_remote { 1 } else { 0 },
            ),
        )?;
    }

    tx.commit()?;

    log::info!(
        "Saved sync history: id={}, files={}",
        sync_history_id,
        files.len()
    );

    Ok(sync_history_id)
}

/// Get the last sync time for a given local path and remote prefix
///
/// This function retrieves the completion time of the most recent successful sync.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `local_path` - Local directory path
/// * `remote_prefix` - Remote prefix (S3 key prefix)
///
/// # Returns
/// * `Ok(Some(DateTime<Utc>))` - The last sync completion time
/// * `Ok(None)` - If no sync history exists
/// * `Err(DbError)` - If the query fails
pub fn get_last_sync_time(
    pool: &DbPool,
    local_path: &str,
    remote_prefix: &str,
) -> Result<Option<DateTime<Utc>>, DbError> {
    let conn = pool.get()?;

    let result = conn.query_row(
        "SELECT sync_completed_at FROM sync_history
         WHERE local_path = ?1 AND remote_prefix = ?2 AND success = 1
         ORDER BY sync_completed_at DESC LIMIT 1",
        (local_path, remote_prefix),
        |row| {
            let timestamp: String = row.get(0)?;
            Ok(timestamp)
        },
    );

    match result {
        Ok(timestamp) => {
            let dt = DateTime::parse_from_rfc3339(&timestamp)
                .map_err(|e| DbError::InvalidState(format!("Invalid timestamp format: {}", e)))?
                .with_timezone(&Utc);
            Ok(Some(dt))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DbError::Sqlite(e)),
    }
}

/// Get the list of files from the last successful sync
///
/// This function retrieves the file paths that existed during the most recent sync.
/// This is used for deletion detection.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `local_path` - Local directory path
/// * `remote_prefix` - Remote prefix (S3 key prefix)
///
/// # Returns
/// * `Ok(Vec<String>)` - List of file paths from the last sync
/// * `Err(DbError)` - If the query fails
pub fn get_last_synced_files(
    pool: &DbPool,
    local_path: &str,
    remote_prefix: &str,
) -> Result<Vec<String>, DbError> {
    let conn = pool.get()?;

    // Get the most recent successful sync ID
    let sync_id_result = conn.query_row(
        "SELECT id FROM sync_history
         WHERE local_path = ?1 AND remote_prefix = ?2 AND success = 1
         ORDER BY sync_completed_at DESC LIMIT 1",
        (local_path, remote_prefix),
        |row| row.get::<_, i64>(0),
    );

    let sync_id = match sync_id_result {
        Ok(id) => id,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(Vec::new()),
        Err(e) => return Err(DbError::Sqlite(e)),
    };

    // Get all file paths from that sync
    let mut stmt = conn.prepare(
        "SELECT file_path FROM synced_files
         WHERE sync_history_id = ?1
         ORDER BY file_path",
    )?;

    let files = stmt
        .query_map([sync_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    log::info!(
        "Retrieved {} files from last sync (id={})",
        files.len(),
        sync_id
    );

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use serial_test::serial;

    const TEST_SERVICE: &str = "file-funeral-test-sync-history";

    fn create_test_history() -> SyncHistory {
        SyncHistory {
            id: None,
            sync_started_at: Utc::now(),
            sync_completed_at: Utc::now(),
            local_path: "/test/local".to_string(),
            remote_prefix: "test/remote".to_string(),
            files_uploaded: 2,
            files_downloaded: 1,
            files_deleted: 0,
            conflicts_resolved: 0,
            success: true,
            error_message: None,
        }
    }

    fn create_test_files() -> Vec<SyncedFile> {
        vec![
            SyncedFile {
                file_path: "file1.txt".to_string(),
                file_size: 100,
                last_modified: Utc::now(),
                etag: Some("etag1".to_string()),
                was_local: true,
                was_remote: true,
            },
            SyncedFile {
                file_path: "file2.txt".to_string(),
                file_size: 200,
                last_modified: Utc::now(),
                etag: Some("etag2".to_string()),
                was_local: true,
                was_remote: false,
            },
        ]
    }

    #[test]
    #[serial]
    fn test_save_sync_history() {
        let pool = db::init_db_pool(TEST_SERVICE).unwrap();
        let history = create_test_history();
        let files = create_test_files();

        let result = save_sync_history(&pool, &history, &files);
        assert!(result.is_ok());

        let sync_id = result.unwrap();
        assert!(sync_id > 0);

        // Clean up
        let db_path = db::get_db_path(TEST_SERVICE).unwrap();
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    #[serial]
    fn test_get_last_sync_time() {
        let pool = db::init_db_pool(TEST_SERVICE).unwrap();
        let history = create_test_history();
        let files = create_test_files();

        // Save history
        save_sync_history(&pool, &history, &files).unwrap();

        // Get last sync time
        let result = get_last_sync_time(&pool, &history.local_path, &history.remote_prefix);
        assert!(result.is_ok());

        let last_sync_time = result.unwrap();
        assert!(last_sync_time.is_some());

        // Clean up
        let db_path = db::get_db_path(TEST_SERVICE).unwrap();
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    #[serial]
    fn test_get_last_sync_time_no_history() {
        let pool = db::init_db_pool(TEST_SERVICE).unwrap();

        let result = get_last_sync_time(&pool, "/nonexistent", "nonexistent/");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Clean up
        let db_path = db::get_db_path(TEST_SERVICE).unwrap();
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    #[serial]
    fn test_get_last_synced_files() {
        let pool = db::init_db_pool(TEST_SERVICE).unwrap();
        let history = create_test_history();
        let files = create_test_files();

        // Save history
        save_sync_history(&pool, &history, &files).unwrap();

        // Get last synced files
        let result = get_last_synced_files(&pool, &history.local_path, &history.remote_prefix);
        assert!(result.is_ok());

        let synced_files = result.unwrap();
        assert_eq!(synced_files.len(), 2);
        assert!(synced_files.contains(&"file1.txt".to_string()));
        assert!(synced_files.contains(&"file2.txt".to_string()));

        // Clean up
        let db_path = db::get_db_path(TEST_SERVICE).unwrap();
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    #[serial]
    fn test_get_last_synced_files_no_history() {
        let pool = db::init_db_pool(TEST_SERVICE).unwrap();

        let result = get_last_synced_files(&pool, "/nonexistent", "nonexistent/");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        // Clean up
        let db_path = db::get_db_path(TEST_SERVICE).unwrap();
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    #[serial]
    fn test_multiple_sync_paths() {
        let pool = db::init_db_pool(TEST_SERVICE).unwrap();

        // Save history for path 1
        let mut history1 = create_test_history();
        history1.local_path = "/test/path1".to_string();
        history1.remote_prefix = "path1/".to_string();
        let files1 = vec![SyncedFile {
            file_path: "file1.txt".to_string(),
            file_size: 100,
            last_modified: Utc::now(),
            etag: None,
            was_local: true,
            was_remote: true,
        }];
        save_sync_history(&pool, &history1, &files1).unwrap();

        // Save history for path 2
        let mut history2 = create_test_history();
        history2.local_path = "/test/path2".to_string();
        history2.remote_prefix = "path2/".to_string();
        let files2 = vec![SyncedFile {
            file_path: "file2.txt".to_string(),
            file_size: 200,
            last_modified: Utc::now(),
            etag: None,
            was_local: true,
            was_remote: true,
        }];
        save_sync_history(&pool, &history2, &files2).unwrap();

        // Get files for path 1
        let result1 = get_last_synced_files(&pool, &history1.local_path, &history1.remote_prefix);
        assert!(result1.is_ok());
        let synced_files1 = result1.unwrap();
        assert_eq!(synced_files1.len(), 1);
        assert_eq!(synced_files1[0], "file1.txt");

        // Get files for path 2
        let result2 = get_last_synced_files(&pool, &history2.local_path, &history2.remote_prefix);
        assert!(result2.is_ok());
        let synced_files2 = result2.unwrap();
        assert_eq!(synced_files2.len(), 1);
        assert_eq!(synced_files2[0], "file2.txt");

        // Clean up
        let db_path = db::get_db_path(TEST_SERVICE).unwrap();
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    #[serial]
    fn test_sync_history_ordering() {
        let pool = db::init_db_pool(TEST_SERVICE).unwrap();

        // Save first sync
        let mut history1 = create_test_history();
        let files1 = vec![SyncedFile {
            file_path: "old_file.txt".to_string(),
            file_size: 100,
            last_modified: Utc::now(),
            etag: None,
            was_local: true,
            was_remote: true,
        }];
        save_sync_history(&pool, &history1, &files1).unwrap();

        // Wait a bit to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Save second sync (more recent)
        history1.sync_started_at = Utc::now();
        history1.sync_completed_at = Utc::now();
        let files2 = vec![SyncedFile {
            file_path: "new_file.txt".to_string(),
            file_size: 200,
            last_modified: Utc::now(),
            etag: None,
            was_local: true,
            was_remote: true,
        }];
        save_sync_history(&pool, &history1, &files2).unwrap();

        // Get last synced files (should be from second sync)
        let result = get_last_synced_files(&pool, &history1.local_path, &history1.remote_prefix);
        assert!(result.is_ok());
        let synced_files = result.unwrap();
        assert_eq!(synced_files.len(), 1);
        assert_eq!(synced_files[0], "new_file.txt");

        // Clean up
        let db_path = db::get_db_path(TEST_SERVICE).unwrap();
        let _ = std::fs::remove_file(&db_path);
    }
}
