// モジュール宣言
pub mod auth;
pub mod commands;
pub mod db;
pub mod storage;
pub mod sync;

use tauri::{AppHandle, Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize database pool
            let db_pool = db::init_db_pool("file-funeral").expect("Failed to initialize database");
            app.manage(db_pool);

            // Auto-sync on startup (Phase 3.4)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                perform_auto_sync_on_startup(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_credentials,
            commands::get_credentials,
            commands::test_connection,
            commands::list_files,
            commands::get_sync_status,
            commands::sync_files,
            commands::save_sync_settings,
            commands::get_sync_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Handle app lifecycle events
            if let RunEvent::Exit = event {
                // Auto-sync on shutdown (Phase 3.4)
                let handle = app_handle.clone();
                tauri::async_runtime::block_on(async move {
                    perform_auto_sync_on_shutdown(handle).await;
                });
            }
        });
}

// ============================================================================
// Auto-sync helper functions (Phase 3.4)
// ============================================================================

/// Perform auto-sync on application startup
///
/// This function:
/// 1. Loads sync settings from the database
/// 2. Checks if auto-sync on startup is enabled
/// 3. If enabled, performs a sync operation
async fn perform_auto_sync_on_startup(app_handle: AppHandle) {
    log::info!("Checking auto-sync on startup");

    // Get database pool from app state
    let db_pool = match app_handle.try_state::<db::DbPool>() {
        Some(pool) => pool.inner().clone(),
        None => {
            log::error!("Failed to get database pool from app state");
            return;
        }
    };

    // Load settings
    let settings = match db::settings::load_settings(&db_pool) {
        Ok(Some(settings)) => settings,
        Ok(None) => {
            log::info!("No sync settings configured, skipping auto-sync on startup");
            return;
        }
        Err(e) => {
            log::error!("Failed to load sync settings: {}", e);
            return;
        }
    };

    // Check if auto-sync on startup is enabled
    if !settings.auto_sync_on_startup {
        log::info!("Auto-sync on startup is disabled");
        return;
    }

    log::info!(
        "Auto-sync on startup enabled, syncing: {} <-> {}",
        settings.local_path,
        settings.remote_prefix
    );

    // Perform sync
    let request = commands::SyncFilesRequest {
        local_path: settings.local_path.clone(),
        remote_prefix: settings.remote_prefix.clone(),
    };

    match commands::sync_files_impl(&request, &db_pool).await {
        Ok(response) => {
            log::info!("Auto-sync on startup completed: {}", response.message);
        }
        Err(e) => {
            log::error!("Auto-sync on startup failed: {}", e);
        }
    }
}

/// Perform auto-sync on application shutdown
///
/// This function:
/// 1. Loads sync settings from the database
/// 2. Checks if auto-sync on shutdown is enabled
/// 3. If enabled, performs a sync operation
async fn perform_auto_sync_on_shutdown(app_handle: AppHandle) {
    log::info!("Checking auto-sync on shutdown");

    // Get database pool from app state
    let db_pool = match app_handle.try_state::<db::DbPool>() {
        Some(pool) => pool.inner().clone(),
        None => {
            log::error!("Failed to get database pool from app state");
            return;
        }
    };

    // Load settings
    let settings = match db::settings::load_settings(&db_pool) {
        Ok(Some(settings)) => settings,
        Ok(None) => {
            log::info!("No sync settings configured, skipping auto-sync on shutdown");
            return;
        }
        Err(e) => {
            log::error!("Failed to load sync settings: {}", e);
            return;
        }
    };

    // Check if auto-sync on shutdown is enabled
    if !settings.auto_sync_on_shutdown {
        log::info!("Auto-sync on shutdown is disabled");
        return;
    }

    log::info!(
        "Auto-sync on shutdown enabled, syncing: {} <-> {}",
        settings.local_path,
        settings.remote_prefix
    );

    // Perform sync
    let request = commands::SyncFilesRequest {
        local_path: settings.local_path.clone(),
        remote_prefix: settings.remote_prefix.clone(),
    };

    match commands::sync_files_impl(&request, &db_pool).await {
        Ok(response) => {
            log::info!("Auto-sync on shutdown completed: {}", response.message);
        }
        Err(e) => {
            log::error!("Auto-sync on shutdown failed: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        // 基本的なテストの例
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_app_name() {
        // アプリケーション名の確認
        let app_name = env!("CARGO_PKG_NAME");
        assert_eq!(app_name, "app");
    }

    // 依存関係の実用的なテスト（フェーズ1.1）

    #[tokio::test]
    async fn test_tokio_async_operations() {
        // Tokioの非同期処理が正しく動作することを確認
        use tokio::time::{sleep, Duration};

        let start = std::time::Instant::now();
        sleep(Duration::from_millis(10)).await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_anyhow_error_propagation() {
        // anyhow のエラー伝播とコンテキスト追加を確認
        use anyhow::{anyhow, Context, Result};

        fn inner_function() -> Result<()> {
            Err(anyhow!("inner error"))
        }

        fn outer_function() -> Result<()> {
            inner_function().context("outer context")?;
            Ok(())
        }

        let result = outer_function();
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(err_msg.contains("outer context"));
        assert!(err_msg.contains("inner error"));
    }

    #[test]
    fn test_chrono_datetime_parsing() {
        // chrono の日時パース機能を確認
        use chrono::{DateTime, TimeZone, Utc};

        let dt_str = "2025-12-19T10:30:45Z";
        let parsed = DateTime::parse_from_rfc3339(dt_str);
        assert!(parsed.is_ok());

        let expected = Utc.with_ymd_and_hms(2025, 12, 19, 10, 30, 45).unwrap();
        assert_eq!(parsed.unwrap().with_timezone(&Utc), expected);
    }

    #[test]
    fn test_serde() {
        // serde, serde_json の確認（既存の依存関係）
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let test = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        let json = serde_json::to_string(&test).unwrap();
        let decoded: TestStruct = serde_json::from_str(&json).unwrap();
        assert_eq!(test, decoded);
    }
}
