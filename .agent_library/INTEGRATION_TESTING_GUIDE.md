# 統合テスト実装ガイド

## 目的

Librarian アプリケーションでのE2Eテストと統合テストの実装方法とベストプラクティス

## E2Eテスト（End-to-End Testing）

### Playwright環境設定

#### playwright.config.ts

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    // 他のブラウザ設定...
  ],

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
});
```

### アプリケーション基本動作のテスト

```typescript
import { test, expect } from '@playwright/test';

test.describe('Librarian Application', () => {
  test('should load the main application', async ({ page }) => {
    await page.goto('/');

    // アプリケーションの読み込み確認
    await expect(page).toHaveTitle(/Librarian/);

    // メインナビゲーション要素の確認
    await expect(page.getByText('Repository')).toBeVisible();
    await expect(page.getByText('Settings')).toBeVisible();
  });

  test('should display welcome message when no repositories', async ({ page }) => {
    await page.goto('/');

    // 空の状態の表示確認
    await expect(page.getByText('リポジトリが見つかりません')).toBeVisible();
    await expect(page.getByText('新しいリポジトリを追加')).toBeVisible();
  });
});
```

### Tauri APIモックの設定

```typescript
test('should add a new repository', async ({ page }) => {
  await page.goto('/');

  // Tauri APIのモック設定
  await page.addInitScript((repoPath) => {
    window.__TAURI_INTERNALS__ = {
      invoke: async (command: string, args: any) => {
        if (command === 'validate_agent_library') {
          return true;
        }
        if (command === 'parse_agent_library') {
          return {
            index: {
              name: 'Test Repository',
              description: 'Test repository for E2E testing',
              version: '1.0.0',
            },
            prompts: [
              {
                id: 'test_prompt',
                title: 'Test Prompt',
                description: 'A test prompt for E2E testing',
                content: '# Test Prompt\n\nThis is a test prompt.',
                filePath: `${repoPath}/.agent_library/test_prompt.md`,
              },
            ],
          };
        }
        return Promise.resolve();
      },
    };
  }, testRepoPath);

  // リポジトリ追加ボタンをクリック
  await page.getByText('新しいリポジトリを追加').click();

  // リポジトリがリストに表示されることを確認
  await expect(page.getByText('Test Repository')).toBeVisible();
});
```

### 複雑なワークフローのテスト

```typescript
test('should complete full MCP workflow', async ({ page }) => {
  // 1. リポジトリ追加
  // 2. プロンプト表示
  // 3. MCPサーバー起動
  // 4. プロンプト編集
  // 5. 保存確認

  await page.goto('/');

  // モック設定...

  // ワークフロー実行
  await page.getByText('Test Repository').click();
  await expect(page.getByText('Test Prompt')).toBeVisible();

  await page.getByRole('button', { name: /MCP.*開始/ }).click();
  await expect(page.getByText(/running|実行中/)).toBeVisible();

  await page.getByText('編集').click();
  await page.getByRole('textbox').fill('# Modified Prompt\n\nModified content');
  await page.getByText('保存').click();

  await expect(page.getByText('保存しました')).toBeVisible();
});
```

## 統合テスト（Integration Testing）

### MCPサーバー統合テストの構造

```rust
use axum::http::StatusCode;
use serde_json::json;
use tower::util::ServiceExt;
use tempfile::TempDir;

use librarian::agent_library::AgentLibraryParser;
use librarian::mcp::{create_mcp_router, McpServerState};
use librarian::mcp::types::{JsonRpcRequest, JsonRpcResponse};

