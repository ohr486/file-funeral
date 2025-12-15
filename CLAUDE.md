# CLAUDE.md

このファイルは、Claude Code (claude.ai/code) がこのリポジトリで作業する際のガイドを提供します。

## プロジェクト概要

file-funeralは、ローカルストレージとクラウドプロバイダー間でファイルをバックアップ・同期するクラウドネイティブなデスクトップアプリケーションです。主な用途は、複数PC間での双方向ファイル同期によるファイル共有です。

**主要機能:**
- ローカルファイルをクラウドストレージ（AWS S3、GCS、S3互換）にアップロード/バックアップ
- クラウドからローカルへのファイルダウンロード
- ローカルとクラウド間の双方向同期
- 同期状態インジケーター付きファイルリストの視覚的表示
- 「両方保存」方式による競合解決（Dropboxスタイル）

**詳細な要件**: `REQUIREMENTS.md` を参照してください。

---

## 技術スタック

### バックエンド
- **フレームワーク**: Tauri v2
- **言語**: Rust
- **非同期ランタイム**: Tokio
- **クラウドSDK**:
  - v1.0: `aws-sdk-s3`
  - v2.0以降: S3互換、GCS SDK
- **プラグイン**: `tauri-plugin-fs`（ファイルシステムアクセス）
- **認証情報ストレージ**: `keyring` クレート（OSキーチェーン）

### フロントエンド
- **ビルドツール**: Vite
- **フレームワーク**: React / Vue / Svelte（未決定）
- **言語**: TypeScript

### 主要な依存関係
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

## プロジェクト構造

想定されるディレクトリ構成:
```
file-funeral/
├── src-tauri/           # Rustバックエンド
│   ├── src/
│   │   ├── main.rs
│   │   ├── storage/     # クラウドストレージ抽象化層
│   │   │   ├── mod.rs   # Storage trait定義
│   │   │   ├── s3.rs    # AWS S3実装
│   │   │   └── ...      # 将来: GCS、S3互換
│   │   ├── sync/        # 同期エンジン
│   │   └── ...
│   └── Cargo.toml
├── src/                 # フロントエンド（React/Vue/Svelte）
├── REQUIREMENTS.md      # 詳細要件定義書
└── CLAUDE.md           # このファイル
```

---

## アーキテクチャ

### ストレージ抽象化レイヤー

アプリはクラウドストレージプロバイダーのためのトレイトベースの抽象化を使用します:

```rust
trait CloudStorageProvider {
    async fn upload(&self, path: &str, data: &[u8], metadata: FileMetadata) -> Result<()>;
    async fn download(&self, path: &str) -> Result<Vec<u8>>;
    async fn list(&self, prefix: &str) -> Result<Vec<FileInfo>>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn get_metadata(&self, path: &str) -> Result<FileMetadata>;
}
```

**実装:**
- v1.0: `S3Provider` (AWS S3)
- v2.0: `S3CompatibleProvider` (MinIO、Backblaze B2等)
- v2.5: `GCSProvider` (Google Cloud Storage)

### 同期エンジン

**同期タイミング:**
- アプリ起動時（自動）
- ユーザーが「同期」ボタンをクリックしたとき（手動）
- （オプション）アプリ終了時

**競合解決:**
- 競合検知時、両方のファイルを保存
- クラウド版: `filename.txt`
- ローカル版: `filename (PC名's conflicted copy YYYY-MM-DD).txt`

**削除の扱い:**
- v1.0: 削除を同期（完全同期）
- v2.0以降: ソフト削除（ゴミ箱/復元機能付き）

---

## 開発コマンド

### セットアップ
```bash
# Tauri CLIのインストール
cargo install tauri-cli

# フロントエンド依存関係のインストール
cd file-funeral
npm install  # または pnpm/yarn
```

### 開発
```bash
# 開発モードで実行
npm run tauri dev

# 本番ビルド
npm run tauri build
```

### テスト
```bash
# Rustテストの実行
cd src-tauri
cargo test

# フロントエンドテストの実行
npm test
```

---

## 重要な実装ガイドライン

### 1. ファイル同期
- 常にTokioで非同期操作を使用する
- ネットワーク障害に対する適切なエラーハンドリングを実装
- ファイル整合性チェックにMD5/ETagを使用
- フルパスを持つS3キーを使用してフォルダ構造を保持

### 2. セキュリティ
- 認証情報を**絶対に**平文で保存しない
- `keyring`クレートを使用してOSキーチェーンを利用:
  - macOS: Keychain
  - Windows: Credential Manager
  - Linux: Secret Service API
- 開発用に環境変数（`AWS_ACCESS_KEY_ID`等）をサポート
- すべてのアップロードでS3サーバーサイド暗号化（SSE-S3）を有効化

### 3. ファイル管理
- **保持するメタデータ**: ファイル名、サイズ、更新日時、ハッシュ値
- **無視するメタデータ**: パーミッション、作成日時、所有者
- **デフォルト除外**: `.DS_Store`、`Thumbs.db`、`*.tmp`、`*.swp`
- **特殊ファイル**: シンボリックリンク、ハードリンクはスキップ（警告表示）
- **サイズ制限**: 最大5GB（v1.0）、100MB以上のファイルはプログレスバー表示

### 4. エラーハンドリング
- ユーザーフレンドリーなエラーメッセージを表示
- トラブルシューティング用の詳細なエラーログを記録
- ネットワークタイムアウトとリトライを適切に処理
- 同期操作を黙って失敗させない

### 5. クロスプラットフォーム対応
- クロスプラットフォームパス処理にTauriのpath APIを使用
- Windows、macOS、Linuxでテスト
- プラットフォーム固有の隠しファイルを適切に処理

---

## 開発ロードマップ

### v1.0 (MVP) - AWS S3のみ
- [x] 要件定義完了
- [ ] プロジェクトセットアップ（Tauri + フロントエンド）
- [ ] Storage trait + S3実装
- [ ] 基本的な同期エンジン
- [ ] 同期状態付きファイルリストUI
- [ ] 認証情報管理（キーチェーン）
- [ ] 初回セットアップウィザード

### v1.5 - 除外パターン機能
- [ ] .gitignore方式の除外ルール
- [ ] 除外設定UI

### v2.0 - S3互換対応 + ソフト削除
- [ ] S3互換プロバイダーサポート
- [ ] マルチプロバイダー選択UI
- [ ] 複数フォルダ選択機能
- [ ] ソフト削除（ゴミ箱機能）

### v2.5 - Google Cloud Storage対応
- [ ] GCSプロバイダー実装
- [ ] Google認証フロー

### v3.0 - 高度な機能
- [ ] Azure Blob Storage対応
- [ ] クライアントサイド暗号化
- [ ] アプリロック機能
- [ ] シンボリックリンク対応

---

## Git ワークフロー

- メインブランチ: `main`
- 開発にはフィーチャーブランチを使用
- Conventional Commitsに従う
- クリーンな履歴を維持

---

## リソース

- **要件定義**: `REQUIREMENTS.md`
- **Tauriドキュメント**: https://v2.tauri.app/
- **AWS SDK for Rust**: https://github.com/awslabs/aws-sdk-rust
- **File Systemプラグイン**: https://v2.tauri.app/plugin/file-system
