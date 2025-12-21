//! File metadata management module
//!
//! This module provides functionality for:
//! - Calculating MD5 checksums of files (compatible with AWS S3 ETag)
//! - Extracting file metadata (size, modification time, checksum)
//! - Comparing file metadata to determine sync requirements

use chrono::{DateTime, Utc};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use super::{FileInfo, FileMetadata, Result, StorageError};

/// Calculate MD5 checksum of a file
///
/// This function reads the entire file and computes its MD5 checksum.
/// MD5 is used to match AWS S3 ETag format for file integrity checking.
///
/// # Arguments
/// * `file_path` - Path to the file
///
/// # Returns
/// * `Ok(String)` - Hexadecimal MD5 checksum string (32 characters)
/// * `Err(StorageError)` - If file cannot be read
///
/// # Example
/// ```no_run
/// use app_lib::storage::metadata::calculate_md5_hash;
///
/// let hash = calculate_md5_hash("/path/to/file.txt").unwrap();
/// println!("MD5: {}", hash);
/// ```
pub fn calculate_md5_hash<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let path = file_path.as_ref();
    let mut file = fs::File::open(path).map_err(|e| {
        StorageError::Io(io::Error::new(
            e.kind(),
            format!(
                "Failed to open file for MD5 calculation: {}",
                path.display()
            ),
        ))
    })?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| {
        StorageError::Io(io::Error::new(
            e.kind(),
            format!(
                "Failed to read file for MD5 calculation: {}",
                path.display()
            ),
        ))
    })?;

    let digest = md5::compute(&buffer);
    Ok(format!("{:x}", digest))
}

/// Calculate MD5 checksum of a file using buffered reading
///
/// This is more memory-efficient for large files as it reads the file
/// in chunks rather than loading the entire file into memory.
///
/// # Arguments
/// * `file_path` - Path to the file
///
/// # Returns
/// * `Ok(String)` - Hexadecimal MD5 checksum string (32 characters)
/// * `Err(StorageError)` - If file cannot be read
pub fn calculate_md5_hash_buffered<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let path = file_path.as_ref();
    let mut file = fs::File::open(path).map_err(|e| {
        StorageError::Io(io::Error::new(
            e.kind(),
            format!(
                "Failed to open file for buffered MD5 calculation: {}",
                path.display()
            ),
        ))
    })?;

    let mut context = md5::Context::new();
    let mut buffer = [0u8; 8192]; // 8KB buffer

    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| {
            StorageError::Io(io::Error::new(
                e.kind(),
                format!(
                    "Failed to read file for buffered MD5 calculation: {}",
                    path.display()
                ),
            ))
        })?;

        if bytes_read == 0 {
            break;
        }

        context.consume(&buffer[..bytes_read]);
    }

    let digest = context.compute();
    Ok(format!("{:x}", digest))
}

/// Get file metadata from local filesystem
///
/// This function extracts metadata from a local file, including:
/// - File size
/// - Last modified time
/// - MD5 checksum
///
/// For files larger than 100MB, it uses buffered MD5 calculation
/// to avoid loading the entire file into memory.
///
/// # Arguments
/// * `file_path` - Path to the local file
///
/// # Returns
/// * `Ok(FileMetadata)` - File metadata
/// * `Err(StorageError)` - If file cannot be accessed or metadata cannot be read
pub fn get_local_file_metadata<P: AsRef<Path>>(file_path: P) -> Result<FileMetadata> {
    let path = file_path.as_ref();

    // Get file metadata from filesystem
    let metadata = fs::metadata(path).map_err(|e| {
        StorageError::Metadata(format!(
            "Failed to get file metadata for {}: {}",
            path.display(),
            e
        ))
    })?;

    // Extract file size
    let size = metadata.len();

    // Extract last modified time
    let modified = metadata.modified().map_err(|e| {
        StorageError::Metadata(format!(
            "Failed to get modification time for {}: {}",
            path.display(),
            e
        ))
    })?;

    let last_modified: DateTime<Utc> = modified.into();

    // Calculate MD5 checksum
    // Use buffered calculation for files larger than 100MB
    let hash = if size > 100 * 1024 * 1024 {
        calculate_md5_hash_buffered(path)?
    } else {
        calculate_md5_hash(path)?
    };

    Ok(FileMetadata::new(
        size,
        last_modified,
        None, // content_type is not determined from local files
        Some(hash),
    ))
}

