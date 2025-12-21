# file-funeral

クラウドファイル同期デスクトップアプリケーション

ローカルファイルをクラウドストレージ（AWS S3、GCS、S3互換）にバックアップ・同期するデスクトップアプリです。複数PC間での双方向ファイル同期を主な用途としています。

## 主要機能

- ✅ ローカルファイルのクラウドへのアップロード/バックアップ
- ✅ クラウドからローカルへのダウンロード
- ✅ ローカルとクラウド間の双方向同期
- ✅ 同期状態の視覚的表示
- ✅ 競合解決（両方保存方式）

## 技術スタック

- **Backend**: Rust + Tauri v2.9.5
- **Frontend**: React 19 + TypeScript 5.9 + Vite 7
- **Cloud**: AWS SDK for Rust (v1.0), 将来的にGCS、S3互換対応
- **開発ツール**: ESLint, Clippy, Cargo Test

## セットアップ

### 必要な環境

- Rust 1.92.0以降
- Node.js 18以降
- npm または pnpm

### インストール

```bash
# リポジトリをクローン
git clone https://github.com/ohr486/file-funeral.git
cd file-funeral

# 依存関係をインストール
npm install

# Tauri CLIをインストール（グローバル）
cargo install tauri-cli
```

### 開発サーバーの起動

```bash
npm run tauri:dev
```

初回起動時は、Rustの依存関係のダウンロード・コンパイルに3-5分かかります。

## 開発コマンド

```bash
# 開発サーバー起動
npm run tauri:dev

# ビルド
npm run tauri:build

# テスト実行
npm test

# Lint実行
npm run lint

# 自動修正
npm run lint:fix

# フォーマット
npm run format

# データベース確認
npm run db:inspect

# SQLite対話シェル
npm run db:shell
```

## AWS認証情報の設定（macOS）

### 1. AWS認証情報の取得

