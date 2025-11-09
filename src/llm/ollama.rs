
use std::sync::Arc;
use serde_json::error::Error as SerdeJsonError;
use serde::de::Error as SerdeDeError;
use async_stream::stream as async_stream;
use futures::{
    FutureExt,
    future::BoxFuture,
    stream::BoxStream
};


use crate::message::Message;
use crate::tools::stream::StreamData;
use crate::message::MessageRole as MsgRole;

use crate::llm::{
    traits::LLM,
    tokens::TokenUsage,
    error::LLMError,
    GenerateResult,
    LLMResult,
};

/// Default model name used when no model is specified.
/// Adjust this to match the model name you have installed in your local Ollama.
/// Common names: "llama3.2", "llama3", "llama2", or custom names from `ollama list`.
pub const DEFAULT_MODEL: &str = "llama3.2";

pub use ollama_rs::{
    error::OllamaError,
    Ollama as OllamaClient,
    models::ModelOptions,
    generation::{
        chat::{request::ChatMessageRequest,ChatMessage, MessageRole},
        completion::request::GenerationRequest,
    }
};


#[derive(Debug, Clone)]
pub struct Ollama {
    pub(crate) client: Arc<OllamaClient>,
    pub(crate) model: String,
    pub(crate) options: Option<ModelOptions>,
}
impl Ollama {
    /// Create an `Ollama` wrapper using the provided client and the default model.
    ///
    /// If your local Ollama uses a different default model name, change
    /// `DEFAULT_MODEL` or call `Ollama::with_model`.
    pub fn new(client: Arc<OllamaClient>) -> Self {
        Self {
            client,
            model: DEFAULT_MODEL.to_string(),
            options: None,
        }
    }

    /// Create an `Ollama` wrapper with an explicit model name.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Create an `Ollama` wrapper with additional generation options.
    pub fn with_options(mut self, options: ModelOptions) -> Self {
        self.options = Some(options);
        self
    }

    fn generate_request(&self, messages: &[Message]) -> ChatMessageRequest {
        let mapped_messages = messages.iter().map(|message| message.into()).collect();
        ChatMessageRequest::new(self.model.clone(), mapped_messages).think(true)
    }


}

impl Default for Ollama {
    fn default() -> Self {
        let client = Arc::new(OllamaClient::default());
        Ollama::new(client)
    }
}




impl From<&Message> for ChatMessage {
    fn from(message: &Message) -> Self {
        let role = match message.role {
            MsgRole::System => MessageRole::System,
            MsgRole::User => MessageRole::User,
            MsgRole::Assistant => MessageRole::Assistant,
            MsgRole::ToolResponce => MessageRole::Tool,
            MsgRole::Tool | MsgRole::Developer => MessageRole::System,

        };
        ChatMessage::new(role, message.content.clone())
    }
}


impl LLM for Ollama {
    fn generate<'a>(&'a self, messages: &'a [Message]) -> BoxFuture<'a, LLMResult<GenerateResult>> {
        async move {
            // build request (this clones/moves as generate_request does)
            let request = self.generate_request(messages);

            // perform async call and map errors into our LLMError
            let response = self
                .client
                .send_chat_messages(request)
                .await
                .map_err(|e| LLMError::InvalidResponse(format!("{:?}", e)))?;
            let generation = response.message.content.clone();

            let tokens = if let Some(final_data) = response.final_data {
                let prompt_tokens = final_data.prompt_eval_count as u32;
                let completion_tokens = final_data.eval_count as u32;

                TokenUsage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                }
            } else {
                TokenUsage::default()
            };
            // Robustly extract tool_calls: [{name, args}] from generation text
            let mut call_tools: Vec<crate::llm::CallInfo> = Vec::new();
            let parsed_json_res = serde_json::from_str::<serde_json::Value>(&generation)
                .or_else(|_err| {
                    if let (Some(start), Some(end)) = (generation.find('{'), generation.rfind('}')) {
                        let sub = &generation[start..=end];
                        serde_json::from_str::<serde_json::Value>(sub)
                    } else {
                        Err(SerdeJsonError::custom("no json substring"))
                    }
                });
            if let Ok(parsed) = parsed_json_res {
                if let Some(arr) = parsed.get("tool_calls").and_then(|v| v.as_array()) {
                    for entry in arr.iter() {
                        if let Some(obj) = entry.as_object() {
                            if let Some(name_val) = obj.get("name").and_then(|v| v.as_str()) {
                                let name = name_val.to_string();
                                let args = obj.get("args").cloned().unwrap_or_else(|| serde_json::json!({}));
                                call_tools.push(crate::llm::CallInfo { name, args });
                            }
                        }
                    }
                }
            }

            Ok(GenerateResult { tokens, generation, call_tools })
        }
        .boxed()
    }

    fn stream<'a>(&'a self, messages: &'a [Message]) -> BoxStream<'a, LLMResult<StreamData>> {
        // Keep borrowed references `self` and `messages` in scope for the async generator.
        let this = self;
        let msgs = messages;

        let s = async_stream! {
            // Prefer upstream streaming if feature enabled
            #[cfg(feature = "ollama_stream")]
            {
                let request = this.generate_request(msgs);
                // get upstream stream (awaitable)
                let upstream = match this.client.send_chat_messages_stream(request).await {
                    Ok(s) => s,
                    Err(e) => {
                        yield Err(LLMError::InvalidResponse(format!("{:?}", e)));
                        return;
                    }
                };

                futures::pin_mut!(upstream);
                while let Some(item_res) = upstream.next().await {
                    match item_res {
                        Ok(item) => {
                            let value = serde_json::to_value(&item).unwrap_or_default();
                            let content = item.message.content.clone();
                            let tokens = item.final_data.map(|final_data| crate::llm::tokens::TokenUsage {
                                prompt_tokens: final_data.prompt_eval_count as u32,
                                completion_tokens: final_data.eval_count as u32,
                                total_tokens: final_data.prompt_eval_count as u32 + final_data.eval_count as u32,
                            });
                            yield Ok(StreamData::new(value, tokens, content));
                        }
                        Err(e) => {
                            // map upstream error to LLMError
                            yield Err(LLMError::InvalidResponse(format!("{:?}", e)));
                        }
                    }
                }
            }

            // Fallback: call non-streaming endpoint and yield single item
            #[cfg(not(feature = "ollama_stream"))]
            {
                let request = this.generate_request(msgs);
                match this.client.send_chat_messages(request).await {
                    Ok(response) => {
                        let content = response.message.content.clone();
                        let value = serde_json::to_value(response.message).unwrap_or_default();

                        let tokens = response.final_data.map(|final_data| {
                            let prompt_tokens = final_data.prompt_eval_count as u32;
                            let completion_tokens = final_data.eval_count as u32;
                            TokenUsage {
                                prompt_tokens,
                                completion_tokens,
                                total_tokens: prompt_tokens + completion_tokens,
                            }
                        });

                        let sd = StreamData::new(value, tokens, content);
                        yield Ok(sd);
                    }
                    Err(e) => {
                        yield Err(LLMError::InvalidResponse(format!("{:?}", e)));
                    }
                }
            }
        };

        Box::pin(s)
    }
}