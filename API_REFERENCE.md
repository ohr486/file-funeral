# file-funeral API リファレンス

このドキュメントでは、file-funeralのTauri Commands APIを詳細に説明します。すべてのコマンドはフロントエンド（React）からバックエンド（Rust）を呼び出すために使用されます。

## 目次

- [概要](#概要)
- [認証情報管理](#認証情報管理)
  - [set_credentials](#set_credentials)
  - [get_credentials](#get_credentials)
  - [test_connection](#test_connection)
- [ファイル操作](#ファイル操作)
  - [list_files](#list_files)
  - [sync_files](#sync_files)
  - [get_sync_status](#get_sync_status)
- [設定管理](#設定管理)
  - [save_sync_settings](#save_sync_settings)
  - [get_sync_settings](#get_sync_settings)
- [エラーハンドリング](#エラーハンドリング)
- [型定義](#型定義)

---

## 概要

### 使用方法

すべてのTauri Commandsは、フロントエンドから以下のように呼び出します:

```typescript
import { invoke } from '@tauri-apps/api/core';

// 例: 認証情報の設定
const response = await invoke<CredentialsResponse>('set_credentials', {
  request: {
    access_key_id: 'AKIAIOSFODNN7EXAMPLE',
    secret_access_key: 'wJalrXUtnFEMI/K7MDENG/...',
    region: 'ap-northeast-1',
    bucket_name: 'my-bucket'
  }
});
```

### エラーハンドリング

すべてのコマンドは `Result<T, CommandError>` を返します。エラーが発生した場合、Promiseがrejectされます:

```typescript
try {
  const response = await invoke('sync_files', { request });
  console.log('Success:', response);
} catch (error) {
  console.error('Error:', error);
}
```

---

## 認証情報管理

### set_credentials

AWS認証情報をmacOSキーチェーンに保存します。

**シグネチャ**:
```rust
#[tauri::command]
pub async fn set_credentials(
    request: SetCredentialsRequest
) -> CommandResult<CredentialsResponse>
```

**パラメータ**:

| 名前 | 型 | 説明 | 必須 |
|------|-----|------|------|
| `access_key_id` | `string` | AWSアクセスキーID | ✅ |
| `secret_access_key` | `string` | AWSシークレットアクセスキー | ✅* |
| `region` | `string` | AWSリージョン（例: `ap-northeast-1`） | ✅ |
| `bucket_name` | `string` | S3バケット名 | ✅ |

*注: `secret_access_key` が空の場合、既存の値を使用します（更新時のみ）。

**リクエスト例**:
```typescript
const request: SetCredentialsRequest = {
  access_key_id: 'AKIAIOSFODNN7EXAMPLE',
  secret_access_key: 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY',
  region: 'us-west-2',
  bucket_name: 'my-file-funeral-backup'
};

const response = await invoke<CredentialsResponse>('set_credentials', { request });
```

**レスポンス**:
```typescript
interface CredentialsResponse {
  success: boolean;        // 成功: true、失敗: false
  message: string;         // 結果メッセージ
}
```

**レスポンス例**:
```json
{
  "success": true,
  "message": "Credentials saved successfully"
}
```

**エラー**:
- `InvalidInput`: パラメータが空または無効
- `Credential`: キーチェーンへの保存に失敗

**実装場所**: `src-tauri/src/commands.rs:200`

---

### get_credentials

保存されているAWS認証情報を取得します（シークレットアクセスキーを除く）。

**シグネチャ**:
```rust
#[tauri::command]
pub async fn get_credentials() -> CommandResult<GetCredentialsResponse>
```

**パラメータ**: なし

**呼び出し例**:
```typescript
const response = await invoke<GetCredentialsResponse>('get_credentials');
```

**レスポンス**:
```typescript
interface GetCredentialsResponse {
  has_credentials: boolean;           // 認証情報が存在するか
  access_key_id: string | null;       // アクセスキーID
  region: string | null;               // リージョン
  bucket_name: string | null;          // バケット名
  // secret_access_key は含まれません（セキュリティ上の理由）
}
```

**レスポンス例**:
```json
{
  "has_credentials": true,
  "access_key_id": "AKIAIOSFODNN7EXAMPLE",
  "region": "ap-northeast-1",
  "bucket_name": "my-bucket"
}
```

**エラー**: なし（認証情報がない場合は `has_credentials: false`）

**実装場所**: `src-tauri/src/commands.rs:361`

---

### test_connection

保存されている認証情報を使用してAWS S3への接続をテストします。

**シグネチャ**:
```rust
#[tauri::command]
pub async fn test_connection() -> CommandResult<ConnectionTestResponse>
```

**パラメータ**: なし

**呼び出し例**:
```typescript
const response = await invoke<ConnectionTestResponse>('test_connection');
```

**レスポンス**:
```typescript
interface ConnectionTestResponse {
  connected: boolean;              // 接続成功: true、失敗: false
  message: string;                 // 結果メッセージ
  region: string | null;           // リージョン
  bucket_name: string | null;      // バケット名
}
```

**レスポンス例（成功時）**:
```json
{
  "connected": true,
  "message": "Successfully connected to bucket 'my-bucket' in region 'ap-northeast-1'",
  "region": "ap-northeast-1",
  "bucket_name": "my-bucket"
}
```

**レスポンス例（失敗時）**:
```json
{
  "connected": false,
  "message": "Failed to connect to S3: NoSuchBucket. Please check your credentials and bucket name.",
  "region": "us-west-2",
  "bucket_name": "non-existent-bucket"
}
```

**エラー**:
- `NotConfigured`: 認証情報が設定されていない

**実装場所**: `src-tauri/src/commands.rs:275`

---

## ファイル操作

### list_files

S3バケット内のファイル一覧を取得します。

**シグネチャ**:
```rust
#[tauri::command]
pub async fn list_files(
    request: ListFilesRequest
) -> CommandResult<ListFilesResponse>
```

**パラメータ**:

| 名前 | 型 | 説明 | 必須 |
|------|-----|------|------|
| `prefix` | `string` | ファイルをフィルタするプレフィックス | ✅ |

**リクエスト例**:
```typescript
const request: ListFilesRequest = {
  prefix: 'documents/'  // 'documents/' で始まるファイルのみ取得
};

const response = await invoke<ListFilesResponse>('list_files', { request });
```

**レスポンス**:
```typescript
interface ListFilesResponse {
  files: FileInfo[];      // ファイル情報の配列
  total_count: number;    // ファイル総数
}

interface FileInfo {
  path: string;              // ファイルパス
  size: number;              // ファイルサイズ（バイト）
  last_modified: string;     // 最終更新日時（RFC3339形式）
  etag: string | null;       // ETag（S3のバージョン識別子）
}
```

**レスポンス例**:
```json
{
  "files": [
    {
      "path": "documents/report.pdf",
      "size": 1048576,
      "last_modified": "2024-01-15T10:30:00Z",
      "etag": "d41d8cd98f00b204e9800998ecf8427e"
    },
    {
      "path": "documents/notes.txt",
      "size": 2048,
      "last_modified": "2024-01-14T15:20:00Z",
      "etag": "098f6bcd4621d373cade4e832627b4f6"
    }
  ],
  "total_count": 2
}
```

**エラー**:
- `NotConfigured`: 認証情報が設定されていない
- `Storage`: S3接続エラー

**実装場所**: `src-tauri/src/commands.rs:602`

---

### sync_files

ローカルとリモート間でファイルを同期します。

**シグネチャ**:
```rust
#[tauri::command]
pub async fn sync_files(
    request: SyncFilesRequest,
    db_pool: tauri::State<'_, crate::db::DbPool>
) -> CommandResult<SyncFilesResponse>
```

**パラメータ**:

| 名前 | 型 | 説明 | 必須 |
|------|-----|------|------|
| `local_path` | `string` | ローカルディレクトリのパス | ✅ |
| `remote_prefix` | `string` | S3のプレフィックス | ✅ |

**リクエスト例**:
```typescript
const request: SyncFilesRequest = {
  local_path: '/Users/username/Documents',
  remote_prefix: 'documents/'
};

const response = await invoke<SyncFilesResponse>('sync_files', { request });
```

**レスポンス**:
```typescript
interface SyncFilesResponse {
  success: boolean;              // 同期成功: true、失敗: false
  files_uploaded: number;        // アップロードされたファイル数
  files_downloaded: number;      // ダウンロードされたファイル数
  conflicts_resolved: number;    // 解決された競合数
  files_deleted: number;         // 削除されたファイル数
  message: string;               // 結果メッセージ
}
```

**レスポンス例（成功時）**:
```json
{
  "success": true,
  "files_uploaded": 3,
  "files_downloaded": 2,
  "conflicts_resolved": 1,
  "files_deleted": 0,
  "message": "Sync completed successfully. Uploaded: 3, Downloaded: 2, Conflicts: 1, Deleted: 0"
}
```

**レスポンス例（エラーあり）**:
```json
{
  "success": false,
  "files_uploaded": 2,
  "files_downloaded": 1,
  "conflicts_resolved": 0,
  "files_deleted": 0,
  "message": "Sync completed with 1 errors. Uploaded: 2, Downloaded: 1, Conflicts: 0, Deleted: 0. Errors: file1.txt: Access Denied"
}
```

**処理内容**:
1. ローカルファイル一覧を取得
2. S3ファイル一覧を取得
3. 同期履歴から削除を検出
4. 各ファイルの同期状態を判定:
   - `NeedsUpload`: ローカル → S3へアップロード
   - `NeedsDownload`: S3 → ローカルへダウンロード
   - `Conflict`: 両方保存方式で解決
   - `PendingLocalDeletion`: S3から削除
   - `PendingRemoteDeletion`: ローカルから削除
5. 同期履歴をSQLiteに保存

**エラー**:
- `InvalidInput`: パスが空
- `NotConfigured`: 認証情報が設定されていない
- `Storage`: S3エラー
- `Database`: SQLiteエラー

**実装場所**: `src-tauri/src/commands.rs:950`

---

### get_sync_status

ローカルとリモートのファイルを比較し、同期状態を返します（実際の同期は行いません）。

**シグネチャ**:
```rust
#[tauri::command]
pub async fn get_sync_status(
    local_path: String,
    remote_prefix: String,
    db_pool: tauri::State<'_, crate::db::DbPool>
) -> CommandResult<SyncStatusResponse>
```

**パラメータ**:

| 名前 | 型 | 説明 | 必須 |
|------|-----|------|------|
| `local_path` | `string` | ローカルディレクトリのパス | ✅ |
| `remote_prefix` | `string` | S3のプレフィックス | ✅ |

**呼び出し例**:
```typescript
const response = await invoke<SyncStatusResponse>('get_sync_status', {
  local_path: '/Users/username/Documents',
  remote_prefix: 'documents/'
});
```

**レスポンス**:
```typescript
interface SyncStatusResponse {
  comparisons: ComparisonResultDto[];    // 各ファイルの比較結果
  in_sync_count: number;                 // 同期済みファイル数
  needs_upload_count: number;            // アップロード必要数
  needs_download_count: number;          // ダウンロード必要数
  conflict_count: number;                // 競合数
  pending_local_deletion_count: number;  // ローカル削除待ち数
  pending_remote_deletion_count: number; // リモート削除待ち数
}

interface ComparisonResultDto {
  path: string;                    // ファイルパス
  state: SyncState;                // 同期状態
  local_size: number | null;       // ローカルファイルサイズ
  remote_size: number | null;      // リモートファイルサイズ
  local_modified: string | null;   // ローカル最終更新日時
  remote_modified: string | null;  // リモート最終更新日時
  local_etag: string | null;       // ローカルETag（MD5ハッシュ）
  remote_etag: string | null;      // リモートETag
}

type SyncState =
  | 'InSync'                   // 同期済み
  | 'NeedsUpload'              // アップロード必要
  | 'NeedsDownload'            // ダウンロード必要
  | 'Conflict'                 // 競合あり
  | 'PendingLocalDeletion'     // ローカル削除待ち
  | 'PendingRemoteDeletion';   // リモート削除待ち
```

**レスポンス例**:
```json
{
  "comparisons": [
    {
      "path": "report.pdf",
      "state": "InSync",
      "local_size": 1048576,
      "remote_size": 1048576,
      "local_modified": "2024-01-15T10:30:00Z",
      "remote_modified": "2024-01-15T10:30:00Z",
      "local_etag": "d41d8cd98f00b204e9800998ecf8427e",
      "remote_etag": "d41d8cd98f00b204e9800998ecf8427e"
    },
    {
      "path": "notes.txt",
      "state": "NeedsUpload",
      "local_size": 2048,
      "remote_size": null,
      "local_modified": "2024-01-16T08:00:00Z",
      "remote_modified": null,
      "local_etag": "098f6bcd4621d373cade4e832627b4f6",
      "remote_etag": null
    },
    {
      "path": "old-file.txt",
      "state": "PendingRemoteDeletion",
      "local_size": null,
      "remote_size": 512,
      "local_modified": null,
      "remote_modified": "2024-01-10T12:00:00Z",
      "local_etag": null,
      "remote_etag": "5d41402abc4b2a76b9719d911017c592"
    }
  ],
  "in_sync_count": 10,
  "needs_upload_count": 3,
  "needs_download_count": 2,
  "conflict_count": 1,
  "pending_local_deletion_count": 0,
  "pending_remote_deletion_count": 1
}
```

**エラー**:
- `InvalidInput`: パスが空
- `NotConfigured`: 認証情報が設定されていない
- `Storage`: S3エラー

**実装場所**: `src-tauri/src/commands.rs:789`

---

## 設定管理

### save_sync_settings

同期設定をSQLiteデータベースに保存します。

**シグネチャ**:
```rust
#[tauri::command]
pub async fn save_sync_settings(
    request: SaveSettingsRequest,
    db_pool: tauri::State<'_, crate::db::DbPool>
) -> CommandResult<SettingsResponse>
```

**パラメータ**:

| 名前 | 型 | 説明 | 必須 |
|------|-----|------|------|
| `local_path` | `string` | ローカルディレクトリのパス | ✅ |
| `remote_prefix` | `string` | S3のプレフィックス | ✅ |
| `auto_sync_on_startup` | `boolean` | アプリ起動時に自動同期 | ✅ |
| `auto_sync_on_shutdown` | `boolean` | アプリ終了時に自動同期 | ✅ |

**リクエスト例**:
```typescript
const request: SaveSettingsRequest = {
  local_path: '/Users/username/Documents',
  remote_prefix: 'documents/',
  auto_sync_on_startup: true,
  auto_sync_on_shutdown: true
};

const response = await invoke<SettingsResponse>('save_sync_settings', { request });
```

**レスポンス**:
```typescript
interface SettingsResponse {
  success: boolean;                  // 成功: true、失敗: false
  message: string;                   // 結果メッセージ
  settings: SyncSettings | null;     // 保存された設定
}

interface SyncSettings {
  local_path: string;
  remote_prefix: string;
  auto_sync_on_startup: boolean;
  auto_sync_on_shutdown: boolean;
  created_at: string;                // 作成日時（RFC3339形式）
  updated_at: string;                // 更新日時（RFC3339形式）
}
```

**レスポンス例**:
```json
{
  "success": true,
  "message": "Settings saved successfully",
  "settings": {
    "local_path": "/Users/username/Documents",
    "remote_prefix": "documents/",
    "auto_sync_on_startup": true,
    "auto_sync_on_shutdown": true,
    "created_at": "2024-01-15T10:00:00Z",
    "updated_at": "2024-01-16T08:30:00Z"
  }
}
```

**エラー**:
- `InvalidInput`: ローカルパスが空
- `Database`: SQLiteエラー

**実装場所**: `src-tauri/src/commands.rs:1970`

---

### get_sync_settings

保存されている同期設定を取得します。

**シグネチャ**:
```rust
#[tauri::command]
pub async fn get_sync_settings(
    db_pool: tauri::State<'_, crate::db::DbPool>
) -> CommandResult<SettingsResponse>
```

**パラメータ**: なし

**呼び出し例**:
```typescript
const response = await invoke<SettingsResponse>('get_sync_settings');
```

**レスポンス**: `save_sync_settings` と同じ

**レスポンス例（設定がある場合）**:
```json
{
  "success": true,
  "message": "Settings loaded successfully",
  "settings": {
    "local_path": "/Users/username/Documents",
    "remote_prefix": "documents/",
    "auto_sync_on_startup": true,
    "auto_sync_on_shutdown": true,
    "created_at": "2024-01-15T10:00:00Z",
    "updated_at": "2024-01-16T08:30:00Z"
  }
}
```

**レスポンス例（設定がない場合）**:
```json
{
  "success": false,
  "message": "No settings found",
  "settings": null
}
```

**エラー**:
- `Database`: SQLiteエラー

**実装場所**: `src-tauri/src/commands.rs:2012`

---

## エラーハンドリング

### CommandError型

すべてのTauri Commandsは `CommandError` を返す可能性があります:

```rust
pub enum CommandError {
    Storage(StorageError),          // S3エラー
    Credential(CredentialError),    // 認証情報エラー
    Sync(SyncError),                // 同期エラー
    Database(DbError),              // データベースエラー
    NotConfigured(String),          // 未設定エラー
    InvalidInput(String),           // 入力エラー
    OperationFailed(String),        // 操作失敗
}
```

### フロントエンドでのエラーハンドリング

```typescript
try {
  const response = await invoke<SyncFilesResponse>('sync_files', { request });
  console.log('Success:', response);
} catch (error) {
  // エラーメッセージは文字列として返される
  if (typeof error === 'string') {
    if (error.includes('Not configured')) {
      // 認証情報未設定
      alert('Please set AWS credentials first');
    } else if (error.includes('Access Denied')) {
      // 権限エラー
      alert('Access denied. Check your IAM permissions');
    } else if (error.includes('Network')) {
      // ネットワークエラー
      alert('Network error. Please check your connection');
    } else {
      // その他のエラー
      alert(`Error: ${error}`);
    }
  }
}
```

---

## 型定義

### TypeScript型定義

完全な型定義は `src/types/index.ts` に含まれています。以下は主要な型のサマリーです:

```typescript
// 認証情報
export interface SetCredentialsRequest {
  access_key_id: string;
  secret_access_key: string;
  region: string;
  bucket_name: string;
}

export interface CredentialsResponse {
  success: boolean;
  message: string;
}

export interface GetCredentialsResponse {
  has_credentials: boolean;
  access_key_id: string | null;
  region: string | null;
  bucket_name: string | null;
}

export interface ConnectionTestResponse {
  connected: boolean;
  message: string;
  region: string | null;
  bucket_name: string | null;
}

// ファイル操作
export interface ListFilesRequest {
  prefix: string;
}

export interface FileInfo {
  path: string;
  size: number;
  last_modified: string;
  etag: string | null;
}

export interface ListFilesResponse {
  files: FileInfo[];
  total_count: number;
}

export interface SyncFilesRequest {
  local_path: string;
  remote_prefix: string;
}

export interface SyncFilesResponse {
  success: boolean;
  files_uploaded: number;
  files_downloaded: number;
  conflicts_resolved: number;
  files_deleted: number;
  message: string;
}

export type SyncState =
  | 'InSync'
  | 'NeedsUpload'
  | 'NeedsDownload'
  | 'Conflict'
  | 'PendingLocalDeletion'
  | 'PendingRemoteDeletion';

export interface ComparisonResultDto {
  path: string;
  state: SyncState;
  local_size: number | null;
  remote_size: number | null;
  local_modified: string | null;
  remote_modified: string | null;
  local_etag: string | null;
  remote_etag: string | null;
}

export interface SyncStatusResponse {
  comparisons: ComparisonResultDto[];
  in_sync_count: number;
  needs_upload_count: number;
  needs_download_count: number;
  conflict_count: number;
  pending_local_deletion_count: number;
  pending_remote_deletion_count: number;
}

// 設定管理
export interface SaveSettingsRequest {
  local_path: string;
  remote_prefix: string;
  auto_sync_on_startup: boolean;
  auto_sync_on_shutdown: boolean;
}

export interface SyncSettings {
  local_path: string;
  remote_prefix: string;
  auto_sync_on_startup: boolean;
  auto_sync_on_shutdown: boolean;
  created_at: string;
  updated_at: string;
}

export interface SettingsResponse {
  success: boolean;
  message: string;
  settings: SyncSettings | null;
}
```

---

## リファレンス

- **実装**: `src-tauri/src/commands.rs`
- **テスト**: `src-tauri/src/commands.rs` (テストセクション)
- **アーキテクチャ**: [ARCHITECTURE.md](./ARCHITECTURE.md)

---

質問や問題は、[GitHub Issues](https://github.com/ohr486/file-funeral/issues)で受け付けています。
