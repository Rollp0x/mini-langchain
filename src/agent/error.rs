use crate::tools::error::ToolError;
use crate::llm::error::LLMError;

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution error: {0}")]
    ToolExecutionError(#[from] ToolError),


    #[error("LLM error: {0}")]
    LLMExecutionError(#[from] LLMError),

    #[error("Maximum iterations exceeded: {0}")]
    MaxIterationsExceeded(usize)

}
