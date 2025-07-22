use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIndex {
    pub mcp_endpoints: Vec<McpEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEndpoint {
    pub id: String,
    pub label: String,
    pub description: String,
    pub prompt_file: String,
    pub trigger: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLibrary {
    pub index: AgentIndex,
    #[serde(skip)]
    pub base_path: PathBuf,
    pub prompts: Vec<Prompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content: String,
    #[serde(skip)]
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    #[serde(skip)]
    pub path: PathBuf,
    pub agent_library: Option<AgentLibrary>,
}