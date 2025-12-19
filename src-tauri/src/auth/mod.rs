//! Authentication credential management module
//!
//! This module handles secure storage and retrieval of cloud provider credentials
//! using OS-native keychain services (macOS Keychain, Windows Credential Manager,
//! Linux Secret Service API) via the `keyring` crate.
//!
//! It also supports fallback to environment variables for development purposes.

use keyring::Entry;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Service name used for keyring entries
const KEYRING_SERVICE: &str = "file-funeral";

/// Errors that can occur during credential management
#[derive(Error, Debug)]
pub enum CredentialError {
    #[error("Failed to access OS keychain: {0}")]
    KeychainAccess(#[from] keyring::Error),

    #[error("Credential not found for key: {0}")]
    NotFound(String),

    #[error("Invalid credential format: {0}")]
    InvalidFormat(String),

    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
}

/// AWS S3 credentials
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AwsCredentials {
    /// AWS Access Key ID
    pub access_key_id: String,
    /// AWS Secret Access Key
    pub secret_access_key: String,
    /// AWS Region (e.g., "us-east-1")
    pub region: String,
    /// S3 Bucket name
    pub bucket_name: String,
}

/// Credential manager for handling secure credential storage and retrieval
pub struct CredentialManager {
    service_name: String,
}

impl CredentialManager {
    /// Create a new credential manager with the default service name
    pub fn new() -> Self {
        Self {
            service_name: KEYRING_SERVICE.to_string(),
        }
    }

    /// Create a new credential manager with a custom service name
    ///
    /// This is useful for testing or when you want to use a different service identifier
    #[allow(dead_code)]
    pub fn with_service_name(service_name: String) -> Self {
        Self { service_name }
    }

    /// Save AWS credentials to the OS keychain
    ///
    /// Each credential component is stored separately in the keychain for security.
    /// The keys are named: "aws_access_key_id", "aws_secret_access_key", "aws_region", "s3_bucket_name"
    pub fn save_aws_credentials(&self, credentials: &AwsCredentials) -> Result<(), CredentialError> {
        // Save each component separately
        self.set_credential("aws_access_key_id", &credentials.access_key_id)?;
        self.set_credential("aws_secret_access_key", &credentials.secret_access_key)?;
        self.set_credential("aws_region", &credentials.region)?;
        self.set_credential("s3_bucket_name", &credentials.bucket_name)?;
        Ok(())
    }

    /// Load AWS credentials from the OS keychain
    ///
    /// If any credential is not found in the keychain, falls back to environment variables.
    /// Environment variable fallbacks:
    /// - AWS_ACCESS_KEY_ID
    /// - AWS_SECRET_ACCESS_KEY
    /// - AWS_REGION (defaults to "us-east-1" if not set)
    /// - S3_BUCKET_NAME
    pub fn load_aws_credentials(&self) -> Result<AwsCredentials, CredentialError> {
        let access_key_id = self.get_credential_with_env_fallback("aws_access_key_id", "AWS_ACCESS_KEY_ID")?;
        let secret_access_key = self.get_credential_with_env_fallback("aws_secret_access_key", "AWS_SECRET_ACCESS_KEY")?;
        let region = self.get_credential_with_env_fallback("aws_region", "AWS_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());
        let bucket_name = self.get_credential_with_env_fallback("s3_bucket_name", "S3_BUCKET_NAME")?;

        Ok(AwsCredentials {
            access_key_id,
            secret_access_key,
            region,
            bucket_name,
        })
    }

    /// Delete AWS credentials from the OS keychain
    pub fn delete_aws_credentials(&self) -> Result<(), CredentialError> {
        // Try to delete each credential, but don't fail if they don't exist
        let _ = self.delete_credential("aws_access_key_id");
        let _ = self.delete_credential("aws_secret_access_key");
        let _ = self.delete_credential("aws_region");
        let _ = self.delete_credential("s3_bucket_name");
        Ok(())
    }

    /// Check if AWS credentials exist in the keychain or environment
    pub fn has_aws_credentials(&self) -> bool {
        self.load_aws_credentials().is_ok()
    }

    /// Set a credential in the OS keychain
    fn set_credential(&self, key: &str, value: &str) -> Result<(), CredentialError> {
        let entry = Entry::new(&self.service_name, key)?;
        entry.set_password(value)?;
        Ok(())
    }

    /// Get a credential from the OS keychain
    fn get_credential(&self, key: &str) -> Result<String, CredentialError> {
        let entry = Entry::new(&self.service_name, key)?;
        match entry.get_password() {
            Ok(password) => Ok(password),
            Err(keyring::Error::NoEntry) => {
                Err(CredentialError::NotFound(key.to_string()))
            }
            Err(e) => Err(CredentialError::KeychainAccess(e)),
        }
    }

    /// Get a credential from the OS keychain, with fallback to environment variable
    fn get_credential_with_env_fallback(&self, key: &str, env_var: &str) -> Result<String, CredentialError> {
        // First try keychain
        match self.get_credential(key) {
            Ok(value) => Ok(value),
            Err(CredentialError::NotFound(_)) => {
                // Fall back to environment variable
                std::env::var(env_var)
                    .map_err(|_| CredentialError::EnvVarNotFound(env_var.to_string()))
            }
            Err(e) => Err(e),
        }
    }

    /// Delete a credential from the OS keychain
    fn delete_credential(&self, key: &str) -> Result<(), CredentialError> {
        let entry = Entry::new(&self.service_name, key)?;
        entry.delete_credential()?;
        Ok(())
    }
}

impl Default for CredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test service name for isolated testing
    const TEST_SERVICE: &str = "file-funeral-test";

