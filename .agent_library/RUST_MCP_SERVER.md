# Rust MCP サーバー実装ガイド

## 概要

AxumとJSON-RPCを使ったModel Context Protocol (MCP) サーバーの実装方法

## MCP仕様概要

### プロトコル基本情報

- **プロトコル**: JSON-RPC 2.0 over HTTP
- **メソッド**: initialize, prompts/list, prompts/get, resources/list, resources/read
- **ポート**: 9500-9599 (Librarian専用範囲)
- **CORS**: 有効化必須

## Axumサーバー設定

### 1. 依存関係設定

```toml
# Cargo.toml
[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1.0", features = ["v4"] }
anyhow = "1.0"
```

### 2. 基本サーバー構造

```rust
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// JSON-RPC 2.0 基本構造
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}
```

### 3. MCPサーバー状態管理

```rust
#[derive(Debug, Clone)]
pub struct McpServerState {
    pub agent_libraries: Arc<RwLock<Vec<AgentLibrary>>>,
    pub server_info: ServerInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

impl McpServerState {
    pub fn new() -> Self {
        Self {
            agent_libraries: Arc::new(RwLock::new(Vec::new())),
            server_info: ServerInfo {
                name: "librarian-mcp-server".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }
}
```

## MCPエンドポイント実装

### 1. initialize メソッド

```rust
async fn handle_initialize(
    params: Option<serde_json::Value>,
    state: Arc<McpServerState>,
) -> Result<serde_json::Value, JsonRpcError> {
    #[derive(Serialize)]
    struct InitializeResult {
        #[serde(rename = "protocolVersion")]
        protocol_version: String,
        capabilities: Capabilities,
        #[serde(rename = "serverInfo")]
        server_info: ServerInfo,
    }

    #[derive(Serialize)]
    struct Capabilities {
        prompts: PromptCapabilities,
        resources: ResourceCapabilities,
    }

    #[derive(Serialize)]
    struct PromptCapabilities {
        #[serde(rename = "listChanged")]
        list_changed: bool,
    }

    #[derive(Serialize)]
    struct ResourceCapabilities {
        subscribe: bool,
        #[serde(rename = "listChanged")]
        list_changed: bool,
    }

    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: Capabilities {
            prompts: PromptCapabilities {
                list_changed: false,
            },
            resources: ResourceCapabilities {
                subscribe: false,
                list_changed: false,
            },
        },
        server_info: state.server_info.clone(),
    };

    Ok(serde_json::to_value(result)?)
}
```

### 2. prompts/list メソッド

```rust
async fn handle_prompts_list(
    params: Option<serde_json::Value>,
    state: Arc<McpServerState>,
) -> Result<serde_json::Value, JsonRpcError> {
    #[derive(Serialize)]
    struct PromptsListResult {
        prompts: Vec<PromptInfo>,
    }

    #[derive(Serialize)]
    struct PromptInfo {
        name: String,
        description: Option<String>,
        arguments: Option<Vec<PromptArgument>>,
    }

    #[derive(Serialize)]
    struct PromptArgument {
        name: String,
        description: Option<String>,
        required: Option<bool>,
    }

    let libraries = state.agent_libraries.read().await;
    let mut prompts = Vec::new();

    for library in libraries.iter() {
        for prompt in &library.prompts {
            prompts.push(PromptInfo {
                name: prompt.name.clone(),
                description: Some(prompt.description.clone()),
                arguments: None,
            });
        }
    }

    let result = PromptsListResult { prompts };
    Ok(serde_json::to_value(result)?)
}
```

### 3. prompts/get メソッド

```rust
async fn handle_prompts_get(
    params: Option<serde_json::Value>,
    state: Arc<McpServerState>,
) -> Result<serde_json::Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct PromptsGetParams {
        name: String,
        arguments: Option<serde_json::Value>,
    }

    #[derive(Serialize)]
    struct PromptsGetResult {
        description: Option<String>,
        messages: Vec<PromptMessage>,
    }

    #[derive(Serialize)]
    struct PromptMessage {
        role: String,
        content: PromptContent,
    }

    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum PromptContent {
        #[serde(rename = "text")]
        Text { text: String },
    }

    let params: PromptsGetParams = serde_json::from_value(
        params.ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        })?
    )?;

    let libraries = state.agent_libraries.read().await;

    for library in libraries.iter() {
        for prompt in &library.prompts {
            if prompt.name == params.name {
                let result = PromptsGetResult {
                    description: Some(prompt.description.clone()),
                    messages: vec![PromptMessage {
                        role: "user".to_string(),
                        content: PromptContent::Text {
                            text: prompt.content.clone(),
                        },
                    }],
                };
                return Ok(serde_json::to_value(result)?);
            }
        }
    }

    Err(JsonRpcError {
        code: -32602,
        message: format!("Prompt '{}' not found", params.name),
        data: None,
    })
}
```

