# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

file-funeral is a cloud-native file synchronization desktop application that enables users to backup and sync files between local storage and cloud providers. The primary use case is file sharing across multiple PCs with bidirectional synchronization.

**Key Features:**
- Upload/backup local files to cloud storage (AWS S3, GCS, S3-compatible)
- Download files from cloud to local
- Bidirectional synchronization between local and cloud
- Visual file list with sync status indicators
- Conflict resolution using "both save" strategy (Dropbox-style)

**Full Requirements**: See `REQUIREMENTS.md` for detailed specifications.

---

## Technology Stack

### Backend
- **Framework**: Tauri v2
- **Language**: Rust
- **Async Runtime**: Tokio
- **Cloud SDK**:
  - v1.0: `aws-sdk-s3`
  - v2.0+: S3-compatible, GCS SDK
- **Plugins**: `tauri-plugin-fs` (file system access)
- **Credential Storage**: `keyring` crate (OS keychain)

### Frontend
- **Build Tool**: Vite
- **Framework**: React / Vue / Svelte (TBD)
- **Language**: TypeScript

### Key Dependencies
```toml
[dependencies]
tauri = "2.x"
aws-config = { version = "1.1.7", features = ["behavior-version-latest"] }
aws-sdk-s3 = "1.117.0"
tokio = { version = "1", features = ["full"] }
tauri-plugin-fs = "*"
keyring = "*"
```

---

## Project Structure

Expected directory layout:
```
file-funeral/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── storage/     # Cloud storage abstraction
│   │   │   ├── mod.rs   # Storage trait definition
│   │   │   ├── s3.rs    # AWS S3 implementation
│   │   │   └── ...      # Future: GCS, S3-compatible
│   │   ├── sync/        # Sync engine
│   │   └── ...
│   └── Cargo.toml
├── src/                 # Frontend (React/Vue/Svelte)
├── REQUIREMENTS.md      # Detailed requirements
└── CLAUDE.md           # This file
```

---

## Architecture

### Storage Abstraction Layer

The app uses a trait-based abstraction for cloud storage providers:

```rust
trait CloudStorageProvider {
    async fn upload(&self, path: &str, data: &[u8], metadata: FileMetadata) -> Result<()>;
    async fn download(&self, path: &str) -> Result<Vec<u8>>;
    async fn list(&self, prefix: &str) -> Result<Vec<FileInfo>>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn get_metadata(&self, path: &str) -> Result<FileMetadata>;
}
```

**Implementations:**
- v1.0: `S3Provider` (AWS S3)
- v2.0: `S3CompatibleProvider` (MinIO, Backblaze B2, etc.)
- v2.5: `GCSProvider` (Google Cloud Storage)

### Sync Engine

**Sync Timing:**
- On app startup (automatic)
- When user clicks "Sync" button (manual)
- (Optional) On app exit

**Conflict Resolution:**
- Both files are saved when conflict detected
- Cloud version: `filename.txt`
- Local version: `filename (PCName's conflicted copy YYYY-MM-DD).txt`

**Deletion Handling:**
- v1.0: Deletions are synced (full sync)
- v2.0+: Soft delete with trash/restore functionality

---

## Development Commands

### Setup
```bash
# Install Tauri CLI
cargo install tauri-cli

# Install frontend dependencies
cd file-funeral
npm install  # or pnpm/yarn
```

### Development
```bash
# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Testing
```bash
# Run Rust tests
cd src-tauri
cargo test

# Run frontend tests
npm test
```

---

## Key Implementation Guidelines

### 1. File Synchronization
- Always use async operations with Tokio
- Implement proper error handling for network failures
- Use MD5/ETag for file integrity checks
- Preserve folder structure using S3 keys with full paths

### 2. Security
- **Never** store credentials in plain text
- Use OS keychain via `keyring` crate:
  - macOS: Keychain
  - Windows: Credential Manager
  - Linux: Secret Service API
- Support environment variables for development (`AWS_ACCESS_KEY_ID`, etc.)
- Enable S3 server-side encryption (SSE-S3) for all uploads

### 3. File Management
- **Preserved metadata**: filename, size, modified time, hash
- **Ignored metadata**: permissions, creation time, owner
- **Default exclusions**: `.DS_Store`, `Thumbs.db`, `*.tmp`, `*.swp`
- **Special files**: Skip symlinks, hard links (warn user)
- **Size limit**: 5GB max (v1.0), show progress bar for files >100MB

### 4. Error Handling
- Display user-friendly error messages
- Log detailed errors for troubleshooting
- Handle network timeouts and retries gracefully
- Never silently fail sync operations

### 5. Cross-Platform Considerations
- Use Tauri's path APIs for cross-platform path handling
- Test on Windows, macOS, and Linux
- Handle platform-specific hidden files appropriately

---

## Development Roadmap

### v1.0 (MVP) - AWS S3 Only
- [x] Requirements defined
- [ ] Project setup (Tauri + frontend)
- [ ] Storage trait + S3 implementation
- [ ] Basic sync engine
- [ ] File list UI with sync status
- [ ] Credentials management (keychain)
- [ ] Initial setup wizard

### v1.5 - Exclusion Patterns
- [ ] .gitignore-style exclusion rules
- [ ] Exclusion settings UI

### v2.0 - S3-Compatible + Soft Delete
- [ ] S3-compatible provider support
- [ ] Multi-provider selection UI
- [ ] Multiple folder selection
- [ ] Soft delete with trash

### v2.5 - Google Cloud Storage
- [ ] GCS provider implementation
- [ ] Google authentication flow

### v3.0 - Advanced Features
- [ ] Azure Blob Storage
- [ ] Client-side encryption
- [ ] App lock feature
- [ ] Symlink support

---

## Git Workflow

- Main branch: `main`
- Use feature branches for development
- Follow conventional commits
- Repository maintains clean history

---

## Resources

- **Requirements**: `REQUIREMENTS.md`
- **Tauri Docs**: https://v2.tauri.app/
- **AWS SDK for Rust**: https://github.com/awslabs/aws-sdk-rust
- **File System Plugin**: https://v2.tauri.app/plugin/file-system
