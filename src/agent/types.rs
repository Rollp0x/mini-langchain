use crate::llm::traits::LLM;
use std::sync::Arc;
use crate::tools::traits::Tool;
use std::collections::HashMap;
use super::error::AgentError;
use crate::llm::tokens::TokenUsage;
use serde::{Serialize, Deserialize};

/// High-level agent that holds an LLM and a set of tools, plus simple agent state.
pub struct Agent {
    /// A short, human-friendly name for the agent instance.
    pub name: String,

    /// The LLM implementation used to generate responses/thoughts.
    pub llm: Arc<dyn LLM>,

    /// Registered tools the agent may call by name.
    pub tools: HashMap<String, Arc<dyn Tool>>,

    /// Optional system prompt / instructions provided to the LLM describing
    /// the agent's role and available behaviors.
    pub system_prompt: Option<String>,

    /// Simple short-term memory / conversation context kept by the agent.
    /// We store `Message` objects elsewhere in the crate; for a minimal
    /// implementation we keep user-visible strings here.
    pub memory: Vec<String>,

    /// Maximum iterations when running a looped decision process.
    pub max_iterations: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AgentResult {
    pub tokens: TokenUsage,
    pub generation: String,
}

pub type AgentExecuteResult = Result<AgentResult, AgentError>;