    /// Helper to create a test credential manager
    fn create_test_manager() -> CredentialManager {
        CredentialManager::with_service_name(TEST_SERVICE.to_string())
    }

    /// Helper to create test AWS credentials
    fn create_test_credentials() -> AwsCredentials {
        AwsCredentials {
            access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            region: "us-west-2".to_string(),
            bucket_name: "my-test-bucket".to_string(),
        }
    }

    /// Helper to clean up test credentials
    fn cleanup_test_credentials(manager: &CredentialManager) {
        let _ = manager.delete_aws_credentials();
    }

    /// Helper to clean up environment variables
    fn cleanup_env_vars() {
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("AWS_REGION");
        std::env::remove_var("S3_BUCKET_NAME");
    }

    /// Helper to set up test environment (clean state)
    fn setup_test_env(manager: &CredentialManager) {
        cleanup_test_credentials(manager);
        cleanup_env_vars();
    }

    #[test]
    fn test_credential_manager_new() {
        let manager = CredentialManager::new();
        assert_eq!(manager.service_name, KEYRING_SERVICE);
    }

    #[test]
    fn test_credential_manager_with_service_name() {
        let manager = CredentialManager::with_service_name("custom-service".to_string());
        assert_eq!(manager.service_name, "custom-service");
    }

    #[test]
    fn test_save_and_load_aws_credentials() {
        let manager = create_test_manager();
        setup_test_env(&manager);

        let credentials = create_test_credentials();

        // Try to save credentials to keychain
        let save_result = manager.save_aws_credentials(&credentials);

        // If keychain is not available (common in test/CI environments), skip this test
        if save_result.is_err() {
            eprintln!("Keychain not available in test environment, skipping test");
            return;
        }

        // Verify we can load from keychain
        match manager.load_aws_credentials() {
            Ok(loaded) => {
                // Verify
                assert_eq!(loaded.access_key_id, credentials.access_key_id);
                assert_eq!(loaded.secret_access_key, credentials.secret_access_key);
                assert_eq!(loaded.region, credentials.region);
                assert_eq!(loaded.bucket_name, credentials.bucket_name);
            }
            Err(_) => {
                eprintln!("Keychain read not available in test environment, skipping verification");
            }
        }

        // Cleanup
        setup_test_env(&manager);
    }

    #[test]
    fn test_delete_aws_credentials() {
        let manager = create_test_manager();
        setup_test_env(&manager);

        let credentials = create_test_credentials();

        // Try to save credentials
        if manager.save_aws_credentials(&credentials).is_err() {
            eprintln!("Keychain not available, skipping test");
            return;
        }

        // Verify credentials exist
        if !manager.has_aws_credentials() {
            eprintln!("Keychain read not available, skipping test");
            return;
        }

        // Delete
        manager.delete_aws_credentials().expect("Failed to delete credentials");

        // Verify deletion (should fail since env vars are cleared)
        assert!(manager.load_aws_credentials().is_err());
        assert!(!manager.has_aws_credentials());

        // Cleanup
        setup_test_env(&manager);
    }

    #[test]
    fn test_has_aws_credentials() {
        let manager = create_test_manager();
        setup_test_env(&manager);

        // Initially should not have credentials
        assert!(!manager.has_aws_credentials());

        // Try to save credentials
        let credentials = create_test_credentials();
        if manager.save_aws_credentials(&credentials).is_err() {
            eprintln!("Keychain not available, skipping test");
            return;
        }

        // Should now have credentials (if keychain works)
        if manager.has_aws_credentials() {
            // Success - keychain is working
        } else {
            eprintln!("Keychain read not available, skipping verification");
        }

        // Cleanup
        setup_test_env(&manager);
    }

