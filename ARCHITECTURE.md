# file-funeral アーキテクチャドキュメント

このドキュメントでは、file-funeralの設計、アーキテクチャ、技術的な実装の詳細を説明します。

## 目次

- [システム概要](#システム概要)
- [アーキテクチャ図](#アーキテクチャ図)
- [技術スタック](#技術スタック)
- [モジュール構成](#モジュール構成)
- [データフロー](#データフロー)
- [主要コンポーネント](#主要コンポーネント)
- [セキュリティ](#セキュリティ)
- [データ永続化](#データ永続化)
- [パフォーマンス最適化](#パフォーマンス最適化)
- [将来の拡張](#将来の拡張)

---

## システム概要

file-funeralは、Tauri v2フレームワークを使用したハイブリッドデスクトップアプリケーションです。

### 設計原則

1. **セキュリティファースト**: 認証情報の暗号化保存、AWS S3のサーバーサイド暗号化
2. **シンプルな操作性**: Dropbox風のUIとワークフロー
3. **拡張性**: トレイトベースの抽象化により、将来的に複数のクラウドプロバイダーをサポート
4. **クロスプラットフォーム**: TauriによりWindows、macOS、Linuxをサポート（v1.0はmacOS優先）
5. **オフライン対応**: SQLiteによる同期履歴の永続化

### 主要機能

- **双方向ファイル同期**: ローカル ↔ AWS S3
- **競合解決**: 両方保存方式（Dropboxスタイル）
- **削除同期**: 同期履歴に基づく削除検出
- **自動同期**: アプリ起動時・終了時の自動同期
- **同期履歴管理**: SQLiteによる永続化

---

## アーキテクチャ図

### システムアーキテクチャ

```
┌─────────────────────────────────────────────────────────────┐
│                      Frontend (React)                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ MainPage.tsx │  │SettingsPage  │  │ Components   │       │
│  │              │  │.tsx          │  │ (UI)         │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
│         │                 │                 │                │
│         └─────────────────┴─────────────────┘                │
│                           │                                  │
│                    @tauri-apps/api                           │
│                     invoke('command')                        │
└───────────────────────────┬─────────────────────────────────┘
                            │ IPC (Inter-Process Communication)
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   Backend (Rust + Tauri)                     │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Tauri Commands (commands.rs)            │    │
│  │  - set_credentials()    - sync_files()              │    │
│  │  - test_connection()    - get_sync_status()         │    │
│  │  - get_credentials()    - save_sync_settings()      │    │
│  │  - list_files()         - get_sync_settings()       │    │
│  └────┬────────┬────────┬────────┬────────┬────────────┘    │
│       │        │        │        │        │                  │
│       ▼        ▼        ▼        ▼        ▼                  │
│  ┌────────┐ ┌────┐ ┌──────┐ ┌──────┐ ┌──────┐              │
│  │ auth/  │ │sync│ │storage│ │ db/  │ │retry/│              │
│  │ module │ │    │ │       │ │      │ │      │              │
│  └────┬───┘ └─┬──┘ └───┬──┘ └───┬──┘ └──┬───┘              │
│       │       │        │        │       │                    │
│       ▼       ▼        ▼        ▼       ▼                    │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐              │
│  │keyring│ │chrono│ │aws-sdk│ │rusqlite│ │backon│           │
│  │(OS)   │ │      │ │-s3    │ │(SQLite)│ │      │           │
│  └───┬───┘ └──────┘ └───┬──┘ └───┬────┘ └──────┘           │
│      │                  │        │                            │
└──────┼──────────────────┼────────┼───────────────────────────┘
       │                  │        │
       ▼                  ▼        ▼
┌──────────┐        ┌─────────┐ ┌──────────────┐
│ macOS    │        │ AWS S3  │ │ SQLite DB    │
│ Keychain │        │ Bucket  │ │ (Local)      │
└──────────┘        └─────────┘ └──────────────┘
```

### データフロー（同期処理）

```
User clicks "Sync" button
         │
         ▼
┌─────────────────────┐
│ Frontend (React)    │
│ SyncButton.tsx      │
└─────────┬───────────┘
          │ invoke('sync_files', {...})
          ▼
┌─────────────────────────────────────────────┐
│ Backend: commands::sync_files()             │
│                                             │
│  1. Validate inputs                         │
│  2. Load credentials from Keychain          │
│  3. Create S3Provider                       │
│  4. Get sync status (compare files)         │
│     ├─ List local files                     │
│     ├─ List remote files (S3)               │
│     ├─ Load last sync history (SQLite)      │
│     └─ Detect deletions & conflicts         │
│  5. Sync each file based on state           │
│     ├─ NeedsUpload → upload to S3           │
│     ├─ NeedsDownload → download from S3     │
│     ├─ Conflict → both-save resolution      │
│     ├─ PendingLocalDeletion → delete S3     │
│     └─ PendingRemoteDeletion → delete local │
│  6. Save sync history to SQLite             │
│  7. Return SyncFilesResponse                │
└─────────┬───────────────────────────────────┘
          │ Result<SyncFilesResponse>
          ▼
┌─────────────────────┐
│ Frontend            │
│ Display results     │
└─────────────────────┘
```

---

## 技術スタック

### Backend (Rust)

| カテゴリ | クレート | バージョン | 用途 |
|---------|---------|-----------|------|
| **Framework** | tauri | 2.9.5 | デスクトップアプリフレームワーク |
| **Cloud SDK** | aws-sdk-s3 | 1.117.0 | AWS S3統合 |
| | aws-config | 1.1.7 | AWS設定管理 |
| **Async Runtime** | tokio | 1.x | 非同期ランタイム |
| **Database** | rusqlite | 0.32 | SQLiteデータベース |
| | r2d2 | 0.8 | コネクションプール |
| | r2d2_sqlite | 0.25 | SQLiteプール |
| **Security** | keyring | 3 | OS認証情報ストレージ |
| **Error Handling** | anyhow | 1.0 | 柔軟なエラーハンドリング |
| | thiserror | 2.0 | カスタムエラー型 |
| **Utilities** | chrono | 0.4 | 日時処理 |
| | serde | 1.0 | シリアライゼーション |
| | serde_json | 1.0 | JSON処理 |
| | md5 | 0.7 | MD5ハッシュ計算 |
| | hostname | 0.4 | ホスト名取得 |
| **Retry Logic** | backon | 1.2 | リトライと指数バックオフ |
| **Logging** | log | 0.4 | ロギングファサード |
| | env_logger | 0.11 | ログ設定 |
| **Tauri Plugins** | tauri-plugin-log | 2 | Tauriログプラグイン |
| | tauri-plugin-fs | 2 | ファイルシステム |
| | tauri-plugin-dialog | 2 | ダイアログ |

### Frontend (React)

| カテゴリ | パッケージ | バージョン | 用途 |
|---------|-----------|-----------|------|
| **Framework** | react | 19.2.3 | UIフレームワーク |
| | react-dom | 19.2.3 | DOM操作 |
| **Build Tool** | vite | 7.3.0 | 高速ビルドツール |
| **Language** | typescript | 5.9.3 | 型安全性 |
| **UI Components** | @radix-ui/* | 1.x | アクセシブルなUIコンポーネント |
| | lucide-react | 0.562.0 | アイコンライブラリ |
| **Styling** | tailwindcss | 4.1.18 | ユーティリティファーストCSS |
| **Tauri Bridge** | @tauri-apps/api | 2.9.1 | Tauri API |
| | @tauri-apps/plugin-dialog | 2.4.2 | ファイルダイアログ |
| **State Management** | React Hooks | - | ローカル状態管理 |
| **Notifications** | sonner | 2.0.7 | トースト通知 |

---

## モジュール構成

### Backend (src-tauri/src/)

```
src-tauri/src/
├── main.rs                 # エントリポイント（minimal）
├── lib.rs                  # メインライブラリ、Tauri setup
├── commands.rs             # Tauri Commands（フロントエンドとの通信層）
│
├── auth/                   # 認証情報管理
│   └── mod.rs              # CredentialManager、keyring統合
│
├── storage/                # クラウドストレージ抽象化層
│   ├── mod.rs              # CloudStorageProvider trait、FileInfo/FileMetadata
│   ├── s3.rs               # S3Provider実装
│   └── metadata.rs         # メタデータ処理（MD5ハッシュ計算など）
│
├── sync/                   # 同期エンジン
│   ├── mod.rs              # ファイル比較、削除検出ロジック
│   └── conflict.rs         # 競合解決（両方保存方式）
│
├── db/                     # データベース（SQLite）
│   ├── mod.rs              # データベース初期化、DbPool
│   ├── sync_history.rs     # 同期履歴の保存・読み込み
│   └── settings.rs         # アプリ設定の永続化
│
├── retry/                  # リトライロジック
│   └── mod.rs              # backonを使用した指数バックオフ
│
└── logging/                # ログ設定
    └── mod.rs              # env_logger設定
```

### Frontend (src/)

```
src/
├── main.tsx                # Reactエントリポイント
├── App.tsx                 # ルートコンポーネント
│
├── pages/                  # ページコンポーネント
│   ├── MainPage.tsx        # メイン画面（ファイルリスト + 同期）
│   └── SettingsPage.tsx    # 設定画面（認証情報 + 同期設定）
│
├── components/             # 再利用可能なコンポーネント
│   ├── FileList.tsx        # ファイルリスト表示
│   ├── SyncButton.tsx      # 同期ボタン + プログレスバー
│   ├── SetupWizard.tsx     # 初回セットアップウィザード
│   └── ui/                 # shadcn/uiコンポーネント
│       ├── button.tsx
│       ├── dialog.tsx
│       ├── progress.tsx
│       └── ...
│
└── types/                  # TypeScript型定義
    └── index.ts            # Tauri Commands のレスポンス型
```

---

## データフロー

### 1. 認証情報の設定

```
User inputs credentials
      │
      ▼
Frontend: SettingsPage.tsx
      │ invoke('set_credentials', {
      │   access_key_id,
      │   secret_access_key,
      │   region,
      │   bucket_name
      │ })
      ▼
Backend: commands::set_credentials()
      │
      ├─> auth::CredentialManager::save_aws_credentials()
      │   └─> keyring::set_password() → macOS Keychain
      │
      └─> Return CredentialsResponse { success: true }
      │
      ▼
Frontend: Display success notification
```

### 2. 同期ステータスの取得

```
User opens MainPage
      │
      ▼
Frontend: useEffect()
      │ invoke('get_sync_status', {
      │   local_path,
      │   remote_prefix
      │ })
      ▼
Backend: commands::get_sync_status()
      │
      ├─> List local files (list_local_files)
      │   └─> Walk directory recursively
      │       └─> Skip hidden files, .DS_Store, etc.
      │       └─> Calculate MD5 hash for each file
      │
      ├─> List remote files (S3Provider::list)
      │   └─> aws_sdk_s3::list_objects_v2()
      │
      ├─> Load last sync history (SQLite)
      │   └─> db::sync_history::get_last_synced_files()
      │
      ├─> Detect deletions
      │   └─> sync::detect_deletions()
      │       ├─> Local deletions (was synced, not local, still remote)
      │       └─> Remote deletions (was synced, not remote, still local)
      │
      └─> Compare files
          └─> sync::compare_files_with_deletion()
              ├─> InSync: same ETag or modified time
              ├─> NeedsUpload: local newer or remote missing
              ├─> NeedsDownload: remote newer or local missing
              ├─> Conflict: both modified since last sync
              ├─> PendingLocalDeletion: deleted locally
              └─> PendingRemoteDeletion: deleted remotely
      │
      └─> Return SyncStatusResponse
      │
      ▼
Frontend: Display file list with sync states
```

### 3. ファイルの同期

```
User clicks "Sync" button
      │
      ▼
Frontend: SyncButton.tsx
      │ invoke('sync_files', {
      │   local_path,
      │   remote_prefix
      │ })
      ▼
Backend: commands::sync_files()
      │
      ├─> Get sync status (same as above)
      │
      ├─> For each file:
      │   │
      │   ├─ NeedsUpload:
      │   │  └─> Read local file
      │   │  └─> S3Provider::upload()
      │   │      └─> aws_sdk_s3::put_object()
      │   │
      │   ├─ NeedsDownload:
      │   │  └─> S3Provider::download()
      │   │      └─> aws_sdk_s3::get_object()
      │   │  └─> Write to local file
      │   │
      │   ├─ Conflict:
      │   │  └─> sync::resolve_conflict()
      │   │      ├─> Download remote → original path
      │   │      ├─> Save local → "conflicted copy" path
      │   │      └─> Upload conflicted copy to S3
      │   │
      │   ├─ PendingLocalDeletion:
      │   │  └─> S3Provider::delete()
      │   │      └─> aws_sdk_s3::delete_object()
      │   │
      │   └─ PendingRemoteDeletion:
      │      └─> fs::remove_file() (local)
      │
      ├─> Save sync history to SQLite
      │   └─> db::sync_history::save_sync_history()
      │       ├─> INSERT INTO sync_history
      │       └─> INSERT INTO synced_files (for each file)
      │
      └─> Return SyncFilesResponse
      │
      ▼
Frontend: Display sync results
```

---

## 主要コンポーネント

### 1. Storage抽象化層

**目的**: クラウドストレージプロバイダーを抽象化し、将来的に複数のプロバイダーをサポート

**設計**:
```rust
// src-tauri/src/storage/mod.rs

#[async_trait]
pub trait CloudStorageProvider: Send + Sync {
    async fn upload(&self, path: &str, data: &[u8], metadata: FileMetadata)
        -> Result<(), StorageError>;
    async fn download(&self, path: &str) -> Result<Vec<u8>, StorageError>;
    async fn list(&self, prefix: &str) -> Result<Vec<FileInfo>, StorageError>;
    async fn delete(&self, path: &str) -> Result<(), StorageError>;
    async fn get_metadata(&self, path: &str) -> Result<FileMetadata, StorageError>;
}
```

**実装**:
- `S3Provider` (v1.0): AWS S3実装
- `S3CompatibleProvider` (v2.0 計画): MinIO、Backblaze B2対応
- `GCSProvider` (v2.5 計画): Google Cloud Storage対応

### 2. 同期エンジン

**目的**: ローカルとリモートのファイルを比較し、同期状態を判定

**主要関数**:
```rust
// src-tauri/src/sync/mod.rs

/// ファイルの同期状態
pub enum SyncState {
    InSync,                  // 同期済み
    NeedsUpload,             // アップロード必要
    NeedsDownload,           // ダウンロード必要
    Conflict,                // 競合あり
    PendingLocalDeletion,    // ローカル削除待ち（リモートから削除）
    PendingRemoteDeletion,   // リモート削除待ち（ローカルから削除）
}

/// ファイルを比較して同期状態を判定（削除対応版）
pub fn compare_files_with_deletion(
    local: Option<&FileInfo>,
    remote: Option<&FileInfo>,
    last_sync_time: Option<DateTime<Utc>>,
    was_previously_synced: bool,
) -> ComparisonResult {
    // ロジック:
    // 1. 両方存在 → ETag比較で InSync / NeedsUpload / NeedsDownload / Conflict
    // 2. ローカルのみ存在 → 新規ファイルまたはリモート削除
    // 3. リモートのみ存在 → 新規ファイルまたはローカル削除
    // 4. 両方不在 → エラー（このケースは発生しない）
}

/// 削除を検出
pub fn detect_deletions(
    local_files: &[FileInfo],
    remote_files: &[FileInfo],
    last_synced_files: &HashSet<String>,
) -> DeletionResult {
    // 前回同期時に存在したが、今は存在しないファイルを検出
}
```

### 3. 競合解決

**目的**: ローカルとリモートの両方でファイルが変更された場合の競合を解決

**戦略**: 両方保存方式（Dropboxスタイル）
- クラウド版: 元のファイル名
- ローカル版: `filename (PC名's conflicted copy YYYY-MM-DD).ext`

```rust
// src-tauri/src/sync/conflict.rs

pub struct ConflictResolution {
    pub original_path: String,
    pub conflicted_copy_path: String,
}

pub fn resolve_conflict(original_path: &str) -> Result<ConflictResolution> {
    // 1. ホスト名を取得（例: "MacBook-Pro"）
    // 2. 現在の日付を取得（例: "2024-01-15"）
    // 3. 競合ファイル名を生成
    //    例: "document.txt" → "document (MacBook-Pro's conflicted copy 2024-01-15).txt"
}
```

### 4. 認証情報管理

**目的**: AWS認証情報を安全に保存・取得

**実装**:
```rust
// src-tauri/src/auth/mod.rs

pub struct CredentialManager {
    service_name: String,  // "file-funeral" (macOS Keychain のサービス名)
}

impl CredentialManager {
    /// 認証情報をOSキーチェーンに保存
    pub fn save_aws_credentials(&self, creds: &AwsCredentials)
        -> Result<(), CredentialError> {
        // keyring::set_password() を使用
        // macOS: Keychain, Windows: Credential Manager, Linux: Secret Service
    }

    /// 認証情報をOSキーチェーンから読み込み
    pub fn load_aws_credentials(&self) -> Result<AwsCredentials, CredentialError> {
        // keyring::get_password() を使用
    }
}
```

### 5. データベース（SQLite）

**目的**: 同期履歴とアプリ設定を永続化

**スキーマ**:
```sql
-- sync_history テーブル（同期操作の履歴）
CREATE TABLE sync_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sync_started_at TEXT NOT NULL,
    sync_completed_at TEXT NOT NULL,
    local_path TEXT NOT NULL,
    remote_prefix TEXT NOT NULL,
    files_uploaded INTEGER NOT NULL,
    files_downloaded INTEGER NOT NULL,
    files_deleted INTEGER NOT NULL,
    conflicts_resolved INTEGER NOT NULL,
    success INTEGER NOT NULL,  -- 0 or 1
    error_message TEXT
);

-- synced_files テーブル（最終同期時のファイル一覧）
CREATE TABLE synced_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sync_history_id INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    last_modified TEXT NOT NULL,
    etag TEXT,
    was_local INTEGER NOT NULL,
    was_remote INTEGER NOT NULL,
    FOREIGN KEY (sync_history_id) REFERENCES sync_history(id)
);

-- sync_settings テーブル（アプリ設定）
CREATE TABLE sync_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- 常に1行のみ
    local_path TEXT NOT NULL,
    remote_prefix TEXT NOT NULL,
    auto_sync_on_startup INTEGER NOT NULL,
    auto_sync_on_shutdown INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### 6. リトライロジック

**目的**: ネットワークエラー時の自動リトライ

**実装**:
```rust
// src-tauri/src/retry/mod.rs

use backon::{ExponentialBuilder, Retryable};

/// S3操作にリトライを適用
pub async fn with_retry<F, Fut, T>(operation: F) -> Result<T, StorageError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, StorageError>>,
{
    let retry_strategy = ExponentialBuilder::default()
        .with_max_times(3)           // 最大3回リトライ
        .with_min_delay(Duration::from_millis(100))
        .with_max_delay(Duration::from_secs(5));

    operation.retry(retry_strategy).await
}
```

---

## セキュリティ

### 1. 認証情報の保護

- **保存**: OS keychain使用（macOS Keychain）
- **メモリ**: String型で保持、自動的にdrop時にクリア
- **ログ**: シークレットアクセスキーは**絶対に**ログに出力しない
- **転送**: HTTPS経由でAWS S3と通信

### 2. データ暗号化

- **転送時**: HTTPS（TLS 1.2+）
- **保存時**: AWS S3サーバーサイド暗号化（SSE-S3）デフォルト有効
- **将来**: クライアントサイド暗号化（v3.0計画）

### 3. アクセス制御

- **IAM最小権限**: 必要最小限のS3権限のみ
- **バケットポリシー**: パブリックアクセスをブロック
- **認証情報のスコープ**: 特定のS3バケットのみアクセス可能

---

## データ永続化

### SQLiteデータベース

**場所**:
- 本番: `~/.file-funeral/sync_history.db`
- テスト: `/tmp/.file-funeral-test-{PID}/sync_history.db`

**コネクションプール**:
- `r2d2`を使用してコネクションを管理
- プールサイズ: 最大10コネクション

**トランザクション**:
- 同期履歴の保存は1トランザクション内で実行
- ロールバックにより、部分的な保存を防ぐ

---

## パフォーマンス最適化

### 1. ファイルハッシュ計算

**小さいファイル（< 100MB）**:
```rust
// 一度にメモリに読み込んで計算
let data = fs::read(&path)?;
let hash = format!("{:x}", md5::compute(&data));
```

**大きいファイル（>= 100MB）**:
```rust
// バッファリングして計算
let mut file = File::open(&path)?;
let mut context = md5::Context::new();
let mut buffer = [0; 8192];

loop {
    let n = file.read(&mut buffer)?;
    if n == 0 { break; }
    context.consume(&buffer[..n]);
}

let hash = format!("{:x}", context.compute());
```

### 2. 並列処理

**v1.0**: 逐次処理（シンプル、信頼性優先）
**v1.5+**: 並列アップロード/ダウンロード（Tokio parallel tasks）

### 3. リトライと指数バックオフ

- 初回: 即座にリトライ
- 2回目: 100ms待機
- 3回目: 1秒待機
- 最大: 5秒待機

---

## 将来の拡張

### v1.5 - 除外パターン
```rust
// .gitignore方式の除外ルール
// src-tauri/src/sync/exclusion.rs
pub struct ExclusionRules {
    patterns: Vec<Glob>,
}
```

### v2.0 - マルチプロバイダー
```rust
// 複数のクラウドプロバイダーを同時サポート
pub enum ProviderType {
    AwsS3,
    S3Compatible { endpoint: String },
    GoogleCloudStorage,
}
```

### v3.0 - クライアントサイド暗号化
```rust
// アップロード前にファイルを暗号化
pub struct EncryptedStorageProvider<P: CloudStorageProvider> {
    inner: P,
    encryption_key: EncryptionKey,
}
```

---

## 参考資料

- [Tauri Documentation](https://v2.tauri.app/)
- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust)
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [Rust Async Book](https://rust-lang.github.io/async-book/)

---

開発に関する質問は、[GitHub Issues](https://github.com/ohr486/file-funeral/issues)で受け付けています。
