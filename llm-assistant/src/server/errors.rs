use derive_more::{Display, Error};
use llm_toolkit::provider::CompletionError;
use std::{error::Error, fmt::Display};

#[derive(Debug, Display, Error)]
#[display(fmt = "server error: {}", err)]
pub struct LlmError {
    err: anyhow::Error,
}
impl actix_web::error::ResponseError for LlmError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::InternalServerError().json(self.err.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
impl From<anyhow::Error> for LlmError {
    fn from(err: anyhow::Error) -> LlmError {
        LlmError { err }
    }
}
