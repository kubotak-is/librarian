# デバッグ・トラブルシューティングガイド

## 概要

Tauri + Svelte 5 + Rustアプリケーションでの一般的な問題とデバッグ方法

## 開発環境セットアップの問題

### 1. Rustツールチェーンが見つからない

```bash
# エラー: cargo not found
# 解決策: Rustupでインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 確認
cargo --version
rustc --version
```

### 2. Node.js依存関係の問題

```bash
# エラー: package not found
# 解決策: 依存関係の再インストール
rm -rf node_modules package-lock.json
npm install

# キャッシュクリア
npm cache clean --force
```

### 3. Tauriビルドエラー

```bash
# エラー: tauri build failed
# 解決策: 段階的確認
npm run build  # フロントエンドビルド確認
cd src-tauri && cargo check  # Rustコンパイル確認
cd src-tauri && cargo build  # Rustビルド確認
```

## Tauriアプリケーション固有の問題

### 1. Tauriコマンドが見つからない

```rust
// エラー: Command not found
// 原因: invoke_handlerに登録されていない

// ✅ 解決策: lib.rsで登録確認
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        your_command_name  // ここに追加
    ])
```

### 2. パーミッションエラー

```json
// エラー: Permission denied
// 解決策: capabilities/default.json で権限追加
{
  "permissions": ["core:default", "dialog:allow-open", "fs:allow-read-file"]
}
```

### 3. フロントエンド-バックエンド通信エラー

```typescript
// デバッグ用ログ追加
try {
  const result = await invoke('command_name', { param: value });
  console.log('Success:', result);
} catch (error) {
  console.error('Tauri command failed:', error);
  // エラーの詳細を確認
  console.error('Error type:', typeof error);
  console.error('Error details:', JSON.stringify(error));
}
```

## Svelte 5固有の問題

### 1. Svelte 5 API移行エラー

```javascript
// ❌ 古いAPI（エラーの原因）
import Component from './Component.svelte';
new Component({ target: document.body });

// ✅ 新しいAPI
import { mount } from 'svelte';
import Component from './Component.svelte';
mount(Component, { target: document.body });
```

### 2. Runes使用時のエラー

```javascript
// ❌ reactive宣言とrunesの混在
let count = 0;
$: doubled = count * 2; // 古いreactive構文

// ✅ runesに統一
let count = $state(0);
let doubled = $derived(count * 2);
```

### 3. TypeScript型エラー

```typescript
// Props型定義の問題
interface Props {
  title: string;
  items?: string[];
}

// ✅ Svelte 5での正しいprops定義
let { title, items = [] }: Props = $props();
```

## MCPサーバー関連の問題

### 1. ポート競合エラー

```bash
# エラー: Address already in use
# 確認: ポート使用状況
lsof -i :9500

# 解決策: プロセス終了または別ポート使用
kill -9 <PID>
# または
# アプリケーションで自動ポート割り当て使用
```

### 2. JSON-RPC通信エラー

```bash
# デバッグ: curlでエンドポイントテスト
curl -X POST http://localhost:9500 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize"}'

# レスポンス確認
# 正常: {"jsonrpc":"2.0","id":1,"result":{...}}
# エラー: {"jsonrpc":"2.0","id":1,"error":{...}}
```

### 3. CORS問題

```rust
// デバッグ: レスポンスヘッダー確認
let mut headers = HeaderMap::new();
headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
headers.insert("Access-Control-Allow-Methods", "POST, GET, OPTIONS".parse().unwrap());
headers.insert("Access-Control-Allow-Headers", "Content-Type".parse().unwrap());

println!("CORS headers set: {:?}", headers);
```

## 品質チェックエラー

### 1. ESLintエラー

```bash
# エラー確認
npm run lint

# よくあるエラーと解決策
# - Unused variable: 変数名を_で始める
let _unusedVar = something;

# - Missing semicolon: セミコロン追加
const value = getValue();

# - Prefer const: letをconstに変更
const immutableValue = 'hello';
```

