use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    routing::post,
    Json, Router,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::HashMap;
// use once_cell::sync::Lazy; // 現在未使用

use super::types::{JsonRpcRequest, JsonRpcResponse, JsonRpcError, McpPrompt, McpMessage, McpContent, McpResource};
use crate::agent_library::AgentLibrary;

// レスポンスキャッシュの実装（シンプルなHashMapベース）
#[derive(Clone)]
struct CachedResponse {
    data: serde_json::Value,
    #[allow(dead_code)]
    created_at: Instant,
}

type ResponseCacheEntry = (CachedResponse, Instant);
static RESPONSE_CACHE: std::sync::LazyLock<Mutex<HashMap<String, ResponseCacheEntry>>> = std::sync::LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

#[derive(Clone)]
pub struct McpServerState {
    pub agent_libraries: Arc<RwLock<Vec<AgentLibrary>>>,
}

impl Default for McpServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServerState {
    #[must_use] pub fn new() -> Self {
        Self {
            agent_libraries: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub fn create_mcp_router() -> Router<McpServerState> {
    Router::new()
        .route("/", post(handle_jsonrpc))
        .route("/rpc", post(handle_jsonrpc))
}

#[allow(dead_code)]
async fn handle_cors() -> (StatusCode, HeaderMap) {
    let mut headers = HeaderMap::new();
    
    // セキュリティ: localhost限定のCORS設定
    headers.insert("Access-Control-Allow-Origin", "http://localhost:1420".parse()
        .unwrap_or_else(|_| "null".parse().unwrap()));
    headers.insert("Access-Control-Allow-Methods", "POST, OPTIONS".parse()
        .unwrap_or_else(|_| "POST".parse().unwrap()));
    headers.insert("Access-Control-Allow-Headers", "Content-Type".parse()
        .unwrap_or_else(|_| "Content-Type".parse().unwrap()));
    
    (StatusCode::OK, headers)
}

async fn handle_jsonrpc(
    State(state): State<McpServerState>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<(HeaderMap, Json<JsonRpcResponse>), StatusCode> {
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(request.id, request.params).await,
        "initialized" => handle_initialized(request.id).await,
        "prompts/list" => handle_prompts_list(state, request.id, request.params).await,
        "prompts/get" => handle_prompts_get(state, request.id, request.params).await,
        "resources/list" => handle_resources_list(state, request.id, request.params).await,
        "resources/read" => handle_resources_read(state, request.id, request.params).await,
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

    let mut headers = HeaderMap::new();
    // セキュリティ: localhost限定のCORS設定
    headers.insert("Access-Control-Allow-Origin", "http://localhost:1420".parse()
        .unwrap_or_else(|_| "null".parse().unwrap()));
    headers.insert("Content-Type", "application/json".parse()
        .unwrap_or_else(|_| "text/plain".parse().unwrap()));

    Ok((headers, Json(response)))
}

async fn handle_initialize(id: Option<serde_json::Value>, _params: Option<serde_json::Value>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {
                "prompts": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "librarian",
                "version": "0.1.0"
            }
        })),
        error: None,
    }
}

async fn handle_initialized(id: Option<serde_json::Value>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({})),
        error: None,
    }
}

async fn handle_prompts_list(
    state: McpServerState,
    id: Option<serde_json::Value>,
    _params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    // キャッシュキーを生成（状態のハッシュベース）
    let cache_key = "prompts_list".to_string();
    let cache_ttl = Duration::from_secs(30);
    
    // キャッシュから確認
    if let Ok(cache) = RESPONSE_CACHE.lock() {
        if let Some((cached_response, cached_time)) = cache.get(&cache_key) {
            if cached_time.elapsed() < cache_ttl {
                tracing::debug!("Serving prompts list from cache");
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(cached_response.data.clone()),
                    error: None,
                };
            }
        }
    }

    // キャッシュミス時は実際の処理を実行
    tracing::debug!("Cache miss, generating prompts list");
    let libraries = state.agent_libraries.read().await;
    let mut prompts = Vec::new();

    for library in libraries.iter() {
        for prompt in &library.prompts {
            prompts.push(McpPrompt {
                name: prompt.id.clone(),
                title: Some(prompt.title.clone()),
                description: Some(prompt.description.clone()),
                arguments: vec![],
            });
        }
    }

    let result_data = serde_json::json!({ "prompts": prompts });
    
    // キャッシュに保存
    if let Ok(mut cache) = RESPONSE_CACHE.lock() {
        cache.insert(
            cache_key,
            (CachedResponse {
                data: result_data.clone(),
                created_at: Instant::now(),
            }, Instant::now())
        );
    }
    
    tracing::debug!("Cached prompts list response");

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result_data),
        error: None,
    }
}

async fn handle_prompts_get(
    state: McpServerState,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let name = match params.as_ref().and_then(|p| p.get("name")).and_then(|n| n.as_str()) {
        Some(name) => name.to_string(),
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Invalid params: name required".to_string(),
                    data: None,
                }),
            }
        }
    };

    let libraries = state.agent_libraries.read().await;
    for library in libraries.iter() {
        if let Some(prompt) = library.prompts.iter().find(|p| p.id == name) {
            let message = McpMessage {
                role: "user".to_string(),
                content: McpContent {
                    content_type: "text".to_string(),
                    text: prompt.content.clone(),
                },
            };

            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::json!({ "messages": [message] })),
                error: None,
            };
        }
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code: -32602,
            message: format!("Prompt '{name}' not found"),
            data: None,
        }),
    }
}

