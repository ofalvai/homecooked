use actix_cors::Cors;
use actix_files::Files;
use actix_web::{middleware::Logger, post, web, App, HttpServer, Responder};
use anyhow::Context;

use crate::{
    models::get_client,
    server::{errors::LlmError, openai_types::CreateChatCompletionRequest},
};

mod errors;
mod openai_adapter;
mod openai_types;
mod tools;

pub async fn start(port: Option<u16>) -> anyhow::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(completions)
            .service(tools::youtube)
            .service(Files::new("/config", "./src/config"))
    })
    .bind(("127.0.0.1", port.unwrap_or(8080)))?
    .run()
    .await
    .context("Server error")
}

#[post("/v1/chat/completions")]
async fn completions(
    req: web::Json<CreateChatCompletionRequest>,
) -> Result<impl Responder, LlmError> {
    let req = req.into_inner();
    let params = openai_adapter::completion_params(&req);
    let conv = openai_adapter::conversation(&req)?;
    let client = get_client(&req.model)?;
    let stream = client.completion_stream(conv, params).await?;
    Ok(openai_adapter::adapt_stream(stream, req.model))
}
