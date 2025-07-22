# Rust 品質管理ガイド

## 概要

Rustプロジェクト（特にTauriバックエンド）での品質管理ツールの設定と使用方法

## 必要なツール

### 1. Clippy

Rustの静的解析ツール（デフォルトでインストール済み）

```bash
# 実行
cargo clippy

# より厳しいチェック
cargo clippy -- -D warnings

# 特定のlintを有効化
cargo clippy -- -W clippy::all -W clippy::pedantic
```

### 2. rustfmt

Rustコードフォーマッター（デフォルトでインストール済み）

```bash
# 実行
cargo fmt

# チェックのみ（フォーマットしない）
cargo fmt -- --check
```

### 3. cargo test

テスト実行

```bash
# 全テスト実行
cargo test

# 特定のテスト実行
cargo test test_name

# テストカバレッジ
cargo install cargo-tarpaulin
cargo tarpaulin
```

### 4. cargo check

高速な型チェック

```bash
# 型チェックのみ
cargo check

# 全ての対象でチェック
cargo check --all-targets
```

## Cargo.toml 品質設定

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

# 過度に厳しいlintを無効化
module_name_repetitions = "allow"
missing_errors_doc = "allow"
```

## rustfmt設定 (rustfmt.toml)

```toml
# 最大行長
max_width = 100

# インデント
hard_tabs = false
tab_spaces = 4

# インポート整理
reorder_imports = true
group_imports = "StdExternalCrate"

# 関数定義
fn_args_layout = "Tall"
brace_style = "SameLineWhere"

# コメント
comment_width = 80
wrap_comments = true

# その他
trailing_comma = "Vertical"
```

## 開発ワークフロー

### コミット前チェック

```bash
# フォーマットチェック
cargo fmt -- --check

# Clippyチェック
cargo clippy -- -D warnings

# テスト実行
cargo test

# ビルドチェック
cargo check --all-targets
```

### 品質スクリプト作成

`Makefile` または `scripts/quality.sh`:

```bash
#!/bin/bash
set -e

echo "🔍 Running Rust quality checks..."

echo "📝 Checking format..."
cargo fmt -- --check

echo "📎 Running Clippy..."
cargo clippy --all-targets -- -D warnings

echo "🧪 Running tests..."
cargo test

echo "🏗️ Checking compilation..."
cargo check --all-targets

echo "✅ All quality checks passed!"
```

## Tauri固有の設定

### Cargo.toml でのTauri設定

```toml
[dependencies]
tauri = { version = "2", features = [] }
# 不要なfeatureは含めない

[build-dependencies]
tauri-build = { version = "2", features = [] }

# デバッグビルドの最適化
[profile.dev]
opt-level = 1

# リリースビルドの最適化
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
strip = true
```

## よくある問題と解決方法

### 1. Clippyの警告対応

```rust
// ❌ 非効率なString操作
let result = format!("{}", value.to_string());

// ✅ 効率的な変換
let result = value.to_string();

// ❌ 不要なclone
let data = expensive_data.clone();
process(data);

// ✅ 借用を使用
process(&expensive_data);
```

### 2. エラーハンドリング

```rust
// ❌ unwrapの多用
let file = std::fs::File::open("config.txt").unwrap();

// ✅ 適切なエラーハンドリング
let file = std::fs::File::open("config.txt")
    .map_err(|e| format!("Failed to open config: {}", e))?;
```

### 3. 非同期コードの品質

```rust
// ❌ 不要なasync
async fn sync_operation() -> i32 {
    42
}

// ✅ 同期関数として定義
fn sync_operation() -> i32 {
    42
}
```

## ベストプラクティス

### 1. ドキュメンテーション

````rust
/// MCPサーバーの状態を管理する構造体
///
/// # Examples
///
/// ```
/// let state = McpServerState::new();
/// ```
pub struct McpServerState {
    // ...
}
````

### 2. エラー型の定義

```rust
#[derive(Debug, thiserror::Error)]
pub enum LibrarianError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parsing error: {0}")]
    Parse(String),
}
```

### 3. 設定管理

```rust
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub port: u16,
    pub host: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 9500,
            host: "127.0.0.1".to_string(),
        }
    }
}
```

## CI/CD統合

### GitHub Actions設定例

```yaml
- name: Run Rust quality checks
  run: |
    cargo fmt -- --check
    cargo clippy --all-targets -- -D warnings
    cargo test --all-targets
    cargo check --all-targets
```

## 品質メトリクス

- Clippy警告: 0個
- コンパイル警告: 0個
- テストカバレッジ: 80%以上
- ドキュメンテーション: パブリックAPI 100%
- unsafe コード: 0個（forbid設定）
