#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
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
}
