use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use self::openai::CompletionArgs;
use crate::prompt::Message;

pub mod openai;

#[async_trait]
pub trait Client {
    type CompletionArgs;

    async fn completion(
        &self,
        messages: Vec<Message>,
        args: CompletionArgs,
    ) -> Result<CompletionResponse, CompletionError>;

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
        args: CompletionArgs,
    ) -> Result<CompletionResponseStream, CompletionError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CompletionError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("API returned error: {0}")]
    ApiError(String),

    #[error("response stream failed: {0}")]
    StreamError(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("unknown error: {0}")]
    UnknownError(String),
}
pub struct CompletionResponse {
    pub id: String,
    pub content: String,
}

pub struct CompletionResponseDelta {
    pub id: String,
    pub content: String,
}

pub type CompletionResponseStream =
    Pin<Box<dyn Stream<Item = Result<CompletionResponseDelta, CompletionError>> + Send>>;

