# file-funeral コントリビューションガイド

file-funeralプロジェクトへのコントリビューションを歓迎します！このガイドでは、プロジェクトに貢献するための手順とガイドラインを説明します。

## 目次

- [行動規範](#行動規範)
- [はじめに](#はじめに)
- [開発環境のセットアップ](#開発環境のセットアップ)
- [開発ワークフロー](#開発ワークフロー)
- [コーディング規約](#コーディング規約)
- [テストの書き方](#テストの書き方)
- [プルリクエストの作成](#プルリクエストの作成)
- [Issue報告](#issue報告)
- [ドキュメントの改善](#ドキュメントの改善)
- [リリースプロセス](#リリースプロセス)

---

## 行動規範

### 私たちの約束

私たちは、オープンで歓迎的なコミュニティを作ることを約束します。すべての参加者に対して、性別、性的指向、障害、外見、体型、人種、年齢、宗教、技術的な選択に関係なく、ハラスメントのない環境を提供します。

### 期待される行動

- 他者への思いやりと尊重
- 建設的なフィードバック
- コミュニティの利益を最優先
- 初心者への寛容さと助け合い

### 禁止される行動

- ハラスメント、侮辱的なコメント
- トローリング、個人攻撃
- 公的または私的なハラスメント
- 他者のプライベート情報の無断公開

---

## はじめに

### コントリビューションの種類

以下のような貢献を歓迎します:

- **バグ修正**: バグレポート、修正のプルリクエスト
- **新機能**: 新しい機能の提案、実装
- **ドキュメント**: ドキュメントの改善、翻訳
- **テスト**: テストカバレッジの向上
- **コードレビュー**: 他の人のプルリクエストのレビュー
- **デザイン**: UIの改善提案
- **パフォーマンス**: パフォーマンス改善

### 初めての貢献

初めての貢献を探している場合は、以下のラベルが付いたIssueを探してください:

- `good first issue`: 初心者向けの簡単な問題
- `help wanted`: コミュニティからの助けを求めている問題
- `documentation`: ドキュメントの改善

---

## 開発環境のセットアップ

### 必要な環境

- **Rust**: 1.92.0以降
- **Node.js**: 18以降
- **Git**: 2.x以降
- **macOS**: 11.0以降（開発環境として推奨）

### セットアップ手順

1. **リポジトリをフォーク**:
   - GitHubで `ohr486/file-funeral` をフォーク

2. **クローン**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/file-funeral.git
   cd file-funeral
   ```

3. **upstream リモートを追加**:
   ```bash
   git remote add upstream https://github.com/ohr486/file-funeral.git
   ```

4. **依存関係をインストール**:
   ```bash
   npm install
   ```

5. **開発サーバーを起動**:
   ```bash
   npm run tauri:dev
   ```

6. **テストを実行**:
   ```bash
   npm test
   ```

7. **Lintを実行**:
   ```bash
   npm run lint
   ```

---

## 開発ワークフロー

### Git ブランチ戦略

- **`main`**: 本番リリース用（安定版）
- **`develop`**: 開発中の機能（デフォルトブランチ）
- **`feature/*`**: 新機能の開発
- **`fix/*`**: バグ修正
- **`docs/*`**: ドキュメント改善

### 作業フロー

1. **最新のdevelopブランチを取得**:
   ```bash
   git checkout develop
   git pull upstream develop
   ```

2. **機能ブランチを作成**:
   ```bash
   # 新機能の場合
   git checkout -b feature/add-google-cloud-storage

   # バグ修正の場合
   git checkout -b fix/s3-connection-timeout

   # ドキュメントの場合
   git checkout -b docs/update-setup-guide
   ```

3. **変更を加える**:
   - コードを書く
   - テストを追加
   - ドキュメントを更新

4. **テストとLintを実行**:
   ```bash
   npm test
   npm run lint
   ```

5. **コミット**:
   ```bash
   git add .
   git commit -m "feat: add Google Cloud Storage support"
   ```

6. **プッシュ**:
   ```bash
   git push origin feature/add-google-cloud-storage
   ```

7. **プルリクエストを作成**:
   - GitHubでプルリクエストを作成
   - ベースブランチ: `develop`
   - 詳細な説明を記述

---

## コーディング規約

### Rust (Backend)

#### コードスタイル

- `cargo fmt` を使用してフォーマット
- `cargo clippy` の警告をすべて解決
- **禁止**: `unwrap()` の使用（`?` または `expect()` を使用）

```rust
// ❌ 悪い例
let value = some_option.unwrap();

// ✅ 良い例
let value = some_option?;
// または
let value = some_option.expect("Expected value but got None");
```

#### 命名規則

- **関数**: `snake_case`
- **構造体**: `PascalCase`
- **定数**: `UPPER_SNAKE_CASE`
- **モジュール**: `snake_case`

```rust
// ✅ 良い例
const MAX_FILE_SIZE: u64 = 5_000_000_000;

struct FileInfo {
    pub path: String,
}

fn calculate_md5_hash(path: &Path) -> Result<String> {
    // ...
}
```

#### ドキュメントコメント

パブリックAPIには必ずドキュメントコメントを付ける:

```rust
/// ファイルをS3にアップロードする
///
/// # Arguments
/// * `path` - S3のキー（パス）
/// * `data` - アップロードするデータ
/// * `metadata` - ファイルメタデータ
///
/// # Returns
/// 成功時は `Ok(())`、失敗時は `StorageError`
///
/// # Examples
/// ```
/// let provider = S3Provider::new("my-bucket".to_string());
/// provider.upload("file.txt", b"content", metadata).await?;
/// ```
pub async fn upload(&self, path: &str, data: &[u8], metadata: FileMetadata)
    -> Result<(), StorageError> {
    // ...
}
```

#### エラーハンドリング

- `anyhow::Result` を内部実装に使用
- `thiserror` を使用してカスタムエラー型を定義
- エラーメッセージは具体的かつ actionable に

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("File too large: {0} bytes (max {1} bytes)")]
    FileTooLarge(u64, u64),

    #[error("Network error: {0}")]
    NetworkError(String),
}
```

### TypeScript/React (Frontend)

#### コードスタイル

- ESLintの設定に従う
- Prettier でフォーマット（自動）

#### 命名規則

- **コンポーネント**: `PascalCase`
- **関数**: `camelCase`
- **定数**: `UPPER_SNAKE_CASE`
- **型/インターフェース**: `PascalCase`

```typescript
// ✅ 良い例
const MAX_RETRY_COUNT = 3;

interface FileInfo {
  path: string;
  size: number;
}

function calculateProgress(current: number, total: number): number {
  return (current / total) * 100;
}

export function FileList({ files }: { files: FileInfo[] }) {
  // ...
}
```

#### Hooks の使用

- 機能ごとにカスタムフックを作成
- `use` プレフィックスを付ける

```typescript
function useSyncStatus(localPath: string, remotePrefix: string) {
  const [status, setStatus] = useState<SyncStatusResponse | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // ...
  }, [localPath, remotePrefix]);

  return { status, loading };
}
```

### コミットメッセージ

Conventional Commits 形式を使用:

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Type

- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメントのみの変更
- `style`: コードの意味に影響しない変更（空白、フォーマットなど）
- `refactor`: バグ修正も機能追加もしないコード変更
- `test`: テストの追加・修正
- `chore`: ビルドプロセスやツールの変更

#### 例

```
feat(storage): add Google Cloud Storage support

- Implement GCSProvider trait
- Add authentication flow for GCS
- Update documentation

Closes #42
```

```
fix(sync): handle network timeout correctly

Previously, network timeouts were not retried. Now we use
exponential backoff to retry failed requests.

Fixes #123
```

---

## テストの書き方

### Rust テスト

#### ユニットテスト

各モジュールに `#[cfg(test)] mod tests` を追加:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation() {
        let now = Utc::now();
        let file = FileInfo::new("test.txt".to_string(), 100, now, None);

        assert_eq!(file.path, "test.txt");
        assert_eq!(file.size, 100);
    }

    #[tokio::test]
    async fn test_s3_upload() {
        // テストの実装
    }
}
```

#### 統合テスト

`tests/` ディレクトリに配置:

```rust
// tests/sync_integration_test.rs
use file_funeral::sync;

#[tokio::test]
async fn test_full_sync_workflow() {
    // テストの実装
}
```

### TypeScript/React テスト

Vitest を使用:

```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { FileList } from './FileList';

describe('FileList', () => {
  it('renders file list correctly', () => {
    const files = [
      { path: 'test.txt', size: 100, last_modified: '2024-01-01', etag: null }
    ];

    render(<FileList files={files} />);
    expect(screen.getByText('test.txt')).toBeInTheDocument();
  });
});
```

### テストの実行

```bash
# すべてのテストを実行
npm test

# Rustテストのみ
npm run test:rust

# TypeScriptテストのみ
npm run test:frontend

# 詳細出力
npm run test:rust:verbose
```

---

## プルリクエストの作成

### PR作成前のチェックリスト

- [ ] `npm test` がパスする
- [ ] `npm run lint` がパスする
- [ ] 新機能にはテストを追加
- [ ] ドキュメントを更新（該当する場合）
- [ ] コミットメッセージがConventional Commits形式
- [ ] 変更内容を `CHANGELOG.md` に追加（メンテナーが行う場合もあり）

### PRテンプレート

```markdown
## 概要
この変更の目的を簡潔に説明してください。

## 変更内容
- 追加した機能/修正したバグ
- 影響を受けるコンポーネント

## テスト
- [ ] ユニットテストを追加
- [ ] 統合テストを追加
- [ ] 手動でテスト済み

## スクリーンショット（UI変更の場合）
変更前と変更後のスクリーンショットを添付してください。

## 関連Issue
Closes #123
```

### コードレビュープロセス

1. プルリクエストを作成
2. CI（自動テスト）がパスするのを待つ
3. メンテナーがレビュー
4. フィードバックに対応
5. 承認後、マージ

---

## Issue報告

### バグレポート

以下の情報を含めてください:

```markdown
## バグの説明
何が起こったか、何が期待されるかを説明してください。

## 再現手順
1. '...'に移動
2. '...'をクリック
3. '...'を入力
4. エラーが表示される

## 期待される動作
何が起こるべきかを説明してください。

## 環境
- OS: macOS 14.2
- Rustバージョン: `rustc --version`
- Node.jsバージョン: `node --version`
- アプリバージョン: v1.0.0

## ログ/エラーメッセージ
```
エラーログをここに貼り付け
```

## スクリーンショット
該当する場合は添付してください。
```

### 機能リクエスト

```markdown
## 機能の説明
提案する機能を説明してください。

## 動機
なぜこの機能が必要なのかを説明してください。

## 代替案
検討した代替案があれば説明してください。

## 追加のコンテキスト
その他の情報やスクリーンショットを追加してください。
```

---

## ドキュメントの改善

### ドキュメントの種類

- **README.md**: プロジェクト概要
- **SETUP_GUIDE.md**: セットアップ手順
- **TROUBLESHOOTING.md**: トラブルシューティング
- **ARCHITECTURE.md**: アーキテクチャ説明
- **API_REFERENCE.md**: API ドキュメント
- **CONTRIBUTING.md**: このファイル

### ドキュメント改善のガイドライン

- 明確で簡潔な文章
- コード例を含める
- スクリーンショットを追加（UI関連）
- 最新の状態を保つ

---

## リリースプロセス

### バージョニング

[Semantic Versioning](https://semver.org/) を使用:

- **Major** (x.0.0): 互換性のない変更
- **Minor** (1.x.0): 後方互換性のある新機能
- **Patch** (1.0.x): 後方互換性のあるバグ修正

### リリース手順（メンテナー向け）

1. `develop` ブランチでバージョンを更新:
   ```bash
   # package.json
   "version": "1.1.0"

   # src-tauri/Cargo.toml
   version = "1.1.0"
   ```

2. `CHANGELOG.md` を更新

3. `develop` → `main` にマージ

4. タグを作成:
   ```bash
   git tag -a v1.1.0 -m "Release v1.1.0"
   git push origin v1.1.0
   ```

5. GitHub Releasesでリリースノートを公開

---

## サポート

質問や不明点がある場合は:

- [GitHub Discussions](https://github.com/ohr486/file-funeral/discussions) で質問
- [GitHub Issues](https://github.com/ohr486/file-funeral/issues) でバグ報告
- [README.md](./README.md) でプロジェクト概要を確認

---

## ライセンス

コントリビューションすることで、あなたの貢献がプロジェクトのライセンス（MITライセンス）の下でライセンスされることに同意したことになります。

---

ご協力ありがとうございます！🙏
