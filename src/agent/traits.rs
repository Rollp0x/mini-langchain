

use super::types::AgentExecuteResult;

/// Trait describing runtime operations an agent can perform.
#[async_trait::async_trait]
pub trait AgentRunner: Send + Sync {
    /// Call the LLM with a prompt and return the generation result.
    async fn call_llm(&self, prompt: &str) -> AgentExecuteResult;
}