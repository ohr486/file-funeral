# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-02

### 🎉 Initial Release - v1.0 MVP

file-funeral の最初の安定版リリースです。AWS S3を使用したローカルファイルのクラウド同期機能を提供します。

### Added

#### Core Functionality
- **双方向ファイル同期**: ローカルとAWS S3間でのファイル同期
- **競合解決**: Dropboxスタイルの両方保存方式による競合解決
- **削除同期**: 同期履歴に基づくファイル削除の検出と同期
- **自動同期**: アプリ起動時・終了時の自動同期機能
- **同期履歴管理**: SQLiteデータベースによる同期操作の履歴管理

#### Security & Authentication
- **macOSキーチェーン統合**: AWS認証情報の安全な暗号化保存
- **環境変数サポート**: 開発環境用の認証情報設定
- **S3サーバーサイド暗号化**: デフォルトでSSE-S3を有効化

#### File Management
- **メタデータ管理**: ファイルサイズ、更新日時、MD5ハッシュ（ETag）の追跡
- **除外ファイル処理**:
  - 隠しファイル（`.`で始まる）の自動スキップ
  - `.DS_Store`, `Thumbs.db`, `*.tmp`, `*.swp`, `*~` の自動除外
  - シンボリックリンクのスキップと警告表示
- **大容量ファイル対応**:
  - 最大5GBまでのファイルをサポート
  - 100MB以上のファイルはプログレス情報をログ出力
  - バッファリングによるメモリ効率的なハッシュ計算

#### Error Handling & Reliability
- **エラーリトライロジック**:
  - 指数バックオフによる自動リトライ（最大3回）
  - ネットワークエラーと認証エラーの自動検出
  - タイムアウト処理
- **詳細なログ出力**:
  - 環境変数（`RUST_LOG`, `DEBUG`）によるログレベル設定
  - タイムスタンプ、モジュールパス、ログレベルを含むフォーマット
  - `env_logger` による柔軟なログ設定

#### User Interface
- **メイン画面**: ファイルリスト表示と同期ボタン
- **設定画面**: AWS認証情報の入力と接続テスト
- **初回セットアップウィザード**: ガイド付きセットアップフロー
- **同期状態の視覚表示**:
  - 🟢 InSync: 同期済み
  - 🔵 NeedsUpload: アップロード必要
  - 🟡 NeedsDownload: ダウンロード必要
  - 🔴 Conflict: 競合あり
  - ⚪ PendingLocalDeletion: ローカル削除待ち
  - ⚪ PendingRemoteDeletion: リモート削除待ち

#### Testing & Quality
- **テストインフラ**:
  - ユニットテスト（`src/lib.rs`）
  - 統合テスト（`tests/integration_test.rs`）
  - テスト実行スクリプト（`npm test`）
  - 191個のテストケース（すべてパス）
- **Lintインフラ**:
  - Rust Clippy設定
  - TypeScript ESLint設定
  - Lintスクリプト（`npm run lint`）

#### Documentation
- **ユーザー向けドキュメント**:
  - `README.md`: プロジェクト概要と基本的な使い方
  - `SETUP_GUIDE.md`: 詳細なセットアップガイド
  - `TROUBLESHOOTING.md`: トラブルシューティングガイド
- **開発者向けドキュメント**:
  - `REQUIREMENTS.md`: 詳細な要件定義
  - `ARCHITECTURE.md`: システムアーキテクチャとデータフロー
  - `API_REFERENCE.md`: Tauri Commands APIリファレンス
  - `CONTRIBUTING.md`: コントリビューションガイド
  - `CLAUDE.md`: 開発ガイド（Claude Code用）
  - `TODO.md`: 実装タスク一覧

#### Database Features
- **SQLiteデータベース**:
  - `sync_history` テーブル: 同期操作の履歴
  - `synced_files` テーブル: 最終同期時のファイル一覧（削除検出用）
  - `sync_settings` テーブル: アプリ設定の永続化
