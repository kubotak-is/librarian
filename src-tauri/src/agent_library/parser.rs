use super::types::{AgentLibrary, AgentIndex, Prompt};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;
use std::collections::HashMap;
use std::sync::Mutex;
// use once_cell::sync::Lazy; // 現在未使用

// Agent Libraryキャッシュの実装（シンプルなHashMapベース）
type CacheEntry = (AgentLibrary, SystemTime);
static AGENT_LIBRARY_CACHE: std::sync::LazyLock<Mutex<HashMap<PathBuf, CacheEntry>>> = std::sync::LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub struct AgentLibraryParser;

impl AgentLibraryParser {
    pub fn parse(repo_path: &Path) -> Result<AgentLibrary> {
        let agent_lib_path = repo_path.join(".agent_library");
        
        if !agent_lib_path.exists() {
            return Err(anyhow::anyhow!(".agent_library directory not found in {}", repo_path.display()));
        }

        // ディレクトリの最終更新時間を確認
        let dir_modified = fs::metadata(&agent_lib_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        // キャッシュから確認
        if let Ok(cache) = AGENT_LIBRARY_CACHE.lock() {
            if let Some((cached_library, cached_time)) = cache.get(&agent_lib_path) {
                if *cached_time >= dir_modified {
                    tracing::debug!("Serving agent library from cache: {}", agent_lib_path.display());
                    return Ok(cached_library.clone());
                }
            }
        }

        tracing::debug!("Cache miss, parsing agent library: {}", agent_lib_path.display());
        let index = Self::parse_index(&agent_lib_path)?;
        let prompts = Self::parse_prompts(&agent_lib_path, &index)?;

        let library = AgentLibrary {
            index,
            base_path: agent_lib_path.clone(),
            prompts,
        };

        // キャッシュに保存
        if let Ok(mut cache) = AGENT_LIBRARY_CACHE.lock() {
            cache.insert(agent_lib_path, (library.clone(), dir_modified));
        }

        Ok(library)
    }

    fn parse_index(agent_lib_path: &Path) -> Result<AgentIndex> {
        let index_path = agent_lib_path.join("agent_index.yml");
        
        if !index_path.exists() {
            return Err(anyhow::anyhow!("agent_index.yml not found in {}", agent_lib_path.display()));
        }

        let content = fs::read_to_string(&index_path)
            .with_context(|| format!("Failed to read {}", index_path.display()))?;

        let index: AgentIndex = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML from {}", index_path.display()))?;

        Ok(index)
    }

    fn parse_prompts(agent_lib_path: &Path, index: &AgentIndex) -> Result<Vec<Prompt>> {
        let mut prompts = Vec::new();

        for endpoint in &index.mcp_endpoints {
            let prompt_path = agent_lib_path.join(&endpoint.prompt_file);
            
            if !prompt_path.exists() {
                eprintln!("Warning: Prompt file {} not found", prompt_path.display());
                continue;
            }

            let content = fs::read_to_string(&prompt_path)
                .with_context(|| format!("Failed to read {}", prompt_path.display()))?;

            let prompt = Prompt {
                id: endpoint.id.clone(),
                title: endpoint.label.clone(),
                description: endpoint.description.clone(),
                content,
                file_path: prompt_path.clone(),
            };

            prompts.push(prompt);
        }

        Ok(prompts)
    }

    pub fn find_repositories(search_paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut repositories = Vec::new();

        for search_path in search_paths {
            if !search_path.exists() {
                continue;
            }

            for entry in WalkDir::new(search_path)
                .max_depth(3) // Limit depth to avoid deep recursion
                .into_iter()
                .filter_map(std::result::Result::ok)
            {
                let path = entry.path();
                if path.is_dir() && path.file_name() == Some(std::ffi::OsStr::new(".agent_library")) {
                    if let Some(parent) = path.parent() {
                        repositories.push(parent.to_path_buf());
                    }
                }
            }
        }

        Ok(repositories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_agent_library(dir: &Path) -> Result<()> {
        let agent_lib_dir = dir.join(".agent_library");
        fs::create_dir_all(&agent_lib_dir)?;

        // Create agent_index.yml
        let index_content = r#"
name: "Test Library"
description: "Test Agent Library"
version: "1.0.0"
mcp_endpoints:
  - id: "test_prompt"
    label: "Test Prompt"
    description: "A test prompt for unit testing"
    prompt_file: "test_prompt.md"
"#;
        fs::write(agent_lib_dir.join("agent_index.yml"), index_content)?;

        // Create test prompt file
        let prompt_content = "# Test Prompt\n\nThis is a test prompt for unit testing.";
        fs::write(agent_lib_dir.join("test_prompt.md"), prompt_content)?;

        Ok(())
    }

    #[test]
    fn test_parse_valid_agent_library() {
        let temp_dir = TempDir::new().unwrap();
        create_test_agent_library(temp_dir.path()).unwrap();

        let result = AgentLibraryParser::parse(temp_dir.path());
        assert!(result.is_ok());

        let library = result.unwrap();
        assert_eq!(library.index.mcp_endpoints.len(), 1);
        assert_eq!(library.prompts.len(), 1);
        assert_eq!(library.prompts[0].id, "test_prompt");
        assert_eq!(library.prompts[0].title, "Test Prompt");
        assert!(library.prompts[0].content.contains("Test Prompt"));
    }

    #[test]
    fn test_parse_missing_agent_library() {
        let temp_dir = TempDir::new().unwrap();
        // .agent_libraryディレクトリを作成しない

        let result = AgentLibraryParser::parse(temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".agent_library directory not found"));
    }

    #[test]
    fn test_parse_missing_index_file() {
        let temp_dir = TempDir::new().unwrap();
        let agent_lib_dir = temp_dir.path().join(".agent_library");
        fs::create_dir_all(&agent_lib_dir).unwrap();
        // agent_index.ymlを作成しない

        let result = AgentLibraryParser::parse(temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("agent_index.yml not found"));
    }

    #[test]
    fn test_find_repositories() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple test repositories
        let repo1 = temp_dir.path().join("repo1");
        let repo2 = temp_dir.path().join("repo2");
        let repo3 = temp_dir.path().join("nested").join("repo3");
        
        create_test_agent_library(&repo1).unwrap();
        create_test_agent_library(&repo2).unwrap();
        fs::create_dir_all(&repo3).unwrap();
        create_test_agent_library(&repo3).unwrap();

        // Create a directory without .agent_library
        fs::create_dir_all(temp_dir.path().join("not_a_repo")).unwrap();

        let search_paths = vec![temp_dir.path().to_path_buf()];
        let result = AgentLibraryParser::find_repositories(&search_paths);
        assert!(result.is_ok());

        let repositories = result.unwrap();
        assert_eq!(repositories.len(), 3);
        assert!(repositories.contains(&repo1));
        assert!(repositories.contains(&repo2));
        assert!(repositories.contains(&repo3));
    }

    #[test]
    fn test_cache_functionality() {
        let temp_dir = TempDir::new().unwrap();
        create_test_agent_library(temp_dir.path()).unwrap();

        // First parse - should cache the result
        let result1 = AgentLibraryParser::parse(temp_dir.path());
        assert!(result1.is_ok());

        // Second parse - should use cache
        let result2 = AgentLibraryParser::parse(temp_dir.path());
        assert!(result2.is_ok());

        // Results should be identical
        let library1 = result1.unwrap();
        let library2 = result2.unwrap();
        assert_eq!(library1.index.mcp_endpoints.len(), library2.index.mcp_endpoints.len());
        assert_eq!(library1.prompts.len(), library2.prompts.len());
    }
}