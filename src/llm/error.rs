use super::ollama::OllamaError;


#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("Ollama error: {0}")]
    OllamaError(#[from] OllamaError),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Streaming not supported")]
    StreamNotSupported,

    #[error("JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}