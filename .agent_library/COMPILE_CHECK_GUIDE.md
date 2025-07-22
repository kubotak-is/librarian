# コンパイルエラー防止ガイドライン

## 目的

Rustコードを変更する際、事前にコンパイルエラーを防ぐためのガイドラインです。

## 基本原則

### 1. 依存関係の確認

- 新しいクレートを使用する前に、必ずCargo.tomlで依存関係が正しく定義されているか確認する
- 使用するfeatureが有効になっているかチェックする
- 例：`moka = { version = "0.12", features = ["future", "sync"] }`

### 2. インポート文の検証

- 使用するモジュールのパスが正確か確認する
- 非同期機能（`moka::future`）と同期機能（`moka::sync`）を混同しない
- `use`文で指定するパスがクレートのAPIドキュメントと一致するか確認する

### 3. 型の整合性

- ジェネリクス型の引数が正しく指定されているか確認する
- async/awaitパターンとblocking操作を混同しない
- 例：`Cache<String, CachedResponse>`の型引数を正確に指定する

## 実装前チェックリスト

### コード変更前

1. [ ] 変更対象ファイルのexisting importsを確認
2. [ ] 新規追加するクレートがCargo.tomlに記載されているか確認
3. [ ] 使用予定のAPIがクレートのドキュメントに存在するか確認

### 実装中

1. [ ] async関数内でasync APIを使用しているか確認
2. [ ] synchronous関数内でblocking APIを使用しているか確認
3. [ ] ライフタイムやオーナーシップの問題がないか確認

### 実装後

1. [ ] `cargo check`でコンパイルエラーがないか確認
2. [ ] 警告（warning）も可能な限り解消する
3. [ ] テストが正常に動作するか確認

## よくあるエラーパターン

### 1. 存在しないモジュールのインポート

```rust
// ❌ 間違い
use moka::sync::Cache; // moka::syncが存在しない場合

// ✅ 正しい
use moka::future::Cache; // または適切な代替手段
```

### 2. 非同期/同期APIの混同

```rust
// ❌ 間違い
let value = cache.get(&key); // 非同期コンテキストで同期API

// ✅ 正しい
let value = cache.get(&key).await; // 非同期API
```

### 3. 型の不一致

```rust
// ❌ 間違い
static CACHE: Lazy<Cache<String>> = ...; // ジェネリクス引数不足

// ✅ 正しい
static CACHE: Lazy<Cache<String, Value>> = ...; // 完全な型指定
```

## 代替実装戦略

### 外部クレートが利用できない場合

1. 標準ライブラリの`HashMap`を使用
2. `std::sync::Mutex`でthread-safetyを確保
3. TTL機能が必要な場合は`std::time::Instant`でタイムスタンプ管理

### 例：シンプルなキャッシュ実装

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use once_cell::sync::Lazy;

static CACHE: Lazy<Mutex<HashMap<String, (Value, Instant)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
```

## 実践例

### 問題のあったmoka実装

```rust
// 問題：moka::sync::Cacheが存在しない
use moka::sync::Cache;
static RESPONSE_CACHE: Lazy<Cache<String, CachedResponse>> = ...;
```

### 修正後のHashMap実装

```rust
// 解決：標準ライブラリを使用
use std::collections::HashMap;
use std::sync::Mutex;
type ResponseCacheEntry = (CachedResponse, Instant);
static RESPONSE_CACHE: Lazy<Mutex<HashMap<String, ResponseCacheEntry>>> = ...;
```

## まとめ

- **事前確認**：新しい依存関係やAPIを使用する前に必ずドキュメントを確認
- **段階的実装**：小さな変更を積み重ねて都度コンパイルチェック
- **代替案準備**：外部クレートに問題がある場合の標準ライブラリベースの代替実装を検討