/// Get FileInfo for a local file
///
/// This is a convenience function that creates a FileInfo struct
/// from a local file path and its metadata.
///
/// # Arguments
/// * `file_path` - Path to the local file (can be relative or absolute)
/// * `relative_path` - The relative path to store in FileInfo (used for comparison)
///
/// # Returns
/// * `Ok(FileInfo)` - File information
/// * `Err(StorageError)` - If file cannot be accessed
pub fn get_local_file_info<P: AsRef<Path>>(
    file_path: P,
    relative_path: String,
) -> Result<FileInfo> {
    let metadata = get_local_file_metadata(&file_path)?;

    Ok(FileInfo::new(
        relative_path,
        metadata.size,
        metadata.last_modified,
        metadata.etag,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_md5_hash() {
        // Create a temporary file with known content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let hash = calculate_md5_hash(temp_file.path()).unwrap();

        // MD5 checksum of "Hello, World!" is "65a8e27d8879283831b664bd8b7f0ad4"
        assert_eq!(hash, "65a8e27d8879283831b664bd8b7f0ad4");
    }

    #[test]
    fn test_calculate_md5_hash_buffered() {
        // Create a temporary file with known content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let hash = calculate_md5_hash_buffered(temp_file.path()).unwrap();

        // Should produce the same hash as non-buffered version
        assert_eq!(hash, "65a8e27d8879283831b664bd8b7f0ad4");
    }

    #[test]
    fn test_calculate_md5_hash_large_content() {
        // Create a file with content larger than the buffer size (8KB)
        let mut temp_file = NamedTempFile::new().unwrap();
        let large_content = vec![b'A'; 20_000]; // 20KB
        temp_file.write_all(&large_content).unwrap();
        temp_file.flush().unwrap();

        let hash_normal = calculate_md5_hash(temp_file.path()).unwrap();
        let hash_buffered = calculate_md5_hash_buffered(temp_file.path()).unwrap();

        // Both methods should produce the same hash
        assert_eq!(hash_normal, hash_buffered);
    }

    #[test]
    fn test_calculate_md5_hash_empty_file() {
        // Create an empty temporary file
        let temp_file = NamedTempFile::new().unwrap();

        let hash = calculate_md5_hash(temp_file.path()).unwrap();

        // MD5 checksum of empty file is "d41d8cd98f00b204e9800998ecf8427e"
        assert_eq!(hash, "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn test_calculate_md5_hash_nonexistent_file() {
        let result = calculate_md5_hash("/nonexistent/file.txt");
        assert!(result.is_err());

        match result {
            Err(StorageError::Io(_)) => {}
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_get_local_file_metadata() {
        // Create a temporary file with known content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Test content").unwrap();
        temp_file.flush().unwrap();

        let metadata = get_local_file_metadata(temp_file.path()).unwrap();

        assert_eq!(metadata.size, 12); // "Test content" is 12 bytes
        assert!(metadata.etag.is_some());
        assert_eq!(metadata.etag.unwrap(), "8bfa8e0684108f419933a5995264d150"); // MD5 of "Test content"
        assert!(metadata.last_modified <= Utc::now());
    }

    #[test]
    fn test_get_local_file_metadata_large_file() {
        // Create a file larger than 100MB threshold
        let mut temp_file = NamedTempFile::new().unwrap();
        let chunk = vec![b'X'; 1_048_576]; // 1MB chunk

        // Write 101MB
        for _ in 0..101 {
            temp_file.write_all(&chunk).unwrap();
        }
        temp_file.flush().unwrap();

        let metadata = get_local_file_metadata(temp_file.path()).unwrap();

        assert_eq!(metadata.size, 101 * 1_048_576);
        assert!(metadata.etag.is_some());
    }

    #[test]
    fn test_get_local_file_info() {
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"File info test").unwrap();
        temp_file.flush().unwrap();

        let file_info = get_local_file_info(temp_file.path(), "test/file.txt".to_string()).unwrap();

        assert_eq!(file_info.path, "test/file.txt");
        assert_eq!(file_info.size, 14); // "File info test" is 14 bytes
        assert!(file_info.etag.is_some());
        assert!(file_info.last_modified <= Utc::now());
    }

    #[test]
    fn test_hash_consistency() {
        // Create two files with identical content
        let mut temp_file1 = NamedTempFile::new().unwrap();
        let mut temp_file2 = NamedTempFile::new().unwrap();

        let content = b"Identical content";
        temp_file1.write_all(content).unwrap();
        temp_file2.write_all(content).unwrap();
        temp_file1.flush().unwrap();
        temp_file2.flush().unwrap();

        let hash1 = calculate_md5_hash(temp_file1.path()).unwrap();
        let hash2 = calculate_md5_hash(temp_file2.path()).unwrap();

        // Files with identical content should have identical hashes
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_content() {
        // Create two files with different content
        let mut temp_file1 = NamedTempFile::new().unwrap();
        let mut temp_file2 = NamedTempFile::new().unwrap();

        temp_file1.write_all(b"Content A").unwrap();
        temp_file2.write_all(b"Content B").unwrap();
        temp_file1.flush().unwrap();
        temp_file2.flush().unwrap();

        let hash1 = calculate_md5_hash(temp_file1.path()).unwrap();
        let hash2 = calculate_md5_hash(temp_file2.path()).unwrap();

        // Files with different content should have different hashes
        assert_ne!(hash1, hash2);
    }
}
