//! Database management module
//!
//! This module handles SQLite database operations for sync history tracking.
//! It uses r2d2 for connection pooling and follows the same pattern as auth/mod.rs
//! for file path management.
//!
//! Database location:
//! - Production: ~/.file-funeral/sync_history.db
//! - Test: /tmp/.file-funeral-test/sync_history.db

pub mod schema;
pub mod sync_history;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Type alias for the database connection pool
pub type DbPool = Pool<SqliteConnectionManager>;

/// Errors that can occur during database operations
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(#[from] r2d2::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Database path not found")]
    PathNotFound,

    #[error("Invalid database state: {0}")]
    InvalidState(String),
}

/// Get the path to the database file
///
/// Following the same pattern as auth/mod.rs:get_credentials_file_path()
/// - Test services: /tmp/.{service-name}/sync_history.db
/// - Production: ~/.{service-name}/sync_history.db
pub fn get_db_path(service_name: &str) -> Result<PathBuf, DbError> {
    // Determine config directory based on service name
    let config_dir = if service_name.starts_with("file-funeral-test") {
        // Use /tmp for test database
        PathBuf::from("/tmp").join(format!(".{}", service_name))
    } else {
        // Use home directory for production database
        let home = std::env::var("HOME").map_err(|_| {
            DbError::FileSystem(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME directory not found",
            ))
        })?;
        PathBuf::from(home).join(format!(".{}", service_name))
    };

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;

        // Set directory permissions to 700 (owner only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&config_dir)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&config_dir, perms)?;
        }
    }

    Ok(config_dir.join("sync_history.db"))
}

/// Initialize the database connection pool
///
/// This function:
/// 1. Gets the database file path
/// 2. Creates an r2d2 connection pool with max size 5
/// 3. Runs database migrations
///
/// # Arguments
/// * `service_name` - Service name (e.g., "file-funeral" or "file-funeral-test")
///
/// # Returns
/// * `Ok(DbPool)` - Initialized connection pool
/// * `Err(DbError)` - If initialization fails
pub fn init_db_pool(service_name: &str) -> Result<DbPool, DbError> {
    log::info!("Initializing database pool for service: {}", service_name);

    // Get database path
    let db_path = get_db_path(service_name)?;
    log::info!("Database path: {:?}", db_path);

    // Create connection manager
    let manager = SqliteConnectionManager::file(&db_path);

    // Build connection pool
    let pool = Pool::builder().max_size(5).build(manager)?;

    log::info!("Database pool created successfully");

    // Run migrations
    let conn = pool.get()?;
    schema::run_migrations(&conn)?;
    log::info!("Database migrations completed");

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SERVICE: &str = "file-funeral-test";

    #[test]
    fn test_get_db_path_production() {
        let service = "file-funeral";
        let result = get_db_path(service);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".file-funeral"));
        assert!(path.to_string_lossy().ends_with("sync_history.db"));
    }

    #[test]
    fn test_get_db_path_test() {
        let service = TEST_SERVICE;
        let result = get_db_path(service);
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path
            .to_string_lossy()
            .starts_with("/tmp/.file-funeral-test"));
        assert!(path.to_string_lossy().ends_with("sync_history.db"));
    }

    #[test]
    fn test_init_db_pool() {
        let service = TEST_SERVICE;
        let result = init_db_pool(service);

        // Clean up
        if result.is_ok() {
            let db_path = get_db_path(service).unwrap();
            let _ = std::fs::remove_file(&db_path);
        }

        assert!(result.is_ok());
    }

    #[test]
    fn test_db_pool_get_connection() {
        let service = TEST_SERVICE;
        let pool = init_db_pool(service).unwrap();

        // Get connection from pool
        let conn = pool.get();
        assert!(conn.is_ok());

        // Clean up
        let db_path = get_db_path(service).unwrap();
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_db_pool_multiple_connections() {
        let service = TEST_SERVICE;
        let pool = init_db_pool(service).unwrap();

        // Get multiple connections
        let conn1 = pool.get();
        let conn2 = pool.get();
        let conn3 = pool.get();

        assert!(conn1.is_ok());
        assert!(conn2.is_ok());
        assert!(conn3.is_ok());

        // Clean up
        let db_path = get_db_path(service).unwrap();
        let _ = std::fs::remove_file(&db_path);
    }
}
