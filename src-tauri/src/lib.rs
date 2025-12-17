// モジュール宣言
pub mod storage;

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

    // 依存関係のインポートテスト（フェーズ1.1）
    #[test]
    fn test_aws_sdk_imports() {
        // AWS SDK のインポート確認（コンパイル時にチェック）
        use aws_config;

        // 型が正しくインポートできることを確認
        let _config_builder = aws_config::BehaviorVersion::latest();
        assert!(true);
    }

    #[test]
    fn test_tokio_runtime() {
        // Tokio ランタイムが動作することを確認
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            assert_eq!(2 + 2, 4);
        });
    }

    #[test]
    fn test_anyhow_error() {
        // anyhow のエラーハンドリング確認
        use anyhow::{Result, Context};

        fn test_function() -> Result<i32> {
            Ok(42)
        }

        let result = test_function().context("test context");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_thiserror() {
        // thiserror のカスタムエラー型確認
        use thiserror::Error;

        #[derive(Error, Debug)]
        enum TestError {
            #[error("test error: {0}")]
            Test(String),
        }

        let err = TestError::Test("example".to_string());
        assert_eq!(err.to_string(), "test error: example");
    }

    #[test]
    fn test_chrono() {
        // chrono の日時処理確認
        use chrono::{Utc, DateTime};

        let now: DateTime<Utc> = Utc::now();
        assert!(now.timestamp() > 0);
    }

    #[test]
    fn test_keyring() {
        // keyring のインポート確認（実際の操作はスキップ）
        use keyring::Entry;

        // キーリングエントリーの型が利用可能であることを確認
        let _entry_type = std::marker::PhantomData::<Entry>;
        assert!(true);
    }

    #[test]
    fn test_serde() {
        // serde, serde_json の確認（既存の依存関係）
        use serde::{Serialize, Deserialize};

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
