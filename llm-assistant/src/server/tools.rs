use actix_web::{post, web, Responder};
use actix_web_lab::sse;
use futures::StreamExt;
use serde::Deserialize;

use crate::{
    server::{errors::LlmError, AppState, STREAM_KEEPALIVE},
    tools,
};

#[derive(Deserialize)]
struct YoutubeRequest {
    url: String,
    prompt: Option<String>,
}

#[post("/v1/tools/youtube")]
async fn youtube(
    req: web::Json<YoutubeRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, LlmError> {
    let model = tools::youtube::DEFAULT_MODEL.to_string();
    let stream = tools::youtube::run(
        data.config.clone(),
        req.url.clone(),
        req.prompt.clone(),
        Some(model),
    )
    .map(|step| {
        let data = sse::Data::new_json(step)?;
        let event = sse::Event::Data(data);
        Ok::<_, anyhow::Error>(event)
    });

    Ok(sse::Sse::from_stream(stream).with_keep_alive(STREAM_KEEPALIVE))
}
