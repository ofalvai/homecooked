pub mod readwise;
pub mod web_article;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("request error: {0}")]
    NetworkRequestError(String),

    #[error("data processing error: {0}")]
    ProcessingError(String),

    #[error("unknown error: {0}")]
    UnknownError(String),
}