### 2. TypeScriptエラー

```bash
# 型チェック
npm run svelte-check

# よくあるエラー
# - Property does not exist: 型定義追加
interface MyObject {
  property: string;
}

# - Type 'undefined' is not assignable: オプショナル対応
const value: string | undefined = getValue();
if (value) {
  // use value
}
```

### 3. Clippyエラー

```bash
# Rust静的解析
cd src-tauri && cargo clippy

# よくある警告と修正
# - Unnecessary clone: clone()削除
let data = expensive_data; // .clone()を削除

# - Use of unwrap: 適切なエラーハンドリング
let result = operation().map_err(|e| format!("Error: {}", e))?;
```

## パフォーマンスデバッグ

### 1. Svelte DevTools

```javascript
// 開発環境でのみデバッグ情報出力
if (import.meta.env.DEV) {
  console.log('Component state:', { count, items });
}

// パフォーマンス測定
console.time('expensive-operation');
expensiveOperation();
console.timeEnd('expensive-operation');
```

### 2. Rustパフォーマンス

```rust
// トレースログでパフォーマンス測定
use tracing::{info, span, Level};

let span = span!(Level::INFO, "expensive_operation");
let _enter = span.enter();

// 処理時間の測定
let start = std::time::Instant::now();
expensive_operation();
let duration = start.elapsed();
info!("Operation took: {:?}", duration);
```

### 3. ビルドサイズ最適化

```bash
# バンドルサイズ分析
npm run build
du -sh dist/*

# Rustバイナリサイズ確認
cd src-tauri && cargo build --release
ls -lh target/release/librarian-app
```

## ログ設定とデバッグ出力

### 1. Rustログ設定

```rust
// Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"

// main.rs
fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("Application starting");
    // ...
}
```

### 2. フロントエンドログ

```typescript
// 環境別ログレベル
const log = {
  debug: (msg: string, data?: any) => {
    if (import.meta.env.DEV) {
      console.log(`[DEBUG] ${msg}`, data);
    }
  },
  error: (msg: string, error?: any) => {
    console.error(`[ERROR] ${msg}`, error);
  },
  info: (msg: string, data?: any) => {
    console.info(`[INFO] ${msg}`, data);
  },
};
```

## トラブルシューティングチェックリスト

### 開発環境問題

- [ ] Rustツールチェーンがインストールされているか
- [ ] Node.js/npmが最新版か
- [ ] 依存関係が正しくインストールされているか
- [ ] ポートが競合していないか

### ビルド問題

- [ ] TypeScript型エラーがないか
- [ ] ESLint/Clippyエラーがないか
- [ ] 必要なファイルが存在するか
- [ ] Tauriコマンドが正しく登録されているか

### 実行時問題

- [ ] パーミッションが適切に設定されているか
- [ ] MCPサーバーが起動しているか
- [ ] CORS設定が正しいか
- [ ] ログにエラーが出力されていないか

### 品質問題

- [ ] `make quality`がエラーなく完了するか
- [ ] テストがパスするか
- [ ] ビルドが成功するか
- [ ] 警告が解決されているか

## 緊急時の対処法

### 1. 完全クリーンビルド

```bash
# 全てクリーンアップ
make clean
rm -rf node_modules package-lock.json
rm -rf src-tauri/target

# 再インストール・再ビルド
npm install
npm run tauri build
```

### 2. 設定ファイル復旧

```bash
# Gitから設定ファイルを復旧
git checkout HEAD -- package.json
git checkout HEAD -- src-tauri/Cargo.toml
git checkout HEAD -- tsconfig.json
```

### 3. バックアップからの復旧

```bash
# 最新の動作するコミットに戻る
git log --oneline -10  # 最近のコミット確認
git reset --hard <working_commit_hash>
```