### 4. CORS対応

```rust
async fn handle_jsonrpc(
    State(state): State<Arc<McpServerState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<(HeaderMap, Json<JsonRpcResponse>), StatusCode> {
    // CORS ヘッダー設定
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert(
        "Access-Control-Allow-Methods",
        "POST, GET, OPTIONS".parse().unwrap()
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization".parse().unwrap()
    );

    let response = match request.method.as_str() {
        "initialize" => {
            match handle_initialize(request.params, state).await {
                Ok(result) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(result),
                    error: None,
                },
                Err(error) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(error),
                },
            }
        }
        "prompts/list" => {
            match handle_prompts_list(request.params, state).await {
                Ok(result) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(result),
                    error: None,
                },
                Err(error) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(error),
                },
            }
        }
        // 他のメソッドも同様に実装
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        },
    };

    Ok((headers, Json(response)))
}
```

## サーバー起動とポート管理

### 1. 利用可能ポート検索

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

### 2. サーバー起動

```rust
pub async fn start_mcp_server(
    state: McpServerState,
    port: Option<u16>,
) -> Result<u16, Box<dyn std::error::Error>> {
    let server_port = match port {
        Some(p) => p,
        None => find_available_port().await?,
    };

    let app = Router::new()
        .route("/", post(handle_jsonrpc))
        .with_state(Arc::new(state));

    let bind_addr = format!("127.0.0.1:{}", server_port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    println!("MCP Server starting on http://{}", bind_addr);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("MCP Server error: {}", e);
        }
    });

    Ok(server_port)
}
```

## エラーハンドリングパターン

### 1. カスタムエラー型

```rust
#[derive(Debug, thiserror::Error)]
pub enum McpServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Agent library parse error: {0}")]
    AgentLibraryParse(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl From<McpServerError> for JsonRpcError {
    fn from(error: McpServerError) -> Self {
        match error {
            McpServerError::InvalidRequest(msg) => JsonRpcError {
                code: -32602,
                message: msg,
                data: None,
            },
            _ => JsonRpcError {
                code: -32603,
                message: error.to_string(),
                data: None,
            },
        }
    }
}
```

### 2. 結果型の使用

```rust
type McpResult<T> = Result<T, McpServerError>;

async fn safe_handle_request(
    request: JsonRpcRequest,
    state: Arc<McpServerState>,
) -> McpResult<serde_json::Value> {
    match request.method.as_str() {
        "initialize" => handle_initialize(request.params, state).await,
        "prompts/list" => handle_prompts_list(request.params, state).await,
        "prompts/get" => handle_prompts_get(request.params, state).await,
        _ => Err(McpServerError::InvalidRequest(
            format!("Unknown method: {}", request.method)
        )),
    }
}
```

## テストとデバッグ

### 1. 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize() {
        let state = Arc::new(McpServerState::new());
        let result = handle_initialize(None, state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prompts_list_empty() {
        let state = Arc::new(McpServerState::new());
        let result = handle_prompts_list(None, state).await;
        assert!(result.is_ok());
    }
}
```

### 2. 統合テスト

```rust
#[tokio::test]
async fn test_full_mcp_server() {
    let state = McpServerState::new();
    let port = start_mcp_server(state, Some(9999)).await.unwrap();

    // HTTPクライアントでテスト
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://127.0.0.1:{}", port))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
}
```

## ベストプラクティス

1. **非同期処理**: tokioを使った適切な非同期処理
2. **エラーハンドリング**: thiserrorを使った型安全なエラー処理
3. **状態管理**: Arc<RwLock<T>>で安全な並行アクセス
4. **JSON-RPC準拠**: 仕様に厳密に従った実装
5. **ログ**: tracing crateでの構造化ログ
6. **テスト**: 単体テストと統合テストの両方
7. **ドキュメント**: パブリックAPIの文書化
