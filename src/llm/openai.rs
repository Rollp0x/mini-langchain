// 参考：https://github.com/64bit/async-openai/blob/main/examples/tool-call/src/main.rs
pub use async_openai::{
    Client, config::{Config, OpenAIConfig}
};
use serde_json::{json, Value};
use crate::message::Message;
use crate::tools::stream::StreamData;
use serde::{Serialize, Deserialize};
use crate::llm::{
    traits::LLM,
    tokens::TokenUsage,
    error::LLMError,
    GenerateResult,
    LLMResult,
};

use std::sync::Arc;
use serde_json::error::Error as SerdeJsonError;
use serde::de::Error as SerdeDeError;
use async_stream::stream as async_stream;
use futures::{
    FutureExt,
    future::BoxFuture,
    stream::BoxStream
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunction{
    #[serde(rename = "type")]
    pub f_type: &'static str,
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub strict: bool,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// How many completions to generate for each prompt.
    /// **Note:** Because this parameter generates many completions, it can quickly consume your token quota. Use carefully and ensure that you have reasonable settings for `max_tokens` and `stop`.
    ///
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>, // min:1 max: 128, default: 1

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>, // nullable: true

    /// A unique identifier representing your end-user, which will help OpenAI to monitor and detect abuse. [Learn more](https://platform.openai.com/docs/usage-policies/end-user-ids).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

pub struct OpenAI{
    pub client:Client<OpenAIConfig>,
    pub options:Option<CompletionOptions>

}

impl OpenAI {
    pub fn new() -> Self {
        let client = Client::new();
        Self {
            client,
            options: None,
        }
    }

    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            options: None,
        }
    }

    pub fn with_options(mut self, options: CompletionOptions) -> Self {
        self.options = Some(options);
        self
    }
}

impl Default for OpenAI {
    fn default() -> Self {
        Self::new()
    }
}

impl LLM for OpenAI {
    fn generate<'a>(&'a self, messages: &'a [Message]) -> BoxFuture<'a, LLMResult<GenerateResult>> {
        // Implementation for generating text using OpenAI API
        unimplemented!()
    }

    fn stream<'a>(&'a self, messages: &'a [Message]) -> BoxStream<'a, LLMResult<StreamData>> {
        // Implementation for streaming text using OpenAI API
        unimplemented!()
    }
}

pub struct OpenAIRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub tools: Vec<OpenAIFunction>,
    pub tool_choice: Option<String>, // "auto" | "none"
}

