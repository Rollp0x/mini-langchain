pub mod traits;
pub mod openai;
pub mod anthropic;
pub mod qwen;
pub mod deepseek;
pub mod ollama;
pub mod tokens;
pub mod error;


use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use tokens::TokenUsage;

/// Result of a text generation from an LLM.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GenerateResult {
    pub tokens: TokenUsage,
    pub generation: String,
    /// Optional structured tool calls the LLM signaled during this generation.
    /// Each entry contains the tool name and the arguments object the LLM wants
    /// the agent to pass when invoking that tool.
    #[serde(default)]
    pub tool_calls: Vec<CallInfo>,
}

/// Structured information about a single tool call requested by the LLM.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CallInfo {
    pub name: String,
    #[serde(default)]
    pub args: JsonValue,
}

/// Result type for LLM operations.
pub type LLMResult<T> = std::result::Result<T, error::LLMError>;