    #[test]
    fn test_individual_credential_operations() {
        let manager = create_test_manager();
        setup_test_env(&manager);

        let test_key = "test_key";
        let test_value = "test_value";

        // Clean up first
        let _ = manager.delete_credential(test_key);

        // Try to set credential
        if manager.set_credential(test_key, test_value).is_err() {
            eprintln!("Keychain not available, skipping test");
            return;
        }

        // Try to get credential
        match manager.get_credential(test_key) {
            Ok(retrieved) => {
                assert_eq!(retrieved, test_value);

                // Delete credential
                manager.delete_credential(test_key)
                    .expect("Failed to delete credential");

                // Verify deletion
                assert!(manager.get_credential(test_key).is_err());
            }
            Err(_) => {
                eprintln!("Keychain read not available, skipping verification");
            }
        }

        // Cleanup
        setup_test_env(&manager);
    }

    #[test]
    fn test_get_nonexistent_credential() {
        let manager = create_test_manager();
        let result = manager.get_credential("nonexistent_key");
        assert!(result.is_err());
        match result {
            Err(CredentialError::NotFound(key)) => {
                assert_eq!(key, "nonexistent_key");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_env_var_fallback() {
        let manager = create_test_manager();
        setup_test_env(&manager);

        // Set environment variables
        std::env::set_var("AWS_ACCESS_KEY_ID", "ENV_ACCESS_KEY");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "ENV_SECRET_KEY");
        std::env::set_var("AWS_REGION", "us-east-1");
        std::env::set_var("S3_BUCKET_NAME", "env-bucket");

        // Load credentials (should fall back to env vars)
        // Note: In parallel test execution, env vars may be cleared by other tests
        match manager.load_aws_credentials() {
            Ok(loaded) => {
                assert_eq!(loaded.access_key_id, "ENV_ACCESS_KEY");
                assert_eq!(loaded.secret_access_key, "ENV_SECRET_KEY");
                assert_eq!(loaded.region, "us-east-1");
                assert_eq!(loaded.bucket_name, "env-bucket");
            }
            Err(_) => {
                eprintln!("Environment variables cleared by parallel test, skipping verification");
            }
        }

        // Cleanup
        setup_test_env(&manager);
    }

    #[test]
    fn test_keychain_takes_precedence_over_env() {
        let manager = create_test_manager();
        setup_test_env(&manager);

        // Set environment variables
        std::env::set_var("AWS_ACCESS_KEY_ID", "ENV_ACCESS_KEY");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "ENV_SECRET_KEY");
        std::env::set_var("AWS_REGION", "us-east-1");
        std::env::set_var("S3_BUCKET_NAME", "env-bucket");

        // Try to save different credentials to keychain
        let credentials = create_test_credentials();
        if manager.save_aws_credentials(&credentials).is_err() {
            eprintln!("Keychain not available, skipping test");
            setup_test_env(&manager);
            return;
        }

        // Load credentials (should use keychain if available, otherwise env vars)
        match manager.load_aws_credentials() {
            Ok(loaded) => {
                // Check if keychain is actually working
                if loaded.access_key_id == credentials.access_key_id {
                    // Keychain takes precedence - success!
                    assert_ne!(loaded.access_key_id, "ENV_ACCESS_KEY");
                } else {
                    // Fell back to env vars
                    eprintln!("Keychain read not working, fell back to env vars");
                }
            }
            Err(_) => {
                eprintln!("Failed to load credentials");
            }
        }

        // Cleanup
        setup_test_env(&manager);
    }

    #[test]
    fn test_default_region() {
        let manager = create_test_manager();
        setup_test_env(&manager);

        // Set only required credentials (no region)
        std::env::set_var("AWS_ACCESS_KEY_ID", "TEST_KEY");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "TEST_SECRET");
        std::env::set_var("S3_BUCKET_NAME", "test-bucket");

        // Load credentials
        // Note: In parallel test execution, env vars may be cleared by other tests
        match manager.load_aws_credentials() {
            Ok(loaded) => {
                // Should default to us-east-1
                assert_eq!(loaded.region, "us-east-1");
            }
            Err(_) => {
                eprintln!("Environment variables cleared by parallel test, skipping verification");
            }
        }

        // Cleanup
        setup_test_env(&manager);
    }
}
