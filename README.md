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