#[tokio::test]
async fn test_mcp_server_full_workflow() {
    // テスト環境のセットアップ
    let temp_dir = TempDir::new().unwrap();
    create_test_agent_library(temp_dir.path()).unwrap();

    let library = AgentLibraryParser::parse(temp_dir.path()).unwrap();

    let state = McpServerState::new();
    {
        let mut libraries = state.agent_libraries.write().await;
        libraries.push(library);
    }

    let app = create_mcp_router().with_state(state);

    // 1. Initialize テスト
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {}
        })),
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(serde_json::to_string(&init_request).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // レスポンスの検証
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let init_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

    assert!(init_response.error.is_none());
    assert!(init_response.result.is_some());

    // プロトコルバージョンの確認
    let result = init_response.result.unwrap();
    assert_eq!(result["protocolVersion"], "2025-06-18");

    // 以降、prompts/list, prompts/get, resources のテスト...
}
```

### エラーハンドリングの統合テスト

```rust
#[tokio::test]
async fn test_mcp_server_error_handling() {
    let state = McpServerState::new();
    let app = create_mcp_router().with_state(state);

    // 無効なメソッドのテスト
    let invalid_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "invalid/method".to_string(),
        params: None,
    };

    let response = app
        .clone()
        .oneshot(/* リクエスト設定 */)
        .await
        .unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let error_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

    assert!(error_response.error.is_some());
    let error = error_response.error.unwrap();
    assert_eq!(error.code, -32601);
    assert_eq!(error.message, "Method not found");
}
```

### キャッシュ動作の統合テスト

```rust
#[tokio::test]
async fn test_mcp_server_caching() {
    // テストデータのセットアップ
    let temp_dir = TempDir::new().unwrap();
    create_test_agent_library(temp_dir.path()).unwrap();
    let library = AgentLibraryParser::parse(temp_dir.path()).unwrap();

    let state = McpServerState::new();
    {
        let mut libraries = state.agent_libraries.write().await;
        libraries.push(library);
    }

    let app = create_mcp_router().with_state(state);

    // 1回目のリクエスト
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "prompts/list".to_string(),
        params: None,
    };

    let response1 = make_request(&app, &request).await;
    let response2 = make_request(&app, &request).await;

    // 両方のレスポンスが成功していることを確認
    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response2.status(), StatusCode::OK);

    // レスポンス内容が一致することを確認（キャッシュが動作）
    let body1 = extract_response_body(response1).await;
    let body2 = extract_response_body(response2).await;

    assert_eq!(body1.result, body2.result);
}
```

### テストヘルパー関数

```rust
fn create_test_agent_library(dir: &std::path::Path) -> anyhow::Result<()> {
    use std::fs;

    let agent_lib_dir = dir.join(".agent_library");
    fs::create_dir_all(&agent_lib_dir)?;

    let index_content = r#"
name: "Integration Test Library"
description: "Test library for integration testing"
version: "1.0.0"
mcp_endpoints:
  - id: "test_prompt"
    label: "Test Prompt"
    description: "A test prompt for integration testing"
    prompt_file: "test_prompt.md"
"#;
    fs::write(agent_lib_dir.join("agent_index.yml"), index_content)?;

    let prompt_content = "# Test Prompt\n\nThis is a test prompt.";
    fs::write(agent_lib_dir.join("test_prompt.md"), prompt_content)?;

    Ok(())
}

async fn make_request(
    app: &Router,
    request: &JsonRpcRequest,
) -> axum::http::Response<axum::body::Body> {
    app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(serde_json::to_string(request).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap()
}
```

## テスト実行とCI/CD統合

### テスト実行コマンド

```bash
# E2Eテスト
npm run test:e2e

# UIモードでのE2Eテスト
npm run test:e2e:ui

# ヘッドレスモードでのE2Eテスト
npm run test:e2e:headed

# 統合テスト
cargo test --test integration

# 全テスト実行
npm run test && cargo test
```

### GitHub Actions統合例

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright
        run: npx playwright install

      - name: Run E2E tests
        run: npm run test:e2e

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run integration tests
        run: cd src-tauri && cargo test --test integration
```

## トラブルシューティング

### よくある問題と解決方法

#### 1. E2Eテストのタイムアウト

```typescript
// より長いタイムアウトを設定
await expect(page.getByText('Loading...')).toBeVisible({ timeout: 10000 });
```

#### 2. Tauri APIモックが動作しない

```typescript
// ページ読み込み前にモックを設定
await page.addInitScript(() => {
  // モック設定をここで実行
});
```

#### 3. 統合テストでのHTTPクライアントエラー

```rust
// Tower utilのServiceExtを使用
use tower::util::ServiceExt;

let response = app.oneshot(request).await.unwrap();
```

## ベストプラクティス

### E2Eテスト

- **安定性**: 要素の選択にdata-testidを使用
- **独立性**: 各テストは独立して実行可能
- **リアルなシナリオ**: 実際のユーザー操作をシミュレート

### 統合テスト

- **完全なワークフロー**: APIの連携動作を検証
- **エラーケース**: 正常系と異常系の両方をテスト
- **パフォーマンス**: レスポンス時間とキャッシュ効果を測定

### 共通

- **テスト環境の分離**: プロダクション環境に影響しない
- **クリーンアップ**: テスト後のリソース削除
- **継続的実行**: CI/CDパイプラインでの自動実行
