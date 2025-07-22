# Tauri コマンド開発ガイド

このプロンプトは、LibrarianアプリケーションでのTauriコマンド実装に特化したガイドです。

## Tauri コマンドアーキテクチャ

### 基本構造

```
Svelte Frontend ← invoke() → Tauri Commands ← Rust Logic → System APIs
```

### フロントエンド側の実装

```typescript
import { invoke } from '@tauri-apps/api/core';

// 型安全なコマンド呼び出し
async function callTauriCommand<T>(command: string, params?: Record<string, any>): Promise<T> {
  try {
    return await invoke(command, params);
  } catch (error) {
    console.error(`Command ${command} failed:`, error);
    throw error;
  }
}
```

## 主要コマンドの実装パターン

### 1. Agent Library パース

```rust
#[tauri::command]
async fn parse_agent_library(repo_path: String) -> Result<AgentLibrary, String> {
    let agent_index_path = Path::new(&repo_path).join(".agent_library/agent_index.yml");

    if !agent_index_path.exists() {
        return Err("agent_index.yml not found".to_string());
    }

    let content = tokio::fs::read_to_string(&agent_index_path)
        .await
        .map_err(|e| format!("Failed to read agent_index.yml: {}", e))?;

    let agent_index: AgentIndex = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    // プロンプトファイルを読み込み
    let mut prompts = Vec::new();
    for endpoint in agent_index.mcp_endpoints {
        let prompt_path = Path::new(&repo_path)
            .join(".agent_library")
            .join(&endpoint.prompt_file);

        if prompt_path.exists() {
            let content = tokio::fs::read_to_string(&prompt_path)
                .await
                .unwrap_or_default();

            prompts.push(Prompt {
                id: endpoint.id,
                title: endpoint.label,
                description: endpoint.description,
                category: endpoint.category,
                content,
            });
        }
    }

    Ok(AgentLibrary {
        name: agent_index.name,
        description: agent_index.description,
        prompts,
    })
}
```

### 2. MCPサーバー管理

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type ServerManager = Arc<RwLock<HashMap<String, McpServerHandle>>>;

#[derive(Clone)]
struct McpServerHandle {
    port: u16,
    handle: Arc<tokio::task::JoinHandle<()>>,
    repository_path: String,
}

#[tauri::command]
async fn start_repository_mcp_server(
    repository_id: String,
    repo_path: String,
    state: tauri::State<'_, ServerManager>,
) -> Result<String, String> {
    let mut servers = state.write().await;

    // 既に起動している場合はスキップ
    if servers.contains_key(&repository_id) {
        return Ok("Server already running".to_string());
    }

    // 利用可能なポートを見つける
    let port = find_available_port().await?;

    // Agent libraryをパース
    let agent_library = parse_agent_library_internal(repo_path.clone()).await?;

    // MCPサーバーを起動
    let handle = tokio::spawn(async move {
        if let Err(e) = start_mcp_server(port, agent_library).await {
            eprintln!("MCP server error: {}", e);
        }
    });

    servers.insert(repository_id, McpServerHandle {
        port,
        handle: Arc::new(handle),
        repository_path: repo_path,
    });

    Ok(format!("MCP server started on port {}", port))
}

#[tauri::command]
async fn stop_repository_mcp_server(
    repository_id: String,
    state: tauri::State<'_, ServerManager>,
) -> Result<String, String> {
    let mut servers = state.write().await;

    if let Some(server) = servers.remove(&repository_id) {
        server.handle.abort();
        Ok("Server stopped".to_string())
    } else {
        Err("Server not found".to_string())
    }
}

#[tauri::command]
async fn get_server_status(
    state: tauri::State<'_, ServerManager>,
) -> Result<Vec<ServerStatus>, String> {
    let servers = state.read().await;
    let mut status_list = Vec::new();

    for (id, server) in servers.iter() {
        status_list.push(ServerStatus {
            repository_id: id.clone(),
            port: server.port,
            status: if server.handle.is_finished() {
                "stopped".to_string()
            } else {
                "running".to_string()
            },
        });
    }

    Ok(status_list)
}
```

### 3. ファイルシステム操作

```rust
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

