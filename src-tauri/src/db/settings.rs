//! Settings management module
//!
//! This module handles storage and retrieval of sync settings from the SQLite database.
//! Settings include local path, remote prefix, and auto-sync flags.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{DbError, DbPool};

/// Sync settings stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub local_path: String,
    pub remote_prefix: String,
    pub auto_sync_on_startup: bool,
    pub auto_sync_on_shutdown: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl SyncSettings {
    /// Create a new SyncSettings instance
    pub fn new(
        local_path: String,
        remote_prefix: String,
        auto_sync_on_startup: bool,
        auto_sync_on_shutdown: bool,
    ) -> Self {
        Self {
            local_path,
            remote_prefix,
            auto_sync_on_startup,
            auto_sync_on_shutdown,
            created_at: None,
            updated_at: None,
        }
    }
}

/// Save or update sync settings
///
/// This function uses an UPSERT pattern to either insert new settings or update existing ones.
/// There can only be one settings record in the database (id = 1).
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `settings` - Settings to save
///
/// # Returns
/// * `Ok(())` - If settings were saved successfully
/// * `Err(DbError)` - If the operation fails
pub fn save_settings(pool: &DbPool, settings: &SyncSettings) -> Result<(), DbError> {
    let conn = pool.get()?;
    let now = Utc::now().to_rfc3339();

    // Use UPSERT to insert or update settings
    conn.execute(
        "INSERT INTO settings (id, local_path, remote_prefix, auto_sync_on_startup, auto_sync_on_shutdown, created_at, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            local_path = excluded.local_path,
            remote_prefix = excluded.remote_prefix,
            auto_sync_on_startup = excluded.auto_sync_on_startup,
            auto_sync_on_shutdown = excluded.auto_sync_on_shutdown,
            updated_at = excluded.updated_at",
        rusqlite::params![
            settings.local_path,
            settings.remote_prefix,
            settings.auto_sync_on_startup as i32,
            settings.auto_sync_on_shutdown as i32,
            now,
            now,
        ],
    )?;

    log::info!("Settings saved successfully");
    Ok(())
}

/// Load sync settings from the database
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Ok(Some(SyncSettings))` - If settings exist
/// * `Ok(None)` - If no settings are configured
/// * `Err(DbError)` - If the operation fails
pub fn load_settings(pool: &DbPool) -> Result<Option<SyncSettings>, DbError> {
    let conn = pool.get()?;

    let result = conn.query_row(
        "SELECT local_path, remote_prefix, auto_sync_on_startup, auto_sync_on_shutdown, created_at, updated_at
         FROM settings
         WHERE id = 1",
        [],
        |row| {
            Ok(SyncSettings {
                local_path: row.get(0)?,
                remote_prefix: row.get(1)?,
                auto_sync_on_startup: row.get::<_, i32>(2)? != 0,
                auto_sync_on_shutdown: row.get::<_, i32>(3)? != 0,
                created_at: row.get::<_, String>(4).ok().and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                updated_at: row.get::<_, String>(5).ok().and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            })
        },
    );

    match result {
        Ok(settings) => {
            log::info!("Settings loaded successfully");
            Ok(Some(settings))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            log::info!("No settings found");
            Ok(None)
        }
        Err(e) => Err(DbError::Sqlite(e)),
    }
}

/// Delete sync settings from the database
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Ok(())` - If settings were deleted successfully
/// * `Err(DbError)` - If the operation fails
pub fn delete_settings(pool: &DbPool) -> Result<(), DbError> {
    let conn = pool.get()?;

    conn.execute("DELETE FROM settings WHERE id = 1", [])?;

    log::info!("Settings deleted successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test database pool
    fn create_test_db_pool() -> DbPool {
        let test_service = format!("file-funeral-test-settings-{}", std::process::id());
        if let Ok(db_path) = crate::db::get_db_path(&test_service) {
            let _ = std::fs::remove_file(&db_path);
        }

        crate::db::init_db_pool(&test_service).expect("Failed to initialize test database")
    }

    #[test]
    fn test_save_and_load_settings() {
        let pool = create_test_db_pool();

        let settings = SyncSettings::new(
            "/home/user/documents".to_string(),
            "documents/".to_string(),
            true,
            false,
        );

        // Save settings
        let save_result = save_settings(&pool, &settings);
        assert!(save_result.is_ok());

        // Load settings
        let load_result = load_settings(&pool);
        assert!(load_result.is_ok());

        let loaded = load_result.unwrap();
        assert!(loaded.is_some());

        let loaded_settings = loaded.unwrap();
        assert_eq!(loaded_settings.local_path, settings.local_path);
        assert_eq!(loaded_settings.remote_prefix, settings.remote_prefix);
        assert_eq!(
            loaded_settings.auto_sync_on_startup,
            settings.auto_sync_on_startup
        );
        assert_eq!(
            loaded_settings.auto_sync_on_shutdown,
            settings.auto_sync_on_shutdown
        );
        assert!(loaded_settings.created_at.is_some());
        assert!(loaded_settings.updated_at.is_some());
    }

    #[test]
    fn test_update_settings() {
        let pool = create_test_db_pool();

        // Save initial settings
        let settings1 =
            SyncSettings::new("/path1".to_string(), "prefix1/".to_string(), true, false);
        save_settings(&pool, &settings1).unwrap();

        // Update settings
        let settings2 =
            SyncSettings::new("/path2".to_string(), "prefix2/".to_string(), false, true);
        save_settings(&pool, &settings2).unwrap();

        // Load and verify
        let loaded = load_settings(&pool).unwrap().unwrap();
        assert_eq!(loaded.local_path, "/path2");
        assert_eq!(loaded.remote_prefix, "prefix2/");
        assert_eq!(loaded.auto_sync_on_startup, false);
        assert_eq!(loaded.auto_sync_on_shutdown, true);
    }

    #[test]
    fn test_load_nonexistent_settings() {
        let pool = create_test_db_pool();

        let result = load_settings(&pool);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_delete_settings() {
        let pool = create_test_db_pool();

        // Save settings
        let settings = SyncSettings::new("/test".to_string(), "test/".to_string(), true, true);
        save_settings(&pool, &settings).unwrap();

        // Verify it exists
        let loaded = load_settings(&pool).unwrap();
        assert!(loaded.is_some());

        // Delete settings
        let delete_result = delete_settings(&pool);
        assert!(delete_result.is_ok());

        // Verify it's gone
        let loaded_after = load_settings(&pool).unwrap();
        assert!(loaded_after.is_none());
    }

    #[test]
    fn test_settings_serialization() {
        let settings = SyncSettings::new(
            "/home/user/docs".to_string(),
            "docs/".to_string(),
            true,
            false,
        );

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: SyncSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.local_path, deserialized.local_path);
        assert_eq!(settings.remote_prefix, deserialized.remote_prefix);
        assert_eq!(
            settings.auto_sync_on_startup,
            deserialized.auto_sync_on_startup
        );
        assert_eq!(
            settings.auto_sync_on_shutdown,
            deserialized.auto_sync_on_shutdown
        );
    }
}
