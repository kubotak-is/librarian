# Tauri プロジェクト設定ガイド

## 概要

Tauri + Svelte 5 + TypeScript + TailwindCSSでのプロジェクト設定と構成方法

## プロジェクト初期化

### 1. 新規Tauriプロジェクト作成

```bash
# Tauri CLI インストール
npm install -g @tauri-apps/cli

# プロジェクト作成
npm create tauri-app@latest my-app

# または既存のフロントエンドプロジェクトにTauri追加
cd existing-project
npm install @tauri-apps/api @tauri-apps/cli
```

### 2. Svelte 5 + TypeScript設定

```bash
# 依存関係インストール
npm install svelte@^5.0.0 @sveltejs/vite-plugin-svelte
npm install -D typescript @tsconfig/svelte
```

### 3. TailwindCSS設定

```bash
# TailwindCSS インストール
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

## 設定ファイル

### tauri.conf.json

```json
{
  "$schema": "../gen/schemas/config.schema.json",
  "productName": "librarian",
  "version": "0.1.0",
  "identifier": "com.example.librarian",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "app": {
    "windows": [
      {
        "title": "librarian",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  }
}
```

### vite.config.ts

```typescript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
});
```

### svelte.config.js

```javascript
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    runes: true, // Svelte 5のrunesを有効化
  },
};
```

### tailwind.config.js

```javascript
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {},
  },
  plugins: [],
};
```

## Capabilities設定

### src-tauri/capabilities/default.json

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": ["core:default", "opener:default", "dialog:default", "dialog:allow-open"]
}
```

## Tauriコマンド実装

### Rustバックエンド (src-tauri/src/lib.rs)

```rust
// 基本的なコマンド例
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 非同期コマンド例
#[tauri::command]
async fn async_operation(data: String) -> Result<String, String> {
    // 何らかの非同期処理
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(format!("Processed: {}", data))
}

// エラーハンドリング付きコマンド
#[tauri::command]
fn risky_operation() -> Result<String, String> {
    // 失敗する可能性のある処理
    std::fs::read_to_string("config.txt")
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            async_operation,
            risky_operation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### フロントエンド呼び出し (TypeScript)

```typescript
import { invoke } from '@tauri-apps/api/core';

// 基本的な呼び出し
const message = await invoke<string>('greet', { name: 'World' });

// エラーハンドリング
try {
  const result = await invoke<string>('risky_operation');
  console.log(result);
} catch (error) {
  console.error('Command failed:', error);
}

// 型安全な呼び出し
interface GreetArgs {
  name: string;
}

const greetUser = async (args: GreetArgs): Promise<string> => {
  return await invoke('greet', args);
};
```

## 開発ワークフロー

### 開発サーバー起動

```bash
# フロントエンドとバックエンドを同時起動
npm run tauri dev

# フロントエンドのみ
npm run dev

# バックエンドのみ（Rust）
cd src-tauri && cargo run
```

### ビルドとパッケージング

```bash
# 開発ビルド
npm run tauri build -- --debug

# リリースビルド
npm run tauri build

# 特定のターゲット向けビルド
npm run tauri build -- --target x86_64-pc-windows-msvc
```

## よくある問題と解決方法

### 1. CORS エラー

```rust
// AxumでのCORS設定例
use axum::http::HeaderMap;

async fn handler() -> (HeaderMap, Json<Response>) {
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "POST, GET, OPTIONS".parse().unwrap());
    (headers, Json(response))
}
```

### 2. パーミッションエラー

```json
// capabilities/default.json にパーミッション追加
{
  "permissions": ["core:default", "dialog:allow-open", "fs:allow-read-file"]
}
```

### 3. ビルドエラー

```bash
# キャッシュクリア
rm -rf node_modules dist src-tauri/target
npm install
npm run tauri build
```

## ベストプラクティス

### 1. エラーハンドリング

```rust
// 統一されたエラー型を定義
#[derive(Debug, serde::Serialize)]
pub struct CommandError {
    pub message: String,
    pub code: String,
}

impl From<std::io::Error> for CommandError {
    fn from(error: std::io::Error) -> Self {
        CommandError {
            message: error.to_string(),
            code: "IO_ERROR".to_string(),
        }
    }
}
```

### 2. 設定管理

```rust
// 設定ファイルの読み込み
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub theme: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = tauri::api::path::app_config_dir(&tauri::generate_context!())?
            .join("config.json");

        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
}
```

### 3. 状態管理

```rust
// グローバル状態管理
use std::sync::{Arc, Mutex};

struct AppState {
    pub data: Arc<Mutex<HashMap<String, String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
```

## デバッグとテスト

### 1. ログ設定

```rust
// env_loggerを使用
use log::{info, warn, error};

fn main() {
    env_logger::init();
    info!("Application starting");

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. テストコマンド

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("Test");
        assert!(result.contains("Test"));
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_operation("test".to_string()).await;
        assert!(result.is_ok());
    }
}
```