#[tauri::command]
async fn select_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let result = app.dialog()
        .file()
        .pick_folder()
        .await
        .map_err(|e| format!("Dialog error: {}", e))?;

    Ok(result.map(|path| path.to_string_lossy().to_string()))
}

#[tauri::command]
async fn validate_repository_path(path: String) -> Result<bool, String> {
    let agent_library_path = Path::new(&path).join(".agent_library");
    let agent_index_path = agent_library_path.join("agent_index.yml");

    Ok(agent_library_path.is_dir() && agent_index_path.is_file())
}

#[tauri::command]
async fn get_repository_info(path: String) -> Result<RepositoryInfo, String> {
    let agent_index_path = Path::new(&path).join(".agent_library/agent_index.yml");

    let content = tokio::fs::read_to_string(&agent_index_path)
        .await
        .map_err(|e| format!("Failed to read agent_index.yml: {}", e))?;

    let agent_index: AgentIndex = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    Ok(RepositoryInfo {
        name: agent_index.name,
        description: agent_index.description,
        prompt_count: agent_index.mcp_endpoints.len(),
        categories: agent_index.mcp_endpoints
            .iter()
            .map(|e| e.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect(),
    })
}
```

## エラーハンドリングパターン

### カスタムエラー型

```rust
#[derive(Debug, thiserror::Error)]
enum LibrarianError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),
}

impl From<LibrarianError> for String {
    fn from(error: LibrarianError) -> Self {
        error.to_string()
    }
}
```

### 結果型の統一

```rust
type LibrarianResult<T> = Result<T, LibrarianError>;

#[tauri::command]
async fn unified_command() -> LibrarianResult<String> {
    // 統一されたエラーハンドリング
    let content = tokio::fs::read_to_string("file.txt").await?;
    let parsed: Value = serde_yaml::from_str(&content)?;
    Ok("Success".to_string())
}
```

## 状態管理

### グローバル状態の設定

```rust
use tauri::Manager;

#[derive(Default)]
struct AppState {
    repositories: Arc<RwLock<Vec<Repository>>>,
    mcp_servers: Arc<RwLock<HashMap<String, McpServerHandle>>>,
    config: Arc<RwLock<AppConfig>>,
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            parse_agent_library,
            start_repository_mcp_server,
            stop_repository_mcp_server,
            select_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 状態の永続化

```rust
#[tauri::command]
async fn save_repositories(
    repositories: Vec<Repository>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let config_dir = app.path().app_config_dir()
        .map_err(|e| format!("Failed to get config dir: {}", e))?;

    tokio::fs::create_dir_all(&config_dir).await
        .map_err(|e| format!("Failed to create config dir: {}", e))?;

    let config_path = config_dir.join("repositories.json");
    let json = serde_json::to_string_pretty(&repositories)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    tokio::fs::write(&config_path, json).await
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn load_repositories(app: tauri::AppHandle) -> Result<Vec<Repository>, String> {
    let config_dir = app.path().app_config_dir()
        .map_err(|e| format!("Failed to get config dir: {}", e))?;

    let config_path = config_dir.join("repositories.json");

    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = tokio::fs::read_to_string(&config_path).await
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let repositories: Vec<Repository> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to deserialize: {}", e))?;

    Ok(repositories)
}
```

## フロントエンド統合

### 型安全なコマンド呼び出し

```typescript
// types.ts
export interface Repository {
  id: string;
  name: string;
  path: string;
  isActive: boolean;
  lastUpdated: Date;
  mcpServer?: {
    port: number;
    status: 'running' | 'stopped' | 'error';
  };
}

export interface AgentLibrary {
  name: string;
  description: string;
  prompts: Prompt[];
}

// api.ts
import { invoke } from '@tauri-apps/api/core';

export class TauriAPI {
  static async parseAgentLibrary(repoPath: string): Promise<AgentLibrary> {
    return await invoke('parse_agent_library', { repoPath });
  }

  static async startMcpServer(repositoryId: string, repoPath: string): Promise<string> {
    return await invoke('start_repository_mcp_server', { repositoryId, repoPath });
  }

  static async stopMcpServer(repositoryId: string): Promise<string> {
    return await invoke('stop_repository_mcp_server', { repositoryId });
  }

  static async selectDirectory(): Promise<string | null> {
    return await invoke('select_directory');
  }
}
```

### エラー境界コンポーネント

```svelte
<script lang="ts">
  import { TauriAPI } from './api';
  import { notification } from './stores';

  async function handleCommand<T>(
    command: () => Promise<T>,
    successMessage?: string
  ): Promise<T | null> {
    try {
      const result = await command();
      if (successMessage) {
        notification.set({
          type: 'success',
          message: successMessage,
        });
      }
      return result;
    } catch (error) {
      notification.set({
        type: 'error',
        message: `エラーが発生しました: ${error}`,
      });
      return null;
    }
  }

  async function addRepository() {
    const path = await handleCommand(
      () => TauriAPI.selectDirectory(),
      'ディレクトリを選択しました'
    );

    if (path) {
      await handleCommand(() => TauriAPI.parseAgentLibrary(path), 'リポジトリを追加しました');
    }
  }
</script>
```

## テストとデバッグ

### ユニットテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_parse_agent_library() {
        let temp_dir = TempDir::new().unwrap();
        let agent_library_dir = temp_dir.path().join(".agent_library");
        std::fs::create_dir_all(&agent_library_dir).unwrap();

