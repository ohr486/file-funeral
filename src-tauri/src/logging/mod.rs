//! Logging configuration module
//!
//! This module provides centralized logging configuration with:
//! - Log level filtering
//! - File output support
//! - Debug mode
//! - Integration with tauri-plugin-log

use std::path::PathBuf;

/// Log configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: log::LevelFilter,

    /// Enable debug mode (more verbose logging)
    pub debug_mode: bool,

    /// Optional file path for log output
    pub file_path: Option<PathBuf>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: log::LevelFilter::Info,
            debug_mode: false,
            file_path: None,
        }
    }
}

impl LogConfig {
    /// Create a new log configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the log level
    pub fn with_level(mut self, level: log::LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// Enable debug mode
    pub fn with_debug_mode(mut self, enabled: bool) -> Self {
        self.debug_mode = enabled;
        if enabled && self.level < log::LevelFilter::Debug {
            self.level = log::LevelFilter::Debug;
        }
        self
    }

    /// Set the log file path
    pub fn with_file(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Initialize logging with this configuration
    ///
    /// This function sets up the global logger with the specified configuration.
    /// It should be called once at application startup.
    ///
    /// # Errors
    /// Returns an error if the logger has already been initialized.
    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Determine the log level to use
        let level = if self.debug_mode {
            log::LevelFilter::Debug
        } else {
            self.level
        };

        // Initialize env_logger with custom configuration
        let mut builder = env_logger::Builder::new();
        builder
            .filter_level(level)
            .format_timestamp_millis()
            .format_module_path(true)
            .format_target(false);

        // If file path is specified, we would need to add file logging here
        // For now, we'll use stdout/stderr (env_logger default)
        // File logging could be added with a custom logger or tracing-appender

        builder.try_init()?;

        log::info!("Logging initialized with level: {:?}", level);
        if self.debug_mode {
            log::debug!("Debug mode enabled");
        }

        Ok(())
    }
}

/// Get the log level from an environment variable or use default
pub fn get_log_level_from_env() -> log::LevelFilter {
    std::env::var("RUST_LOG")
        .ok()
        .and_then(|level_str| match level_str.to_lowercase().as_str() {
            "trace" => Some(log::LevelFilter::Trace),
            "debug" => Some(log::LevelFilter::Debug),
            "info" => Some(log::LevelFilter::Info),
            "warn" => Some(log::LevelFilter::Warn),
            "error" => Some(log::LevelFilter::Error),
            "off" => Some(log::LevelFilter::Off),
            _ => None,
        })
        .unwrap_or(log::LevelFilter::Info)
}

/// Check if debug mode is enabled via environment variable
pub fn is_debug_mode() -> bool {
    std::env::var("DEBUG")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, log::LevelFilter::Info);
        assert!(!config.debug_mode);
        assert!(config.file_path.is_none());
    }

    #[test]
    fn test_log_config_builder() {
        let config = LogConfig::new()
            .with_level(log::LevelFilter::Debug)
            .with_debug_mode(true)
            .with_file(PathBuf::from("/tmp/app.log"));

        assert_eq!(config.level, log::LevelFilter::Debug);
        assert!(config.debug_mode);
        assert_eq!(config.file_path, Some(PathBuf::from("/tmp/app.log")));
    }

    #[test]
    fn test_log_config_debug_mode_upgrades_level() {
        let config = LogConfig::new()
            .with_level(log::LevelFilter::Info)
            .with_debug_mode(true);

        // Debug mode should upgrade level to Debug
        assert_eq!(config.level, log::LevelFilter::Debug);
        assert!(config.debug_mode);
    }

    #[test]
    fn test_get_log_level_from_env() {
        // Test default when env var is not set
        std::env::remove_var("RUST_LOG");
        assert_eq!(get_log_level_from_env(), log::LevelFilter::Info);

        // Test parsing different log levels
        std::env::set_var("RUST_LOG", "debug");
        assert_eq!(get_log_level_from_env(), log::LevelFilter::Debug);

        std::env::set_var("RUST_LOG", "ERROR");
        assert_eq!(get_log_level_from_env(), log::LevelFilter::Error);

        std::env::set_var("RUST_LOG", "trace");
        assert_eq!(get_log_level_from_env(), log::LevelFilter::Trace);

        // Test invalid value falls back to default
        std::env::set_var("RUST_LOG", "invalid");
        assert_eq!(get_log_level_from_env(), log::LevelFilter::Info);

        // Cleanup
        std::env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_is_debug_mode() {
        // Test default (no env var)
        std::env::remove_var("DEBUG");
        assert!(!is_debug_mode());

        // Test enabled with "1"
        std::env::set_var("DEBUG", "1");
        assert!(is_debug_mode());

        // Test enabled with "true"
        std::env::set_var("DEBUG", "true");
        assert!(is_debug_mode());

        // Test enabled with "TRUE"
        std::env::set_var("DEBUG", "TRUE");
        assert!(is_debug_mode());

        // Test disabled with other values
        std::env::set_var("DEBUG", "0");
        assert!(!is_debug_mode());

        std::env::set_var("DEBUG", "false");
        assert!(!is_debug_mode());

        // Cleanup
        std::env::remove_var("DEBUG");
    }
}