- **データベース管理ツール**:
  - `npm run db:inspect`: データベースの概要表示
  - `npm run db:shell`: SQLite対話シェル

### Technical Details

#### Backend Architecture
- **Framework**: Tauri v2.9.5
- **Language**: Rust (edition 2021, 1.77.2+)
- **Cloud SDK**: AWS SDK for Rust v1.117.0
- **Async Runtime**: Tokio (full features)
- **Database**: rusqlite 0.32 + r2d2 connection pool
- **Security**: keyring 3 (macOS Keychain integration)
- **Retry Logic**: backon 1.2 (exponential backoff)
- **Logging**: env_logger 0.11

#### Frontend Architecture
- **Framework**: React 19.2.3
- **Language**: TypeScript 5.9.3
- **Build Tool**: Vite 7.3.0
- **UI Components**: Radix UI + Tailwind CSS 4.1.18
- **Notifications**: sonner 2.0.7

#### Storage Abstraction
- `CloudStorageProvider` trait: クラウドストレージプロバイダーの抽象化
- `S3Provider`: AWS S3実装（v1.0）
- 将来のプロバイダー対応の基盤を構築

### Known Limitations

- **プラットフォーム**: macOSのみサポート（Windows/Linux対応は将来のバージョン）
- **同期フォルダ**: 1つのフォルダのみ
- **クラウドプロバイダー**: AWS S3のみ
- **最大ファイルサイズ**: 5GB
- **プログレスバー**: 詳細なバイト単位のプログレス表示は未実装（AWS SDK制限）
- **除外パターン**: `.gitignore`方式の除外ルールは未実装（v1.5で対応予定）

### Security Notes

- 認証情報はmacOSキーチェーンに暗号化されて保存
- S3転送はHTTPS（TLS 1.2+）で暗号化
- S3サーバーサイド暗号化（SSE-S3）がデフォルトで有効
- シークレットアクセスキーはログに出力されない

### Migration Notes

初回リリースのため、移行手順はありません。

---

## [Unreleased]

### Planned Features (Future Versions)

#### v1.5 - 除外パターン機能
- `.gitignore`方式の除外ルール実装
- 除外設定UI

#### v2.0 - S3互換対応 + ソフト削除
- S3互換Provider実装（MinIO、Backblaze B2など）
- 複数フォルダ選択機能
- ソフト削除（ゴミ箱機能）
- マルチプロバイダー選択UI

#### v2.5 - Google Cloud Storage対応
- GCSProvider実装
- Google認証フロー

#### v3.0 - 高度な機能
- Azure Blob Storage対応
- クライアントサイド暗号化
- アプリロック機能
- シンボリックリンク対応

---

## Release Notes

### v1.0.0 Highlights

**file-funeral v1.0.0** は、AWS S3を使用した安全で高速なファイル同期ソリューションです。

#### 主な特徴

1. **シンプルな操作性**: Dropbox風のUIで、誰でも簡単に使える
2. **安全性**: macOSキーチェーンによる認証情報の暗号化保存
3. **信頼性**: 自動リトライ、エラーハンドリング、同期履歴管理
4. **透明性**: 詳細なログ出力、同期状態の視覚化

#### Getting Started

1. AWSアカウントとS3バケットを準備
2. file-funeralをインストール
3. AWS認証情報を設定
4. 同期フォルダを選択
5. 「Sync」ボタンをクリック

詳細は [SETUP_GUIDE.md](./SETUP_GUIDE.md) を参照してください。

#### Contributors

- [@ohr486](https://github.com/ohr486) - Initial development

#### Special Thanks

- Tauri チーム: クロスプラットフォームデスクトップアプリフレームワーク
- AWS SDK for Rust チーム: AWS統合
- Rustコミュニティ: 素晴らしいエコシステム

---

[1.0.0]: https://github.com/ohr486/file-funeral/releases/tag/v1.0.0
[Unreleased]: https://github.com/ohr486/file-funeral/compare/v1.0.0...HEAD
