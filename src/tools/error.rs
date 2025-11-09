use crate::llm::error::LLMError;


#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool execution error in '{name}': {reason}")]
    ExecutionError {
        name: String,
        reason: String,
    },

    #[error("LLM error: {0}")]
    LLMError(#[from] LLMError),
    
    #[error("Tool parameters do not match: {0}")]
    ParamsNotMatched(String),

    #[error("Unknown tool error")]
    Unknown,
}
