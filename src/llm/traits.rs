use std::sync::Arc;
use crate::message::Message;
use crate::llm::{LLMResult, GenerateResult};
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use crate::tools::stream::StreamData;

/// Convert a concrete L into an `Arc<dyn LLM + Send + Sync>`.
/// Convenience so callers can do `llm_to_arc_dyn(MyLlm::new(...))`.
pub fn llm_to_arc_dyn<L>(llm: L) -> Arc<dyn LLM + Send + Sync>
where
    L: 'static + LLM + Send + Sync,
{
    Arc::new(llm)
}

/// Core LLM trait. This version uses BoxFuture/BoxStream with explicit lifetimes
/// so implementations can borrow the input `&[Message]` and avoid cloning large messages.
///
/// Note:
/// - We intentionally do not use `async_trait` here so that returned futures/streams
///   can be annotated with the input lifetime `'a` (avoids unnecessary cloning when desired).
/// - If implementations need to spawn background tasks for streaming, they must first
///   make the necessary data `'static` (e.g. clone or use Arc inside Message).
pub trait LLM: Send + Sync {
    /// Produce a generation result. The returned future may borrow from `messages`.
    fn generate<'a>(&'a self, messages: &'a [Message]) -> BoxFuture<'a, LLMResult<GenerateResult>>;


    /// Return a stream that may borrow from `messages`. The stream lifetime is tied to `'a`.
    fn stream<'a>(&'a self, messages: &'a [Message]) -> BoxStream<'a, LLMResult<StreamData>>;
}