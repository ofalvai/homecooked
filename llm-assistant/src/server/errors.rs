use llm_toolkit::provider::CompletionError;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unhandled error: {0}")]
    UnhandledError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl LlmError {
    fn name(&self) -> String {
        match self {
            LlmError::InvalidInput(_) => "Invalid input".to_string(),
            LlmError::UnhandledError(_) => "Unhandled error".to_string(),
        }
    }
}

impl actix_web::error::ResponseError for LlmError {
    fn error_response(&self) -> actix_web::HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            error: self.name(),
            message: self.to_string(),
        };
        actix_web::HttpResponse::build(status_code).json(error_response)
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            LlmError::InvalidInput(_) => actix_web::http::StatusCode::BAD_REQUEST,
            LlmError::UnhandledError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
impl From<anyhow::Error> for LlmError {
    fn from(err: anyhow::Error) -> LlmError {
        let err_chain = err.chain()
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        LlmError::UnhandledError(err_chain)
    }
}

impl From<llm_toolkit::provider::CompletionError> for LlmError {
    fn from(err: CompletionError) -> LlmError {
        match err {
            CompletionError::InvalidArgument(msg) => LlmError::InvalidInput(msg),
            CompletionError::ApiError(_, _) => LlmError::UnhandledError(err.to_string()),
            CompletionError::StreamError(msg) => LlmError::UnhandledError(msg),
            CompletionError::InvalidResponse(msg) => LlmError::UnhandledError(msg),
            CompletionError::UnknownError(msg) => LlmError::UnhandledError(msg),
        }
    }
}