        let agent_index_content = r#"
name: "Test Library"
description: "Test description"
mcp_endpoints:
  - id: test_prompt
    label: "Test Prompt"
    description: "Test description"
    prompt_file: "TEST_PROMPT.md"
    category: "test"
"#;

        std::fs::write(
            agent_library_dir.join("agent_index.yml"),
            agent_index_content,
        ).unwrap();

        std::fs::write(
            agent_library_dir.join("TEST_PROMPT.md"),
            "# Test Prompt\nThis is a test prompt.",
        ).unwrap();

        let result = parse_agent_library(temp_dir.path().to_string_lossy().to_string()).await;
        assert!(result.is_ok());

        let library = result.unwrap();
        assert_eq!(library.name, "Test Library");
        assert_eq!(library.prompts.len(), 1);
    }
}
```

### デバッグ用ログ

```rust
use tracing::{info, warn, error, debug};

#[tauri::command]
async fn debug_command(input: String) -> Result<String, String> {
    debug!("Command called with input: {}", input);

    match process_input(&input).await {
        Ok(result) => {
            info!("Command completed successfully");
            Ok(result)
        }
        Err(e) => {
            error!("Command failed: {}", e);
            Err(e.to_string())
        }
    }
}
```

## パフォーマンス最適化

### 非同期処理の活用

```rust
#[tauri::command]
async fn batch_operation(inputs: Vec<String>) -> Result<Vec<String>, String> {
    let futures = inputs.into_iter().map(|input| async move {
        // 各操作を並列実行
        process_single_input(input).await
    });

    let results = futures::future::join_all(futures).await;

    results.into_iter().collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
```

### キャッシュ機能

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

type Cache<K, V> = Arc<RwLock<HashMap<K, V>>>;

#[tauri::command]
async fn cached_parse_agent_library(
    repo_path: String,
    cache: tauri::State<'_, Cache<String, AgentLibrary>>,
) -> Result<AgentLibrary, String> {
    // キャッシュをチェック
    {
        let cache_read = cache.read().await;
        if let Some(cached) = cache_read.get(&repo_path) {
            return Ok(cached.clone());
        }
    }

    // キャッシュになければパース
    let library = parse_agent_library_internal(repo_path.clone()).await?;

    // キャッシュに保存
    {
        let mut cache_write = cache.write().await;
        cache_write.insert(repo_path, library.clone());
    }

    Ok(library)
}
```

このガイドに従って、効率的で保守性の高いTauriコマンドを実装してください。
