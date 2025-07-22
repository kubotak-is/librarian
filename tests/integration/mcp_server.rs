use axum::http::StatusCode;
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};
use tower::util::ServiceExt;

use crate::agent_library::{AgentLibraryParser, AgentIndex};
use crate::mcp::{create_mcp_router, McpServerState};
use crate::mcp::types::{JsonRpcRequest, JsonRpcResponse};

#[tokio::test]
async fn test_mcp_server_full_workflow() {
    // Setup test agent library
    let temp_dir = TempDir::new().unwrap();
    create_test_agent_library(temp_dir.path()).unwrap();
    
    // Parse agent library
    let library = AgentLibraryParser::parse(temp_dir.path()).unwrap();
    
    // Create MCP server state with test data
    let state = McpServerState::new();
    {
        let mut libraries = state.agent_libraries.write().await;
        libraries.push(library);
    }
    
    // Create router
    let app = create_mcp_router().with_state(state);
    
    // Test 1: Initialize
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
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let init_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(init_response.error.is_none());
    assert!(init_response.result.is_some());
    
    // Test 2: Prompts List
    let prompts_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "prompts/list".to_string(),
        params: None,
    };
    
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(serde_json::to_string(&prompts_request).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let prompts_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(prompts_response.error.is_none());
    let result = prompts_response.result.unwrap();
    assert!(result["prompts"].is_array());
    assert_eq!(result["prompts"].as_array().unwrap().len(), 1);
    
    // Test 3: Get specific prompt
    let get_prompt_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "prompts/get".to_string(),
        params: Some(json!({
            "name": "test_prompt"
        })),
    };
    
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(serde_json::to_string(&get_prompt_request).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let get_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(get_response.error.is_none());
    let result = get_response.result.unwrap();
    assert!(result["messages"].is_array());
    assert_eq!(result["messages"].as_array().unwrap().len(), 1);
    
    let message = &result["messages"][0];
    assert_eq!(message["role"], "user");
    assert!(message["content"]["text"].as_str().unwrap().contains("Test Prompt"));
    
    // Test 4: Resources operations
    let resources_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "resources/list".to_string(),
        params: None,
    };
    
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(serde_json::to_string(&resources_request).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let resources_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(resources_response.error.is_none());
    let result = resources_response.result.unwrap();
    assert!(result["resources"].is_array());
    assert_eq!(result["resources"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_mcp_server_error_handling() {
    let state = McpServerState::new();
    let app = create_mcp_router().with_state(state);
    
    // Test invalid method
    let invalid_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "invalid/method".to_string(),
        params: None,
    };
    
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(serde_json::to_string(&invalid_request).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let error_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(error_response.result.is_none());
    assert!(error_response.error.is_some());
    
    let error = error_response.error.unwrap();
    assert_eq!(error.code, -32601);
    assert_eq!(error.message, "Method not found");
}

#[tokio::test]
async fn test_mcp_server_caching() {
    // Setup test data
    let temp_dir = TempDir::new().unwrap();
    create_test_agent_library(temp_dir.path()).unwrap();
    let library = AgentLibraryParser::parse(temp_dir.path()).unwrap();
    
    let state = McpServerState::new();
    {
        let mut libraries = state.agent_libraries.write().await;
        libraries.push(library);
    }
    
    let app = create_mcp_router().with_state(state);
    
    // Make first request
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "prompts/list".to_string(),
        params: None,
    };
    
    let start_time = std::time::Instant::now();
    
    let response1 = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(serde_json::to_string(&request).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();
    
    let first_duration = start_time.elapsed();
    
    // Make second request (should use cache)
    let start_time = std::time::Instant::now();
    
    let response2 = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(serde_json::to_string(&request).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();
    
    let second_duration = start_time.elapsed();
    
    // Both responses should be successful
    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response2.status(), StatusCode::OK);
    
    // Second request should be faster (cached)
    assert!(second_duration < first_duration);
    
    // Response content should be identical
    let body1 = hyper::body::to_bytes(response1.into_body()).await.unwrap();
    let body2 = hyper::body::to_bytes(response2.into_body()).await.unwrap();
    
    let response1: JsonRpcResponse = serde_json::from_slice(&body1).unwrap();
    let response2: JsonRpcResponse = serde_json::from_slice(&body2).unwrap();
    
    assert_eq!(response1.result, response2.result);
}

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

    let prompt_content = "# Test Prompt\n\nThis is a test prompt for integration testing.";
    fs::write(agent_lib_dir.join("test_prompt.md"), prompt_content)?;

    Ok(())
}