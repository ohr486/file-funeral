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

## プロジェクト構造

```
file-funeral/
├── src-tauri/          # Rustバックエンド
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   ├── tests/          # 統合テスト
│   ├── Cargo.toml
│   └── clippy.toml
├── src/                # Reactフロントエンド
│   ├── App.tsx
│   └── main.tsx
├── REQUIREMENTS.md     # 詳細要件定義
├── CLAUDE.md          # 開発ガイド
└── TODO.md            # 実装タスク一覧
```

## ドキュメント

- [REQUIREMENTS.md](./REQUIREMENTS.md) - 詳細な要件定義
- [CLAUDE.md](./CLAUDE.md) - 開発ガイド（Claude Code用）
- [TODO.md](./TODO.md) - 実装タスク一覧

## ライセンス

このプロジェクトは[LICENSE](./LICENSE)の下でライセンスされています。

