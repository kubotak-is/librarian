use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryConfig {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_active: bool,
    pub last_updated: String,
    pub mcp_server: Option<McpServerConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct McpServerConfig {
    pub port: u16,
    pub status: String, // "running", "stopped", "error"
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub repositories: Vec<RepositoryConfig>,
    pub last_opened_repository: Option<String>,
    pub theme: String,
    pub auto_start_servers: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            repositories: Vec::new(),
            last_opened_repository: None,
            theme: "light".to_string(),
            auto_start_servers: true,
        }
    }
}

impl AppConfig {
    /// アプリケーション設定ファイルのパスを取得
    pub fn config_file_path(app: &AppHandle) -> Result<PathBuf, String> {
        let config_dir = app.path().app_config_dir()
            .map_err(|e| format!("Failed to get config directory: {e}"))?;
        
        Ok(config_dir.join("config.json"))
    }

    /// 設定ファイルから読み込み
    pub async fn load(app: &AppHandle) -> Result<Self, String> {
        let config_path = Self::config_file_path(app)?;
        
        if !config_path.exists() {
            // 設定ファイルが存在しない場合は、デフォルト設定を作成
            let default_config = Self::default();
            default_config.save(app).await?;
            return Ok(default_config);
        }
        
        let content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| format!("Failed to read config file: {e}"))?;
        
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {e}"))?;
        
        Ok(config)
    }

    /// 設定ファイルに保存
    pub async fn save(&self, app: &AppHandle) -> Result<(), String> {
        let config_path = Self::config_file_path(app)?;
        
        // 設定ディレクトリを作成
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {e}"))?;
        
        tokio::fs::write(&config_path, json)
            .await
            .map_err(|e| format!("Failed to write config file: {e}"))?;
        
        Ok(())
    }

    /// リポジトリを追加
    pub fn add_repository(&mut self, repository: RepositoryConfig) {
        // 同じIDのリポジトリが存在する場合は更新
        if let Some(existing) = self.repositories.iter_mut().find(|r| r.id == repository.id) {
            *existing = repository;
        } else {
            self.repositories.push(repository);
        }
    }

    /// リポジトリを削除
    pub fn remove_repository(&mut self, repository_id: &str) -> bool {
        let initial_len = self.repositories.len();
        self.repositories.retain(|r| r.id != repository_id);
        self.repositories.len() != initial_len
    }

    /// リポジトリを取得
    #[must_use] pub fn get_repository(&self, repository_id: &str) -> Option<&RepositoryConfig> {
        self.repositories.iter().find(|r| r.id == repository_id)
    }

    /// リポジトリを更新
    pub fn update_repository(&mut self, repository_id: &str, update_fn: impl FnOnce(&mut RepositoryConfig)) -> bool {
        if let Some(repository) = self.repositories.iter_mut().find(|r| r.id == repository_id) {
            update_fn(repository);
            true
        } else {
            false
        }
    }

    /// アクティブなリポジトリのリストを取得
    #[must_use] pub fn get_active_repositories(&self) -> Vec<&RepositoryConfig> {
        self.repositories.iter().filter(|r| r.is_active).collect()
    }

    /// 実行中のMCPサーバーのリストを取得
    #[must_use] pub fn get_running_servers(&self) -> Vec<&RepositoryConfig> {
        self.repositories
            .iter()
            .filter(|r| {
                r.mcp_server
                    .as_ref()
                    .is_some_and(|s| s.status == "running")
            })
            .collect()
    }
}

/// リポジトリ設定の便利な作成関数
impl RepositoryConfig {
    #[must_use] pub fn new(id: String, name: String, path: String) -> Self {
        Self {
            id,
            name,
            path,
            is_active: true,
            last_updated: chrono::Utc::now().to_rfc3339(),
            mcp_server: None,
        }
    }

    #[must_use] pub fn with_mcp_server(mut self, port: u16, status: String) -> Self {
        self.mcp_server = Some(McpServerConfig { port, status });
        self
    }

    pub fn update_last_updated(&mut self) {
        self.last_updated = chrono::Utc::now().to_rfc3339();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.repositories.len(), 0);
        assert_eq!(config.theme, "light");
        assert!(config.auto_start_servers);
    }

    #[test]
    fn test_repository_operations() {
        let mut config = AppConfig::default();
        
        let repo = RepositoryConfig::new(
            "test-repo".to_string(),
            "Test Repository".to_string(),
            "/path/to/repo".to_string(),
        );
        
        // リポジトリ追加
        config.add_repository(repo.clone());
        assert_eq!(config.repositories.len(), 1);
        
        // リポジトリ取得
        let retrieved = config.get_repository("test-repo");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Repository");
        
        // リポジトリ更新
        let updated = config.update_repository("test-repo", |r| {
            r.name = "Updated Repository".to_string();
        });
        assert!(updated);
        assert_eq!(config.get_repository("test-repo").unwrap().name, "Updated Repository");
        
        // リポジトリ削除
        let removed = config.remove_repository("test-repo");
        assert!(removed);
        assert_eq!(config.repositories.len(), 0);
    }

    #[test]
    fn test_repository_filtering() {
        let mut config = AppConfig::default();
        
        let repo1 = RepositoryConfig::new(
            "repo1".to_string(),
            "Repository 1".to_string(),
            "/path/to/repo1".to_string(),
        ).with_mcp_server(9500, "running".to_string());
        
        let mut repo2 = RepositoryConfig::new(
            "repo2".to_string(),
            "Repository 2".to_string(),
            "/path/to/repo2".to_string(),
        );
        repo2.is_active = false;
        
        config.add_repository(repo1);
        config.add_repository(repo2);
        
        // アクティブなリポジトリのフィルタリング
        let active_repos = config.get_active_repositories();
        assert_eq!(active_repos.len(), 1);
        assert_eq!(active_repos[0].id, "repo1");
        
        // 実行中のサーバーのフィルタリング
        let running_servers = config.get_running_servers();
        assert_eq!(running_servers.len(), 1);
        assert_eq!(running_servers[0].id, "repo1");
    }
}