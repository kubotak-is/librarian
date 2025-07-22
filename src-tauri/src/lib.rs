pub mod agent_library;
pub mod mcp;
pub mod config;
pub mod persistence;
pub mod file_watcher;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn, error, debug};

// Global MCP server state
lazy_static::lazy_static! {
    static ref MCP_SERVER_STATE: Arc<Mutex<Option<mcp::McpServerState>>> = Arc::new(Mutex::new(None));
    static ref MCP_SERVERS: Arc<Mutex<HashMap<String, McpServerInstance>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Individual MCP server instance
struct McpServerInstance {
    repository_id: String,
    port: u16,
    state: mcp::McpServerState,
    _handle: tokio::task::JoinHandle<()>,
}

impl McpServerInstance {
    pub const fn new(repository_id: String, port: u16, state: mcp::McpServerState, handle: tokio::task::JoinHandle<()>) -> Self {
        Self {
            repository_id,
            port,
            state,
            _handle: handle,
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[tauri::command]
async fn select_directory() -> Result<Option<String>, String> {
    // Directory selection is handled by the frontend using tauri-plugin-dialog
    // This command is kept for API compatibility but not used
    Ok(None)
}

// Agent library commands
#[tauri::command]
async fn parse_agent_library(repo_path: String) -> Result<agent_library::AgentLibrary, String> {
    // セキュリティ: パス検証
    validate_path_security(&repo_path)?;
    
    let path = std::path::Path::new(&repo_path);
    agent_library::AgentLibraryParser::parse(path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_agent_library(path: String) -> Result<bool, String> {
    // セキュリティ: パス検証
    validate_path_security(&path)?;
    
    let repo_path = std::path::Path::new(&path);
    let agent_lib_path = repo_path.join(".agent_library");
    
    Ok(agent_lib_path.exists() && agent_lib_path.is_dir())
}

#[tauri::command]
async fn find_repositories(search_paths: Vec<String>) -> Result<Vec<String>, String> {
    // セキュリティ: 各パスを検証
    for path in &search_paths {
        validate_path_security(path)?;
    }
    
    let paths: Vec<std::path::PathBuf> = search_paths.into_iter()
        .map(std::path::PathBuf::from)
        .collect();
    
    let repos = agent_library::AgentLibraryParser::find_repositories(&paths)
        .map_err(|e| e.to_string())?;
    
    Ok(repos.into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

// MCP Server commands
#[tauri::command]
async fn start_mcp_server(port: Option<u16>) -> Result<String, String> {
    let server_port = port.unwrap_or(9500);
    
    // セキュリティ: ポート検証
    validate_port_security(server_port)?;
    
    // Create MCP server state
    let state = mcp::McpServerState::new();
    
    // Store the state globally for later access
    {
        let mut global_state = MCP_SERVER_STATE.lock()
            .map_err(|e| format!("Failed to acquire lock: {e}"))?;
        *global_state = Some(state.clone());
    }
    
    // Create router
    let app = mcp::create_mcp_router().with_state(state);
    
    // Try to bind to the address first to verify it's available
    let bind_addr = format!("127.0.0.1:{server_port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| format!("Failed to bind to {bind_addr}: {e}"))?;
    
    println!("MCP Server starting on http://{bind_addr}");
    
    // Start server in background
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("MCP Server error: {e}");
        }
    });
    
    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(format!("MCP Server started on port {server_port}"))
}

#[tauri::command]
async fn load_agent_library_to_mcp(repo_path: String) -> Result<String, String> {
    let path = std::path::Path::new(&repo_path);
    let library = agent_library::AgentLibraryParser::parse(path)
        .map_err(|e| format!("Failed to parse agent library: {e}"))?;
    
    // Get a copy of the state reference before using async
    let state_clone = {
        let state_guard = MCP_SERVER_STATE.lock()
            .map_err(|e| format!("Failed to acquire lock: {e}"))?;
        state_guard.clone()
    };
    
    // Load into the running MCP server state
    if let Some(state) = state_clone {
        let mut libraries = state.agent_libraries.write().await;
        libraries.clear();
        libraries.push(library.clone());
        
        let prompt_count = library.prompts.len();
        let endpoint_count = library.index.mcp_endpoints.len();
        
        Ok(format!(
            "✅ Loaded {prompt_count} prompts and {endpoint_count} endpoints from {repo_path}"
        ))
    } else {
        Err("MCP Server is not running. Please start the server first.".to_string())
    }
}

// Repository-specific MCP server commands
#[tauri::command]
async fn start_repository_mcp_server(repository_id: String, repo_path: String, port: Option<u16>) -> Result<String, String> {
    info!(repository_id = %repository_id, repo_path = %repo_path, "Starting MCP server for repository");
    
    // セキュリティ: パス検証
    validate_path_security(&repo_path)?;
    
    // Find available port starting from 9500
    let server_port = if let Some(p) = port {
        debug!(port = p, "Using provided port");
        validate_port_security(p)?;
        p
    } else {
        debug!("Finding available port");
        find_available_port().await?
    };
    
    // Parse agent library for this repository
    let path = std::path::Path::new(&repo_path);
    let library = agent_library::AgentLibraryParser::parse(path)
        .map_err(|e| {
            error!(repository_id = %repository_id, repo_path = %repo_path, error = %e, "Failed to parse agent library");
            format!("Failed to parse agent library: {e}")
        })?;
    
    info!(repository_id = %repository_id, prompts_count = library.prompts.len(), "Agent library parsed successfully");
    
    // Create MCP server state for this repository
    let state = mcp::McpServerState::new();
    {
        let mut libraries = state.agent_libraries.write().await;
        libraries.push(library);
    }
    
    // Create router
    let app = mcp::create_mcp_router().with_state(state.clone());
    
    // Try to bind to the address
    let bind_addr = format!("127.0.0.1:{server_port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| {
            error!(repository_id = %repository_id, bind_addr = %bind_addr, error = %e, "Failed to bind to address");
            format!("Failed to bind to {bind_addr}: {e}")
        })?;
    
    info!(repository_id = %repository_id, bind_addr = %bind_addr, "MCP Server starting");
    
    // Clone repository_id for use in async block
    let repo_id_for_spawn = repository_id.clone();
    
    // Start server in background
    let handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!(repository_id = %repo_id_for_spawn, error = %e, "MCP Server error");
        }
    });
    
    // Store server instance
    {
        let mut servers = MCP_SERVERS.lock().unwrap();
        let instance = McpServerInstance::new(repository_id.clone(), server_port, state, handle);
        servers.insert(repository_id.clone(), instance);
    }
    
    info!(repository_id = %repository_id, port = server_port, "MCP Server started successfully");
    Ok(format!("MCP Server for repository '{repository_id}' started on port {server_port}"))
}

#[tauri::command]
async fn stop_repository_mcp_server(repository_id: String) -> Result<String, String> {
    let mut servers = MCP_SERVERS.lock().unwrap();
    
    if let Some(instance) = servers.remove(&repository_id) {
        instance._handle.abort();
        Ok(format!("MCP Server for repository '{repository_id}' stopped"))
    } else {
        Err(format!("No MCP server found for repository '{repository_id}'"))
    }
}

#[tauri::command]
async fn get_mcp_server_status(repository_id: String) -> Result<serde_json::Value, String> {
    let servers = MCP_SERVERS.lock().unwrap();
    
    if let Some(instance) = servers.get(&repository_id) {
        Ok(serde_json::json!({
            "repository_id": instance.repository_id,
            "port": instance.port,
            "status": "running"
        }))
    } else {
        Ok(serde_json::json!({
            "repository_id": repository_id,
            "status": "stopped"
        }))
    }
}

// Configuration persistence commands
#[tauri::command]
async fn load_app_config(app: tauri::AppHandle) -> Result<persistence::AppConfig, String> {
    persistence::AppConfig::load(&app).await
}

#[tauri::command]
async fn save_app_config(app: tauri::AppHandle, config: persistence::AppConfig) -> Result<(), String> {
    config.save(&app).await
}

#[tauri::command]
async fn add_repository_config(
    app: tauri::AppHandle,
    repository: persistence::RepositoryConfig,
) -> Result<(), String> {
    let mut config = persistence::AppConfig::load(&app).await?;
    config.add_repository(repository);
    config.save(&app).await
}

#[tauri::command]
async fn remove_repository_config(
    app: tauri::AppHandle,
    repository_id: String,
) -> Result<bool, String> {
    let mut config = persistence::AppConfig::load(&app).await?;
    let removed = config.remove_repository(&repository_id);
    if removed {
        config.save(&app).await?;
    }
    Ok(removed)
}

#[tauri::command]
async fn update_repository_mcp_status(
    app: tauri::AppHandle,
    repository_id: String,
    port: u16,
    status: String,
) -> Result<(), String> {
    let mut config = persistence::AppConfig::load(&app).await?;
    let updated = config.update_repository(&repository_id, |repo| {
        repo.mcp_server = Some(persistence::McpServerConfig { port, status });
        repo.update_last_updated();
    });
    
    if updated {
        config.save(&app).await?;
        Ok(())
    } else {
        Err(format!("Repository '{repository_id}' not found"))
    }
}

// File watching commands
#[tauri::command]
async fn start_watching_repository(
    repository_id: String,
    repository_path: String,
) -> Result<(), String> {
    // セキュリティ: パス検証
    validate_path_security(&repository_path)?;
    
    if let Some(manager) = file_watcher::get_file_watcher_manager().await {
        let path = std::path::PathBuf::from(repository_path);
        manager.watch_repository(repository_id, path).await
    } else {
        Err("File watcher manager not initialized".to_string())
    }
}

#[tauri::command]
async fn stop_watching_repository(repository_id: String) -> Result<(), String> {
    if let Some(manager) = file_watcher::get_file_watcher_manager().await {
        manager.stop_watching(&repository_id).await;
        Ok(())
    } else {
        Err("File watcher manager not initialized".to_string())
    }
}

#[tauri::command]
async fn get_watched_repositories() -> Result<Vec<String>, String> {
    if let Some(manager) = file_watcher::get_file_watcher_manager().await {
        Ok(manager.get_watched_repositories().await)
    } else {
        Err("File watcher manager not initialized".to_string())
    }
}

#[tauri::command]
async fn save_prompt_file(repo_path: String, prompt_id: String, content: String) -> Result<(), String> {
    use std::io::Write;
    
    // セキュリティ: パス検証とコンテンツサイズ制限
    validate_path_security(&repo_path)?;
    
    // コンテンツサイズ制限（1MB）
    if content.len() > 1024 * 1024 {
        return Err("Content size exceeds maximum limit (1MB)".to_string());
    }
    
    info!(repo_path = %repo_path, prompt_id = %prompt_id, content_length = content.len(), "Saving prompt file");
    
    // Parse the agent library to find the prompt file
    let path = std::path::Path::new(&repo_path);
    let library = agent_library::AgentLibraryParser::parse(path)
        .map_err(|e| {
            error!(repo_path = %repo_path, error = %e, "Failed to parse agent library for prompt save");
            format!("Failed to parse agent library: {e}")
        })?;
    
    // Find the prompt by ID
    let prompt = library.prompts.iter()
        .find(|p| p.id == prompt_id)
        .ok_or_else(|| {
            warn!(prompt_id = %prompt_id, available_prompts = ?library.prompts.iter().map(|p| &p.id).collect::<Vec<_>>(), "Prompt not found");
            format!("Prompt with ID '{prompt_id}' not found")
        })?;
    
    // Get the prompt file path (prompt.file_path is already the full path)
    let prompt_file_path = &prompt.file_path;
    debug!(prompt_file_path = %prompt_file_path.display(), "Writing to prompt file");
    
    // Write content to file
    let mut file = std::fs::File::create(prompt_file_path)
        .map_err(|e| {
            error!(prompt_file_path = %prompt_file_path.display(), error = %e, "Failed to create prompt file");
            format!("Failed to create file {}: {}", prompt_file_path.display(), e)
        })?;
    
    file.write_all(content.as_bytes())
        .map_err(|e| {
            error!(prompt_file_path = %prompt_file_path.display(), error = %e, "Failed to write to prompt file");
            format!("Failed to write to file {}: {}", prompt_file_path.display(), e)
        })?;
    
    info!(prompt_id = %prompt_id, prompt_file_path = %prompt_file_path.display(), "Prompt file saved successfully");
    
    Ok(())
}

#[tauri::command]
async fn reload_agent_library(repository_id: String, repository_path: String) -> Result<String, String> {
    info!(repository_id = %repository_id, repository_path = %repository_path, "Reloading agent library");
    
    // セキュリティ: パス検証
    validate_path_security(&repository_path)?;
    
    // キャッシュを無効化
    let _agent_lib_path = std::path::Path::new(&repository_path).join(".agent_library");
    // Note: agent_library::parser::AGENT_LIBRARY_CACHE.invalidate(&agent_lib_path);
    // TODO: キャッシュ無効化APIを公開する必要がある
    
    // agent_library を再読み込みしてMCPサーバーの状態を更新
    let path = std::path::Path::new(&repository_path);
    let library = agent_library::AgentLibraryParser::parse(path)
        .map_err(|e| {
            error!(repository_id = %repository_id, repository_path = %repository_path, error = %e, "Failed to reload agent library");
            format!("Failed to reload agent library: {e}")
        })?;
    
    // 実行中のMCPサーバーがあれば更新
    let state_option = {
        let servers = MCP_SERVERS.lock().unwrap();
        servers.get(&repository_id).map(|instance| instance.state.clone())
    }; // MutexGuardをここでdrop
    
    if let Some(state) = state_option {
        // MCPサーバーの状態を更新
        let mut libraries = state.agent_libraries.write().await;
        libraries.clear();
        libraries.push(library.clone());
        
        Ok(format!(
            "Reloaded {} prompts and {} endpoints for repository '{}'",
            library.prompts.len(),
            library.index.mcp_endpoints.len(),
            repository_id
        ))
    } else {
        Ok(format!(
            "Agent library reloaded ({} prompts, {} endpoints), but no MCP server is running",
            library.prompts.len(),
            library.index.mcp_endpoints.len()
        ))
    }
}

/// セキュリティ：パスの検証を行う
fn validate_path_security(path: &str) -> Result<(), String> {
    let path_obj = Path::new(path);
    
    // パストラバーサル攻撃の防止
    if path.contains("..") {
        return Err("Path traversal detected".to_string());
    }
    
    // 絶対パスのみ許可
    if !path_obj.is_absolute() {
        return Err("Only absolute paths are allowed".to_string());
    }
    
    // 危険なパスの除外
    let dangerous_paths = ["/etc", "/usr", "/bin", "/sbin", "/var", "/boot", "/dev", "/proc", "/sys"];
    if dangerous_paths.iter().any(|&dangerous| path.starts_with(dangerous)) {
        return Err("Access to system directories is not allowed".to_string());
    }
    
    Ok(())
}

/// セキュリティ：MCPサーバーポートの範囲制限
fn validate_port_security(port: u16) -> Result<(), String> {
    if !(9500..=9599).contains(&port) {
        return Err("Port must be in range 9500-9599".to_string());
    }
    Ok(())
}

async fn find_available_port() -> Result<u16, String> {
    for port in 9500..9600 {
        if tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await.is_ok() {
            return Ok(port);
        }
    }
    Err("No available ports in range 9500-9599".to_string())
}

fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "librarian_app_lib=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_security_valid_paths() {
        // 有効な絶対パス
        assert!(validate_path_security("/Users/test/workspace").is_ok());
        assert!(validate_path_security("/home/user/documents").is_ok());
        assert!(validate_path_security("/tmp/test").is_ok());
    }

    #[test]
    fn test_validate_path_security_path_traversal() {
        // パストラバーサル攻撃の検出
        assert!(validate_path_security("/Users/test/../etc/passwd").is_err());
        assert!(validate_path_security("../etc/passwd").is_err());
        assert!(validate_path_security("/home/user/../../etc").is_err());
    }

    #[test]
    fn test_validate_path_security_relative_paths() {
        // 相対パスの拒否
        assert!(validate_path_security("relative/path").is_err());
        assert!(validate_path_security("./current/dir").is_err());
        assert!(validate_path_security("test/file.txt").is_err());
    }

    #[test]
    fn test_validate_path_security_dangerous_paths() {
        // 危険なシステムディレクトリの拒否
        assert!(validate_path_security("/etc/passwd").is_err());
        assert!(validate_path_security("/usr/bin/bash").is_err());
        assert!(validate_path_security("/var/log/system.log").is_err());
        assert!(validate_path_security("/bin/sh").is_err());
        assert!(validate_path_security("/sbin/init").is_err());
        assert!(validate_path_security("/boot/vmlinuz").is_err());
        assert!(validate_path_security("/dev/null").is_err());
        assert!(validate_path_security("/proc/version").is_err());
        assert!(validate_path_security("/sys/kernel").is_err());
    }

    #[test]
    fn test_validate_port_security_valid_ports() {
        // 有効なポート範囲
        assert!(validate_port_security(9500).is_ok());
        assert!(validate_port_security(9550).is_ok());
        assert!(validate_port_security(9599).is_ok());
    }

    #[test]
    fn test_validate_port_security_invalid_ports() {
        // 無効なポート範囲
        assert!(validate_port_security(9499).is_err());
        assert!(validate_port_security(9600).is_err());
        assert!(validate_port_security(3000).is_err());
        assert!(validate_port_security(80).is_err());
        assert!(validate_port_security(443).is_err());
        assert!(validate_port_security(22).is_err());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 構造化ログを初期化
    init_tracing();
    info!("Starting Librarian application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            info!("Setting up Tauri application");
            // ファイル監視マネージャーを初期化
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                info!("Initializing file watcher");
                file_watcher::initialize_file_watcher(app_handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            select_directory,
            parse_agent_library,
            validate_agent_library,
            find_repositories,
            start_mcp_server,
            load_agent_library_to_mcp,
            start_repository_mcp_server,
            stop_repository_mcp_server,
            get_mcp_server_status,
            load_app_config,
            save_app_config,
            add_repository_config,
            remove_repository_config,
            update_repository_mcp_status,
            start_watching_repository,
            stop_watching_repository,
            get_watched_repositories,
            reload_agent_library,
            save_prompt_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
