# Model Context Protocol (MCP) 実装ガイド

このプロンプトは、LibrarianアプリケーションでのModel Context Protocol実装に特化したガイドです。

## MCP プロトコル概要

### プロトコル仕様

- **名称**: Model Context Protocol (MCP)
- **バージョン**: 2025-06-18
- **通信方式**: JSON-RPC 2.0 over HTTP
- **ポート範囲**: 9500-9599 (Librarian専用)

### 基本アーキテクチャ

```
AI Client (Claude Code, Cursor) ← HTTP JSON-RPC → MCP Server (Librarian)
```

## 必須実装メソッド

### 1. 初期化メソッド

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-06-18",
    "capabilities": {
      "prompts": {},
      "resources": {}
    },
    "clientInfo": {
      "name": "claude-code",
      "version": "1.0.0"
    }
  }
}
```

### 2. プロンプト関連メソッド

```rust
// prompts/list - プロンプト一覧取得
#[derive(Serialize)]
struct PromptsListResponse {
    prompts: Vec<PromptInfo>,
}

// prompts/get - 特定プロンプト取得
#[derive(Serialize)]
struct PromptsGetResponse {
    description: Option<String>,
    messages: Vec<PromptMessage>,
}
```

### 3. リソース関連メソッド

```rust
// resources/list - リソース一覧取得
#[derive(Serialize)]
struct ResourcesListResponse {
    resources: Vec<ResourceInfo>,
}

// resources/read - リソース内容取得
#[derive(Serialize)]
struct ResourcesReadResponse {
    contents: Vec<ResourceContent>,
}
```

## Axum実装パターン

### ルーター設定

```rust
use axum::{
    http::HeaderValue,
    http::Method,
    routing::post,
    Router,
};

fn create_mcp_router() -> Router {
    Router::new()
        .route("/", post(handle_jsonrpc))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::POST, Method::OPTIONS])
                .allow_headers([http::header::CONTENT_TYPE]),
        )
}
```

### JSON-RPC ハンドラー

```rust
async fn handle_jsonrpc(
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, JsonRpcError> {
    match request.method.as_str() {
        "initialize" => handle_initialize(request.params).await,
        "prompts/list" => handle_prompts_list(request.params).await,
        "prompts/get" => handle_prompts_get(request.params).await,
        "resources/list" => handle_resources_list(request.params).await,
        "resources/read" => handle_resources_read(request.params).await,
        _ => Err(JsonRpcError::method_not_found()),
    }
}
```

## エラーハンドリング

### JSON-RPC エラーコード

```rust
#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

impl JsonRpcError {
    fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    fn invalid_params() -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        }
    }
}
```

## .agent_library 統合

### ファイル構造解析

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct AgentIndex {
    name: String,
    description: String,
    mcp_endpoints: Vec<McpEndpoint>,
}

#[derive(Deserialize)]
struct McpEndpoint {
    id: String,
    label: String,
    description: String,
    prompt_file: String,
    category: String,
}
```

### プロンプト変換

```rust
impl From<McpEndpoint> for PromptInfo {
    fn from(endpoint: McpEndpoint) -> Self {
        Self {
            name: endpoint.id,
            description: Some(endpoint.description),
            arguments: vec![],
        }
    }
}
```

## ポート管理戦略

### 自動ポート割り当て

```rust
async fn find_available_port() -> Result<u16, String> {
    for port in 9500..9600 {
        if tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            return Ok(port);
        }
    }
    Err("No available ports in range 9500-9599".to_string())
}
```

### 複数サーバー管理

```rust
use std::collections::HashMap;
use tokio::task::JoinHandle;

struct McpServerManager {
    servers: HashMap<String, McpServerInstance>,
}

struct McpServerInstance {
    port: u16,
    handle: JoinHandle<()>,
    repository_path: String,
}
```

## トラブルシューティング

### よくある問題と解決法

1. **CORS エラー**

   ```rust
   // 適切なCORSヘッダーを設定
   .allow_origin("*".parse::<HeaderValue>().unwrap())
   .allow_methods([Method::POST, Method::OPTIONS])
   ```

2. **JSON-RPC フォーマットエラー**

   ```rust
   // リクエスト形式を厳密にチェック
   if request.jsonrpc != "2.0" {
       return Err(JsonRpcError::invalid_request());
   }
   ```

3. **ポート競合**
   ```rust
   // 起動前にポート可用性をチェック
   let port = find_available_port().await?;
   ```

## テスト方法

### curl でのテスト

```bash
# 初期化テスト
curl -X POST http://localhost:9500 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'

# プロンプト一覧取得
curl -X POST http://localhost:9500 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"prompts/list"}'
```

### 統合テスト

```rust
#[tokio::test]
async fn test_mcp_initialize() {
    let app = create_mcp_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

## パフォーマンス最適化

### 非同期処理

```rust
// ファイル読み込みを非同期化
async fn load_prompt_content(file_path: &str) -> Result<String, std::io::Error> {
    tokio::fs::read_to_string(file_path).await
}

// キャッシュ機能
use std::sync::Arc;
use tokio::sync::RwLock;

type PromptCache = Arc<RwLock<HashMap<String, String>>>;
```

## セキュリティ考慮事項

### 入力検証

```rust
fn validate_prompt_name(name: &str) -> bool {
    // ディレクトリトラバーサル攻撃を防止
    !name.contains("..") && !name.contains("/") && !name.contains("\\")
}

fn sanitize_file_path(base_path: &str, requested_file: &str) -> Option<PathBuf> {
    let path = Path::new(base_path).join(requested_file);
    if path.starts_with(base_path) {
        Some(path)
    } else {
        None
    }
}
```

このガイドに従って、堅牢で効率的なMCPサーバーを実装してください。