async fn handle_resources_list(
    state: McpServerState,
    id: Option<serde_json::Value>,
    _params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let libraries = state.agent_libraries.read().await;
    let mut resources = Vec::new();

    for library in libraries.iter() {
        for prompt in &library.prompts {
            resources.push(McpResource {
                uri: format!("agent_library://{}", prompt.id),
                name: prompt.id.clone(),
                title: Some(prompt.title.clone()),
                description: Some(prompt.description.clone()),
                mime_type: Some("text/markdown".to_string()),
            });
        }
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({ "resources": resources })),
        error: None,
    }
}

async fn handle_resources_read(
    state: McpServerState,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let uri = match params.as_ref().and_then(|p| p.get("uri")).and_then(|u| u.as_str()) {
        Some(uri) => uri.to_string(),
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Invalid params: uri required".to_string(),
                    data: None,
                }),
            }
        }
    };

    if let Some(prompt_id) = uri.strip_prefix("agent_library://") {
        let libraries = state.agent_libraries.read().await;
        for library in libraries.iter() {
            if let Some(prompt) = library.prompts.iter().find(|p| p.id == prompt_id) {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({
                        "contents": [{
                            "uri": &uri,
                            "mimeType": "text/markdown",
                            "text": prompt.content
                        }]
                    })),
                    error: None,
                };
            }
        }
    }

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code: -32602,
            message: format!("Resource '{}' not found", &uri),
            data: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_library::{AgentLibrary, AgentIndex, Prompt};
    use std::path::PathBuf;

    fn create_test_agent_library() -> AgentLibrary {
        let index = AgentIndex {
            mcp_endpoints: vec![],
        };

        let prompt = Prompt {
            id: "test_prompt".to_string(),
            title: "Test Prompt".to_string(),
            description: "Test Description".to_string(),
            content: "Test prompt content".to_string(),
            file_path: PathBuf::from("/test/prompt.md"),
        };

        AgentLibrary {
            index,
            base_path: PathBuf::from("/test/.agent_library"),
            prompts: vec![prompt],
        }
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let response = handle_initialize(Some(serde_json::Value::from(1)), None).await;
        
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(serde_json::Value::from(1)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        if let Some(result) = response.result {
            assert_eq!(result["protocolVersion"], "2025-06-18");
            assert!(result["capabilities"]["prompts"].is_object());
            assert!(result["capabilities"]["resources"].is_object());
        }
    }

    #[tokio::test]
    async fn test_handle_initialized() {
        let response = handle_initialized(Some(serde_json::Value::from(1))).await;
        
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(serde_json::Value::from(1)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_prompts_list() {
        let state = McpServerState::new();
        {
            let mut libraries = state.agent_libraries.write().await;
            libraries.push(create_test_agent_library());
        }

        let response = handle_prompts_list(state, Some(serde_json::Value::from(1)), None).await;
        
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(serde_json::Value::from(1)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        if let Some(result) = response.result {
            let prompts = &result["prompts"];
            assert!(prompts.is_array());
            assert_eq!(prompts.as_array().unwrap().len(), 1);
            
            let prompt = &prompts[0];
            assert_eq!(prompt["name"], "test_prompt");
            assert_eq!(prompt["title"], "Test Prompt");
        }
    }

    #[tokio::test]
    async fn test_handle_prompts_get_valid() {
        let state = McpServerState::new();
        {
            let mut libraries = state.agent_libraries.write().await;
            libraries.push(create_test_agent_library());
        }

        let params = serde_json::json!({ "name": "test_prompt" });
        let response = handle_prompts_get(state, Some(serde_json::Value::from(1)), Some(params)).await;
        
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(serde_json::Value::from(1)));
        assert!(response.result.is_some());
        assert!(response.error.is_none());

        if let Some(result) = response.result {
            let messages = &result["messages"];
            assert!(messages.is_array());
            assert_eq!(messages.as_array().unwrap().len(), 1);
            
            let message = &messages[0];
            assert_eq!(message["role"], "user");
            assert_eq!(message["content"]["text"], "Test prompt content");
        }
    }

    #[tokio::test]
    async fn test_handle_prompts_get_not_found() {
        let state = McpServerState::new();
        {
            let mut libraries = state.agent_libraries.write().await;
            libraries.push(create_test_agent_library());
        }

        let params = serde_json::json!({ "name": "nonexistent_prompt" });
        let response = handle_prompts_get(state, Some(serde_json::Value::from(1)), Some(params)).await;
        
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(serde_json::Value::from(1)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        if let Some(error) = response.error {
            assert_eq!(error.code, -32602);
            assert!(error.message.contains("not found"));
        }
    }

    #[tokio::test]
    async fn test_handle_prompts_get_missing_params() {
        let state = McpServerState::new();
        let response = handle_prompts_get(state, Some(serde_json::Value::from(1)), None).await;
        
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(serde_json::Value::from(1)));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        if let Some(error) = response.error {
            assert_eq!(error.code, -32602);
            assert_eq!(error.message, "Invalid params: name required");
        }
    }

    #[tokio::test]
    async fn test_response_cache() {
        let state = McpServerState::new();
        {
            let mut libraries = state.agent_libraries.write().await;
            libraries.push(create_test_agent_library());
        }

        // First request - should cache
        let response1 = handle_prompts_list(state.clone(), Some(serde_json::Value::from(1)), None).await;
        assert!(response1.result.is_some());

        // Second request - should use cache
        let response2 = handle_prompts_list(state, Some(serde_json::Value::from(2)), None).await;
        assert!(response2.result.is_some());

        // Results should be identical (content-wise)
        assert_eq!(response1.result, response2.result);
    }
}