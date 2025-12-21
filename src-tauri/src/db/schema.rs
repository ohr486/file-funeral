//! Database schema management and migrations
//!
//! This module handles database schema versioning and migrations.
//! It creates the necessary tables for sync history tracking.

use rusqlite::Connection;

use super::DbError;

/// Current schema version
const CURRENT_SCHEMA_VERSION: i32 = 2;

/// Run all pending database migrations
///
/// This function:
/// 1. Creates the schema_version table if it doesn't exist
/// 2. Checks the current schema version
/// 3. Applies any pending migrations
///
/// # Arguments
/// * `conn` - SQLite database connection
///
/// # Returns
/// * `Ok(())` - If migrations succeed
/// * `Err(DbError)` - If any migration fails
pub fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    log::info!("Running database migrations");

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Create schema_version table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    // Get current schema version
    let current_version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    log::info!("Current schema version: {}", current_version);

    // Apply pending migrations
    if current_version < 1 {
        log::info!("Applying migration v1");
        apply_migration_v1(conn)?;
    }

    if current_version < 2 {
        log::info!("Applying migration v2");
        apply_migration_v2(conn)?;
    }

    log::info!(
        "Migrations complete. Current version: {}",
        CURRENT_SCHEMA_VERSION
    );
    Ok(())
}

/// Apply migration v1
///
/// Creates the initial database schema:
/// - sync_history table: Tracks sync execution history
/// - synced_files table: Stores file list from each sync
/// - Indexes for performance
fn apply_migration_v1(conn: &Connection) -> Result<(), DbError> {
    log::info!("Creating sync_history table");

    // Create sync_history table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sync_started_at TEXT NOT NULL,
            sync_completed_at TEXT NOT NULL,
            local_path TEXT NOT NULL,
            remote_prefix TEXT NOT NULL,
            files_uploaded INTEGER NOT NULL DEFAULT 0,
            files_downloaded INTEGER NOT NULL DEFAULT 0,
            files_deleted INTEGER NOT NULL DEFAULT 0,
            conflicts_resolved INTEGER NOT NULL DEFAULT 0,
            success INTEGER NOT NULL,
            error_message TEXT
        )",
        [],
    )?;

    // Create indexes for sync_history
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sync_history_completed
            ON sync_history(sync_completed_at DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sync_history_path
            ON sync_history(local_path, remote_prefix)",
        [],
    )?;

    log::info!("Creating synced_files table");

    // Create synced_files table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS synced_files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sync_history_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            last_modified TEXT NOT NULL,
            etag TEXT,
            was_local INTEGER NOT NULL,
            was_remote INTEGER NOT NULL,
            FOREIGN KEY (sync_history_id) REFERENCES sync_history(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Create indexes for synced_files
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_synced_files_history
            ON synced_files(sync_history_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_synced_files_path
            ON synced_files(file_path)",
        [],
    )?;

    log::info!("Recording schema version");

    // Record schema version
    conn.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (?1, datetime('now'))",
        [1],
    )?;

    log::info!("Migration v1 completed successfully");
    Ok(())
}

/// Apply migration v2
///
/// Creates the settings table for storing sync configuration:
/// - local_path: Local folder path to sync
/// - remote_prefix: S3 prefix to sync with
/// - auto_sync_on_startup: Enable auto-sync on app startup
/// - auto_sync_on_shutdown: Enable auto-sync on app shutdown
/// - created_at: When the setting was first created
/// - updated_at: When the setting was last updated
fn apply_migration_v2(conn: &Connection) -> Result<(), DbError> {
    log::info!("Creating settings table");

    // Create settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            local_path TEXT NOT NULL,
            remote_prefix TEXT NOT NULL,
            auto_sync_on_startup INTEGER NOT NULL DEFAULT 1,
            auto_sync_on_shutdown INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    log::info!("Recording schema version");

    // Record schema version
    conn.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (?1, datetime('now'))",
        [2],
    )?;

    log::info!("Migration v2 completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_run_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        let result = run_migrations(&conn);
        assert!(result.is_ok());
    }

    #[test]
    fn test_schema_version_table_created() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Check schema_version table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sync_history_table_created() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Check sync_history table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sync_history'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_synced_files_table_created() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Check synced_files table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='synced_files'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_indexes_created() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Check indexes exist
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 4); // 2 for sync_history, 2 for synced_files
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations twice
        run_migrations(&conn).unwrap();
        let result = run_migrations(&conn);

        // Should succeed (idempotent)
        assert!(result.is_ok());
    }

    #[test]
    fn test_schema_version_recorded() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Check schema version
        let version: i32 = conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Insert a sync_history record
        conn.execute(
            "INSERT INTO sync_history
            (sync_started_at, sync_completed_at, local_path, remote_prefix, success)
            VALUES ('2025-01-01T00:00:00Z', '2025-01-01T00:01:00Z', '/test', 'test/', 1)",
            [],
        )
        .unwrap();

        let sync_id: i64 = conn.last_insert_rowid();

        // Insert a synced_file referencing the sync
        let result = conn.execute(
            "INSERT INTO synced_files
            (sync_history_id, file_path, file_size, last_modified, was_local, was_remote)
            VALUES (?1, 'test.txt', 100, '2025-01-01T00:00:00Z', 1, 1)",
            [sync_id],
        );
        assert!(result.is_ok());

        // Try to insert with invalid foreign key
        let invalid_result = conn.execute(
            "INSERT INTO synced_files
            (sync_history_id, file_path, file_size, last_modified, was_local, was_remote)
            VALUES (99999, 'test2.txt', 100, '2025-01-01T00:00:00Z', 1, 1)",
            [],
        );
        // Should fail due to foreign key constraint
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_settings_table_created() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Check settings table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='settings'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_settings_table_single_row_constraint() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Insert first setting
        let result1 = conn.execute(
            "INSERT INTO settings (id, local_path, remote_prefix, created_at, updated_at)
             VALUES (1, '/test', 'test/', datetime('now'), datetime('now'))",
            [],
        );
        assert!(result1.is_ok());

        // Try to insert another setting with id=1 (should fail due to PRIMARY KEY)
        let result2 = conn.execute(
            "INSERT INTO settings (id, local_path, remote_prefix, created_at, updated_at)
             VALUES (1, '/test2', 'test2/', datetime('now'), datetime('now'))",
            [],
        );
        assert!(result2.is_err());

        // Try to insert with id=2 (should fail due to CHECK constraint)
        let result3 = conn.execute(
            "INSERT INTO settings (id, local_path, remote_prefix, created_at, updated_at)
             VALUES (2, '/test2', 'test2/', datetime('now'), datetime('now'))",
            [],
        );
        assert!(result3.is_err());
    }
}
