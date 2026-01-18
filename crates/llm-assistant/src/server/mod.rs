use actix_cors::Cors;
use actix_files::Files;
use actix_web::{middleware::Logger, post, web, App, HttpServer, Responder};
use anyhow::Context;

use crate::{
    config::Config,
    models::get_client,
    server::{errors::LlmError, openai_types::CreateChatCompletionRequest},
};

mod errors;
mod openai_adapter;
mod openai_types;
mod tools;

struct AppState {
    config: Config,
}

static STREAM_KEEPALIVE: std::time::Duration = std::time::Duration::from_secs(10);

pub async fn start(config: Config, port: Option<u16>) -> anyhow::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(AppState {
                config: config.clone(),
            }))
            .service(completions)
            .service(tools::youtube)
            .service(tools::web_page)
            .service(tools::readwise)
            .service(Files::new("/config", config.persona_file.parent().unwrap()))
        // TODO: make this more robust
    })
    .bind(("0.0.0.0", port.unwrap_or(8080)))?
    .run()
    .await
    .context("Server error")
}

#[post("/v1/chat/completions")]
async fn completions(
    req: web::Json<CreateChatCompletionRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, LlmError> {
    let req = req.into_inner();
    let params = openai_adapter::completion_params(&req);
    let conv = openai_adapter::conversation(&req)?;
    let client = get_client(&req.model, &data.config)?;
    let stream = client.completion_stream(conv, params).await?;
    Ok(openai_adapter::adapt_stream(stream, req.model))
}