1. [AWS Management Console](https://console.aws.amazon.com/)にサインイン
2. IAM（Identity and Access Management）に移動
3. 新しいユーザーを作成、または既存のユーザーを選択
4. 「AmazonS3FullAccess」ポリシーをアタッチ
5. アクセスキーを生成し、以下の情報をコピー:
   - Access Key ID（例: `AKIAIOSFODNN7EXAMPLE`）
   - Secret Access Key（例: `wJalrXUtnFEMI/K7MDENG/...`）

### 2. アプリから認証情報を設定

1. アプリを起動: `npm run tauri:dev`
2. メイン画面右上の「Settings」ボタンをクリック
3. AWS認証情報フォームに入力:
   - **Access Key ID**: 取得したアクセスキーID
   - **Secret Access Key**: 取得したシークレットアクセスキー
   - **Region**: 使用するAWSリージョン（例: `us-east-1`, `ap-northeast-1`）
   - **S3 Bucket Name**: 同期先のS3バケット名
4. 「Save Credentials」ボタンをクリック
5. 自動的に接続テストが実行され、成功すると「Connected successfully」と表示されます

### 3. macOSキーチェーンでの確認（オプション）

認証情報はmacOSのキーチェーンに安全に保存されます。確認するには:

1. Spotlight検索で「キーチェーンアクセス」を開く
2. 左側で「ログイン」キーチェーンを選択
3. 検索バーで「file-funeral」を検索
4. AWS認証情報が暗号化されて保存されていることを確認できます

> **セキュリティ**: 認証情報は平文では保存されず、macOSのキーチェーンAPIを使用して暗号化されます。

## 同期履歴データベース（SQLite）

file-funeralは同期履歴をSQLiteデータベースに保存します。これにより削除検出やファイル変更の追跡が可能になります。

### データベースの場所

| 環境 | パス |
|------|------|
| 本番環境 | `~/.file-funeral/sync_history.db` |
| テスト環境 | `/tmp/.file-funeral-test-{PID}/sync_history.db` |

### データベースの確認方法

#### 1. 簡易確認（推奨）

```bash
# データベースの概要を表示
npm run db:inspect
```

以下の情報が表示されます：
- スキーマバージョン
- 同期操作の総数
- 成功/失敗の統計
- 最新10件の同期履歴
- 追跡されているファイルの総数

#### 2. 対話シェル

```bash
# SQLite対話シェルを開く
npm run db:shell
```

シェル内で使えるコマンド：

```sql
-- テーブル一覧を表示
.tables

-- スキーマを表示
.schema sync_history

-- カラム名を表示する設定
.headers on
.mode column

-- 最新の同期履歴を表示
SELECT id, datetime(sync_completed_at) as time,
       files_uploaded, files_downloaded, files_deleted, success
FROM sync_history
ORDER BY sync_completed_at DESC
LIMIT 10;

-- 特定の同期のファイル一覧
SELECT file_path, file_size, was_local, was_remote
FROM synced_files
WHERE sync_history_id = 1;

-- 終了
.quit
```

#### 3. ワンライナー確認

```bash
# テーブル一覧
sqlite3 ~/.file-funeral/sync_history.db ".tables"

# 最新5件の同期履歴
sqlite3 -header -column ~/.file-funeral/sync_history.db \
  "SELECT id, datetime(sync_completed_at) as time,
   files_uploaded as up, files_downloaded as down, files_deleted as del
   FROM sync_history ORDER BY sync_completed_at DESC LIMIT 5;"

# 同期履歴の総数
sqlite3 ~/.file-funeral/sync_history.db \
  "SELECT COUNT(*) FROM sync_history;"
```

### データベーススキーマ

#### sync_history テーブル
同期操作の履歴を記録します。

| カラム | 型 | 説明 |
|--------|-----|------|
| id | INTEGER | 主キー |
| sync_started_at | TEXT | 同期開始時刻 (RFC3339) |
| sync_completed_at | TEXT | 同期完了時刻 (RFC3339) |
| local_path | TEXT | ローカルパス |
| remote_prefix | TEXT | リモートプレフィックス |
| files_uploaded | INTEGER | アップロードされたファイル数 |
| files_downloaded | INTEGER | ダウンロードされたファイル数 |
| files_deleted | INTEGER | 削除されたファイル数 |
| conflicts_resolved | INTEGER | 解決された競合数 |
| success | INTEGER | 成功フラグ (0/1) |
| error_message | TEXT | エラーメッセージ（失敗時） |

#### synced_files テーブル
最終同期時のファイル一覧を記録します（削除検出に使用）。

| カラム | 型 | 説明 |
|--------|-----|------|
| id | INTEGER | 主キー |
| sync_history_id | INTEGER | sync_historyテーブルへの外部キー |
| file_path | TEXT | ファイルパス |
| file_size | INTEGER | ファイルサイズ（バイト） |
| last_modified | TEXT | 最終更新日時 (RFC3339) |
| etag | TEXT | ETag（S3の場合） |
| was_local | INTEGER | ローカルに存在したか (0/1) |
| was_remote | INTEGER | リモートに存在したか (0/1) |

### データのエクスポート

```bash
# CSV形式でエクスポート
sqlite3 -header -csv ~/.file-funeral/sync_history.db \
  "SELECT * FROM sync_history;" > sync_history.csv

# JSON形式でエクスポート（jqが必要）
sqlite3 ~/.file-funeral/sync_history.db \
  "SELECT json_object(
    'id', id,
    'completed', sync_completed_at,
    'uploaded', files_uploaded,
    'downloaded', files_downloaded
   ) FROM sync_history;" | jq -s '.'
```

## プロジェクト構造

```
file-funeral/
├── src-tauri/              # Rustバックエンド
│   ├── src/
│   │   ├── main.rs         # エントリポイント
│   │   ├── lib.rs          # メインロジック
│   │   ├── commands.rs     # Tauriコマンド
│   │   ├── auth/           # 認証情報管理
│   │   ├── storage/        # ストレージ抽象化層
│   │   ├── sync/           # 同期エンジン
│   │   └── db/             # データベース（SQLite）
│   ├── tests/              # 統合テスト
│   ├── Cargo.toml
│   └── clippy.toml
├── src/                    # Reactフロントエンド
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/         # Reactコンポーネント
│   ├── pages/              # ページコンポーネント
│   └── types/              # TypeScript型定義
├── scripts/                # ユーティリティスクリプト
│   └── inspect_db.sh       # SQLiteデータベース確認スクリプト
├── ~/.file-funeral/        # アプリケーションデータ（ホームディレクトリ）
│   └── sync_history.db     # 同期履歴データベース
├── REQUIREMENTS.md         # 詳細要件定義
├── CLAUDE.md              # 開発ガイド
└── TODO.md                # 実装タスク一覧
```

## ドキュメント

- [REQUIREMENTS.md](./REQUIREMENTS.md) - 詳細な要件定義
- [CLAUDE.md](./CLAUDE.md) - 開発ガイド（Claude Code用）
- [TODO.md](./TODO.md) - 実装タスク一覧

## ライセンス

このプロジェクトは[LICENSE](./LICENSE)の下でライセンスされています。

