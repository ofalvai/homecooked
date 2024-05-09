use actix_web::{post, web, Responder};
use actix_web_lab::sse;
use futures::StreamExt;
use serde::Deserialize;

use crate::{
    server::{errors::LlmError, AppState, STREAM_KEEPALIVE},
    tools::{self, ToolUseEvent},
};

#[derive(Deserialize)]
struct YoutubeRequest {
    url: String,
    prompt: Option<String>,
}

#[derive(Deserialize)]
struct WebPageRequest {
    url: String,
    prompt: Option<String>,
}

#[derive(Deserialize)]
struct ReadwiseRequest {
    query: String,
}

#[post("/v1/tools/youtube")]
async fn youtube(
    req: web::Json<YoutubeRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, LlmError> {
    let stream = tools::youtube::run(
        data.config.clone(),
        req.url.clone(),
        req.prompt.clone(),
        None,
    )
    .map(map_to_sse);

    Ok(sse::Sse::from_stream(stream).with_keep_alive(STREAM_KEEPALIVE))
}

#[post("/v1/tools/web")]
async fn web_page(
    req: web::Json<WebPageRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, LlmError> {
    let stream = tools::web::run(
        data.config.clone(),
        req.url.clone(),
        req.prompt.clone(),
        None,
    )
    .map(map_to_sse);

    Ok(sse::Sse::from_stream(stream).with_keep_alive(STREAM_KEEPALIVE))
}

#[post("/v1/tools/readwise")]
async fn readwise(
    req: web::Json<ReadwiseRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, LlmError> {
    let stream = tools::readwise::run(data.config.clone(), req.query.clone()).map(map_to_sse);

    Ok(sse::Sse::from_stream(stream).with_keep_alive(STREAM_KEEPALIVE))
}

fn map_to_sse(tool_event: ToolUseEvent) -> anyhow::Result<sse::Event> {
    let data = sse::Data::new_json(tool_event)?;
    let event = sse::Event::Data(data);
    Ok::<_, anyhow::Error>(event)
}
