use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::conversation::Conversation;

pub mod anthropic;
#[cfg(feature = "llama-cpp")]
pub mod llama;
pub mod openai;

pub struct CompletionParams {
    pub temp: f32,
    pub max_tokens: u16,
}

impl Default for CompletionParams {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temp: 0.8,
        }
    }
}

#[async_trait]
pub trait Client {
    async fn completion(
        &self,
        conversation: Conversation,
        params: CompletionParams,
    ) -> Result<CompletionResponse, CompletionError>;

    async fn completion_stream(
        &self,
        conversation: Conversation,
        params: CompletionParams,
    ) -> Result<CompletionResponseStream, CompletionError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CompletionError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("{0} API error: {1}")]
    ApiError(String, String),

    #[error("response stream failed: {0}")]
    StreamError(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("unknown error: {0}")]
    UnknownError(String),
}

#[derive(Debug)]
pub struct CompletionResponse {
    pub id: String,
    pub content: String,
}

#[derive(Debug)]
pub struct CompletionResponseDelta {
    pub id: String,
    pub content: String,
}

pub type CompletionResponseStream =
    Pin<Box<dyn Stream<Item = Result<CompletionResponseDelta, CompletionError>> + Send>>;
