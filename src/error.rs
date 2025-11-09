use crate::llm::error::LLMError;
use crate::tools::error::ToolError;
use crate::agent::error::AgentError;
use crate::config::ConfigError;


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LLM error: {0}")]
    LLM(#[from] LLMError),
    
    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),
    
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;