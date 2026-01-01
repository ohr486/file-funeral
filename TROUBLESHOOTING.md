# file-funeral トラブルシューティングガイド

このガイドでは、file-funeralの使用中に発生する可能性がある一般的な問題と解決方法を説明します。

## 目次

- [認証情報の問題](#認証情報の問題)
- [接続エラー](#接続エラー)
- [同期の問題](#同期の問題)
- [パフォーマンスの問題](#パフォーマンスの問題)
- [データベースの問題](#データベースの問題)
- [ビルドとインストールの問題](#ビルドとインストールの問題)
- [ログの確認方法](#ログの確認方法)
- [サポート](#サポート)

---

## 認証情報の問題

### 問題: "AWS credentials not found" エラー

**症状**: 接続テストや同期時に「AWS credentials not found. Please set credentials first.」と表示される

**解決方法**:

1. Settings画面で認証情報が正しく入力されているか確認
2. 「Save Credentials」ボタンをクリックしたか確認
3. macOSキーチェーンアクセスで認証情報を確認:
   ```bash
   # Spotlightで「キーチェーンアクセス」を検索
   # 左側で「ログイン」を選択
   # 検索バーで「file-funeral」を検索
   ```
4. 認証情報を再入力して保存

### 問題: "Access Denied" エラー

**症状**: 接続テストや同期時に「Access Denied」エラーが発生する

**原因**: IAMユーザーに適切な権限がない

**解決方法**:

1. AWS Management Console → IAM → ユーザー → 該当ユーザーを選択
2. 「許可」タブで、S3バケットへのアクセス権限があるか確認
3. 必要な権限:
   - `s3:ListBucket`
   - `s3:GetObject`
   - `s3:PutObject`
   - `s3:DeleteObject`
4. 権限がない場合は、`AmazonS3FullAccess` ポリシーをアタッチするか、[SETUP_GUIDE.md](./SETUP_GUIDE.md)のカスタムポリシーを使用

### 問題: "Invalid credentials" エラー

**症状**: 「Invalid credentials」または「The security token included in the request is invalid」エラー

**解決方法**:

1. アクセスキーIDとシークレットアクセスキーが正しいか確認
2. AWS Management Console → IAM → ユーザー → セキュリティ認証情報で、アクセスキーが「アクティブ」状態か確認
3. アクセスキーが無効化されている場合は、新しいアクセスキーを作成
4. アクセスキーをコピーする際に、余分なスペースが入っていないか確認

---

## 接続エラー

### 問題: "Failed to connect to S3" エラー

**症状**: 「Failed to connect to S3」エラーが表示される

**原因**: ネットワーク接続、リージョン設定、バケット名の問題

**解決方法**:

1. **インターネット接続を確認**:
   ```bash
   ping aws.amazon.com
   ```

2. **リージョンが正しいか確認**:
   - S3バケットを作成したリージョンと、アプリで設定したリージョンが一致しているか確認
   - 例: バケットが東京リージョンにある場合、リージョンは `ap-northeast-1`

3. **バケット名が正しいか確認**:
   - 大文字・小文字を含めて完全に一致しているか確認
   - バケット名にスペースや特殊文字が入っていないか確認

4. **バケットが存在するか確認**:
   - AWS Management Console → S3 で、バケットが存在するか確認
   - バケットが削除されていないか確認

### 問題: "Network timeout" エラー

**症状**: 「Network timeout」または「Request timed out」エラー

**解決方法**:

1. インターネット接続が安定しているか確認
2. ファイアウォールやプロキシ設定を確認:
   - AWS S3への接続（HTTPSポート443）が許可されているか確認
3. VPN使用時は、VPNを無効にして再試行
4. 環境変数でタイムアウトを延長（上級者向け）:
   ```bash
   export AWS_TIMEOUT_MS=60000  # 60秒
   ```

### 問題: "Bucket does not exist" エラー

**症状**: 「The specified bucket does not exist」エラー

**解決方法**:

1. AWS Management Console → S3 で、バケットが存在するか確認
2. バケット名のスペルミスがないか確認
3. 正しいAWSアカウントでログインしているか確認
4. バケットが別のリージョンにある場合は、リージョン設定を変更

---

## 同期の問題

### 問題: ファイルがアップロードされない

**症状**: 同期を実行してもファイルがS3にアップロードされない

**解決方法**:

1. **ファイルが除外パターンに該当していないか確認**:
   - 以下のファイルは自動的にスキップされます:
     - 隠しファイル（`.` で始まる）
     - `.DS_Store`, `Thumbs.db`
     - `*.tmp`, `*.swp`, `*~`
   - 詳細: `src-tauri/src/commands.rs:513` の `should_skip_file` 関数

2. **ファイルサイズを確認**:
   - 5GBを超えるファイルはサポートされていません
   - エラーログで「File too large」を確認

3. **ファイルがシンボリックリンクでないか確認**:
   - シンボリックリンクは自動的にスキップされます
   - ログに「Skipping symbolic link」という警告が表示されます

4. **権限を確認**:
   - ファイルに読み取り権限があるか確認
   ```bash
   ls -l /path/to/file
   ```

### 問題: ファイルがダウンロードされない

**症状**: S3にあるファイルがローカルにダウンロードされない

**解決方法**:

1. **ローカルディレクトリに書き込み権限があるか確認**:
   ```bash
   ls -ld /path/to/sync/folder
   ```

2. **ディスク容量を確認**:
   ```bash
   df -h
   ```

3. **同期状態を確認**:
   - ファイルリストで、ファイルの状態が「Needs Download」になっているか確認
   - 「In Sync」の場合は、既にダウンロード済み

### 問題: 削除が同期されない

**症状**: ローカルまたはリモートでファイルを削除しても、反対側で削除されない

**原因**: 削除検出には同期履歴が必要

**解決方法**:

1. **少なくとも1回同期を実行**:
   - 初回同期後、同期履歴がSQLiteデータベースに保存されます
   - 2回目以降の同期で、削除が検出されるようになります

2. **同期履歴を確認**:
   ```bash
   npm run db:inspect
   ```

3. **同期履歴がない場合は、データベースを確認**:
   ```bash
   ls -la ~/.file-funeral/sync_history.db
   ```

### 問題: 競合が多発する

**症状**: 同期のたびに多くのファイルで競合が発生する

**原因**: 複数のPCで同時にファイルを編集している

**解決方法**:

1. **同期頻度を上げる**:
   - 自動同期を有効にする（起動時・終了時）
   - 作業開始前に手動で同期

2. **作業PCを分ける**:
   - 特定のファイルは特定のPCでのみ編集

3. **競合ファイルを手動で統合**:
   - `filename (PC名's conflicted copy YYYY-MM-DD).ext` ファイルを確認
   - 必要な変更を元のファイルにマージ
   - 競合ファイルを削除

---

## パフォーマンスの問題

### 問題: 同期が遅い

**症状**: 大量のファイルの同期に時間がかかる

**解決方法**:

1. **ファイル数を確認**:
   - 数千ファイル以上の場合、初回同期は時間がかかります
   - ログでプログレスを確認

2. **ネットワーク速度を確認**:
   ```bash
   # インターネット速度テスト
   curl -s https://raw.githubusercontent.com/sivel/speedtest-cli/master/speedtest.py | python -
   ```

3. **大容量ファイルを確認**:
   - 100MB以上のファイルはログに記録されます
   - ログで「Uploading large file」を確認

4. **除外パターンを設定**（将来のバージョン）:
   - v1.5で `.gitignore` 方式の除外パターンをサポート予定

### 問題: メモリ使用量が多い

**症状**: アプリのメモリ使用量が高い

**解決方法**:

1. **大量のファイルを一度に同期していないか確認**
2. **アプリを再起動**
3. **不要なファイルを除外**（将来のバージョン）

---

## データベースの問題

### 問題: 同期履歴が保存されない

**症状**: データベース確認コマンドでエラーが表示される

**解決方法**:

1. **データベースファイルが存在するか確認**:
   ```bash
   ls -la ~/.file-funeral/sync_history.db
   ```

2. **データベースディレクトリに書き込み権限があるか確認**:
   ```bash
   ls -ld ~/.file-funeral/
   ```

3. **データベースを再作成**:
   ```bash
   # バックアップを作成
   cp ~/.file-funeral/sync_history.db ~/.file-funeral/sync_history.db.backup

   # データベースを削除（次回起動時に再作成されます）
   rm ~/.file-funeral/sync_history.db
   ```

### 問題: "database is locked" エラー

**症状**: 「database is locked」エラーが表示される

**解決方法**:

1. **複数のアプリインスタンスが起動していないか確認**:
   ```bash
   ps aux | grep file-funeral
   ```

2. **アプリを完全に終了して再起動**

3. **データベースファイルのロックを解除**:
   ```bash
   # ロックファイルを削除
   rm ~/.file-funeral/sync_history.db-shm
   rm ~/.file-funeral/sync_history.db-wal
   ```

---

## ビルドとインストールの問題

### 問題: "command not found: cargo" エラー

**症状**: ビルド時に「cargo: command not found」エラー

**解決方法**:

1. **Rustをインストール**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **シェルを再起動**:
   ```bash
   source ~/.bashrc
   # または
   source ~/.zshrc
   ```

3. **Rustのバージョンを確認**:
   ```bash
   rustc --version
   # 1.92.0以降が必要
   ```

### 問題: Rustのコンパイルエラー

**症状**: `cargo build` や `npm run tauri:dev` でコンパイルエラー

**解決方法**:

1. **Rustのバージョンを更新**:
   ```bash
   rustup update
   ```

2. **依存関係を再ビルド**:
   ```bash
   cd src-tauri
   rm -rf target
   cargo clean
   cargo build
   ```

3. **Cargo.lockを削除して再生成**:
   ```bash
   cd src-tauri
   rm Cargo.lock
   cargo build
   ```

### 問題: "port 1420 already in use" エラー

**症状**: 開発サーバー起動時に「port 1420 already in use」エラー

**解決方法**:

1. **既存のプロセスを確認**:
   ```bash
   lsof -i :1420
   ```

2. **プロセスを終了**:
   ```bash
   kill -9 <PID>
   ```

3. **別のポートを使用**（vite.config.ts を編集）:
   ```typescript
   server: {
     port: 1421,  // 別のポート
     strictPort: true,
   }
   ```

---

## ログの確認方法

### アプリケーションログ

**開発モード**:
```bash
# ログ出力を確認しながら実行
npm run tauri:dev

# 詳細なログを有効化
RUST_LOG=debug npm run tauri:dev
```

**本番ビルド**:

macOSでは、ログは以下の場所に保存されます:
```bash
# Tauri plugin-logのログ
~/Library/Logs/com.tauri.dev/file-funeral.log

# コンソールアプリで確認
open -a Console
```

### デバッグモード

```bash
# デバッグモードで実行（詳細なログ出力）
DEBUG=true RUST_LOG=debug npm run tauri:dev
```

### データベースログ

```bash
# 同期履歴の確認
npm run db:inspect

# SQLiteシェルで詳細確認
npm run db:shell
```

---

## サポート

### 問題が解決しない場合

1. **GitHubのIssuesを検索**:
   - https://github.com/ohr486/file-funeral/issues
   - 同じ問題が報告されていないか確認

2. **新しいIssueを作成**:
   - タイトル: 問題の簡潔な説明
   - 内容に含める情報:
     - OS: macOS バージョン
     - Rustバージョン: `rustc --version`
     - Node.jsバージョン: `node --version`
     - エラーメッセージ（ログ全体）
     - 再現手順
     - 期待される動作
     - 実際の動作

3. **ログファイルを添付**:
   ```bash
   # ログをファイルに保存
   RUST_LOG=debug npm run tauri:dev 2>&1 | tee debug.log
   ```

### よくある質問

**Q: エラーログに個人情報が含まれていないか心配です**

A: ログには以下の情報が含まれる可能性があります:
- ファイルパス（ユーザー名を含む可能性あり）
- S3バケット名
- リージョン名

**含まれない情報**:
- アクセスキーID（一部マスクされます）
- シークレットアクセスキー（絶対に記録されません）
- ファイルの内容

Issue報告時は、個人情報を削除してから投稿してください。

**Q: デバッグモードでパフォーマンスが低下します**

A: デバッグモードでは詳細なログ出力により、パフォーマンスが低下します。通常使用時はデバッグモードを無効にしてください。

---

## 既知の問題

- **macOS 15 (Sequoia) での警告**: 一部のユーザーで証明書検証の警告が表示される場合があります。これはTauriの既知の問題です。
- **大量ファイル（10,000+）の同期**: 初回同期に時間がかかる場合があります。パフォーマンス最適化は今後のバージョンで改善予定です。
- **Windows/Linux対応**: v1.0ではmacOSのみサポート。Windows/Linux対応は今後のバージョンで予定されています。

---

このガイドで解決しない問題がある場合は、[GitHub Issues](https://github.com/ohr486/file-funeral/issues)で報告してください。
