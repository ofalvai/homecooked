use actix_web::{post, web, Responder};
use serde::Deserialize;

use crate::{
    server::{errors::LlmError, openai_adapter},
    tools,
};

#[derive(Deserialize)]
struct YoutubeRequest {
    url: String,
    prompt: Option<String>,
}

#[post("/v1/tools/youtube")]
async fn youtube(req: web::Json<YoutubeRequest>) -> Result<impl Responder, LlmError> {
    let model = tools::youtube::DEFAULT_MODEL.to_string();
    let stream =
        tools::youtube::run(req.url.clone(), req.prompt.clone(), Some(&model), std::io::sink()).await?;

    Ok(openai_adapter::adapt_stream(
        stream,
        model,
    ))
}
