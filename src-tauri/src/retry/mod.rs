//! Retry logic module
//!
//! This module provides retry functionality with exponential backoff
//! for network operations and other potentially failing operations.

use backon::{ExponentialBuilder, Retryable};
use std::time::Duration;

/// Default retry configuration
pub const DEFAULT_MAX_RETRIES: usize = 3;
pub const DEFAULT_MIN_DELAY_MS: u64 = 100;
pub const DEFAULT_MAX_DELAY_MS: u64 = 10_000;
pub const DEFAULT_FACTOR: f32 = 2.0;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: usize,

    /// Minimum delay between retries (milliseconds)
    pub min_delay_ms: u64,

    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,

    /// Backoff factor (for exponential backoff)
    pub factor: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            min_delay_ms: DEFAULT_MIN_DELAY_MS,
            max_delay_ms: DEFAULT_MAX_DELAY_MS,
            factor: DEFAULT_FACTOR,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set minimum delay
    pub fn with_min_delay(mut self, min_delay_ms: u64) -> Self {
        self.min_delay_ms = min_delay_ms;
        self
    }

    /// Set maximum delay
    pub fn with_max_delay(mut self, max_delay_ms: u64) -> Self {
        self.max_delay_ms = max_delay_ms;
        self
    }

    /// Set backoff factor
    pub fn with_factor(mut self, factor: f32) -> Self {
        self.factor = factor;
        self
    }

    /// Build an ExponentialBuilder from this configuration
    pub fn build_backoff(&self) -> ExponentialBuilder {
        ExponentialBuilder::default()
            .with_min_delay(Duration::from_millis(self.min_delay_ms))
            .with_max_delay(Duration::from_millis(self.max_delay_ms))
            .with_max_times(self.max_retries)
            .with_factor(self.factor)
    }
}

/// Retry an async operation with exponential backoff
///
/// This is a helper function that wraps backon's retry functionality
/// with our default configuration and logging.
///
/// # Example
/// ```no_run
/// use app_lib::retry::retry_with_backoff;
/// use anyhow::Result;
///
/// async fn fetch_data() -> Result<String> {
///     // Your async operation here
///     Ok("data".to_string())
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let result = retry_with_backoff(fetch_data).await?;
///     Ok(())
/// }
/// ```
pub async fn retry_with_backoff<F, Fut, T, E>(operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let config = RetryConfig::default();
    retry_with_config(operation, &config).await
}

/// Retry an async operation with custom configuration
pub async fn retry_with_config<F, Fut, T, E>(operation: F, config: &RetryConfig) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let backoff = config.build_backoff();

    operation
        .retry(backoff)
        .sleep(tokio::time::sleep)
        .notify(|err: &E, dur: Duration| {
            log::warn!("Operation failed: {}. Retrying after {:?}...", err, dur);
        })
        .await
}

/// Check if an error is retryable (network-related)
pub fn is_network_error(error_msg: &str) -> bool {
    let error_lower = error_msg.to_lowercase();
    error_lower.contains("network")
        || error_lower.contains("timeout")
        || error_lower.contains("connection")
        || error_lower.contains("dns")
        || error_lower.contains("eof")
        || error_lower.contains("broken pipe")
        || error_lower.contains("connection reset")
}

/// Check if an error is an authentication error (not retryable)
pub fn is_auth_error(error_msg: &str) -> bool {
    let error_lower = error_msg.to_lowercase();
    error_lower.contains("unauthorized")
        || error_lower.contains("forbidden")
        || error_lower.contains("authentication")
        || error_lower.contains("invalid credentials")
        || error_lower.contains("access denied")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, DEFAULT_MAX_RETRIES);
        assert_eq!(config.min_delay_ms, DEFAULT_MIN_DELAY_MS);
        assert_eq!(config.max_delay_ms, DEFAULT_MAX_DELAY_MS);
        assert_eq!(config.factor, DEFAULT_FACTOR);
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new()
            .with_max_retries(5)
            .with_min_delay(200)
            .with_max_delay(20_000)
            .with_factor(3.0);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.min_delay_ms, 200);
        assert_eq!(config.max_delay_ms, 20_000);
        assert_eq!(config.factor, 3.0);
    }

    #[test]
    fn test_build_backoff() {
        let config = RetryConfig::new()
            .with_max_retries(3)
            .with_min_delay(100)
            .with_max_delay(5000);

        let _backoff = config.build_backoff();
        // Just verify it builds without panic
    }

    #[test]
    fn test_is_network_error() {
        // Network-related errors
        assert!(is_network_error("Network timeout occurred"));
        assert!(is_network_error("Connection refused"));
        assert!(is_network_error("DNS resolution failed"));
        assert!(is_network_error("EOF while reading"));
        assert!(is_network_error("Broken pipe"));
        assert!(is_network_error("Connection reset by peer"));

        // Case insensitive
        assert!(is_network_error("NETWORK ERROR"));
        assert!(is_network_error("Timeout"));

        // Non-network errors
        assert!(!is_network_error("File not found"));
        assert!(!is_network_error("Permission denied"));
        assert!(!is_network_error("Invalid input"));
    }

    #[test]
    fn test_is_auth_error() {
        // Authentication-related errors
        assert!(is_auth_error("Unauthorized access"));
        assert!(is_auth_error("403 Forbidden"));
        assert!(is_auth_error("Authentication failed"));
        assert!(is_auth_error("Invalid credentials"));
        assert!(is_auth_error("Access denied"));

        // Case insensitive
        assert!(is_auth_error("UNAUTHORIZED"));
        assert!(is_auth_error("Forbidden"));

        // Non-auth errors
        assert!(!is_auth_error("Network timeout"));
        assert!(!is_auth_error("File not found"));
        assert!(!is_auth_error("Connection refused"));
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = retry_with_backoff(|| async {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok::<_, String>("success".to_string())
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        // Should succeed on first try
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_eventual_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = retry_with_backoff(|| {
            let counter = Arc::clone(&counter_clone);
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err("temporary error".to_string())
                } else {
                    Ok("success".to_string())
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        // Should succeed on third try
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_max_retries() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = retry_with_backoff(|| {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<String, _>("persistent error".to_string())
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "persistent error");
        // Should try max_retries + 1 times (initial + retries)
        assert_eq!(counter.load(Ordering::SeqCst), DEFAULT_MAX_RETRIES + 1);
    }

    #[tokio::test]
    async fn test_retry_with_custom_config() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let config = RetryConfig::new()
            .with_max_retries(2)
            .with_min_delay(10)
            .with_max_delay(100);

        let result = retry_with_config(
            || {
                let counter = Arc::clone(&counter_clone);
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("error".to_string())
                }
            },
            &config,
        )
        .await;

        assert!(result.is_err());
        // Should try max_retries + 1 times
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
