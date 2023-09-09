use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{middleware::Logger, post, web, App, HttpServer, Responder};
use actix_web_lab::sse::{self};
use anyhow::Context;
use futures::{stream, StreamExt};
use llm_toolkit::{
    conversation::{Conversation, Message, Role},
    provider::CompletionParams,
};
use log::error;
use uuid::Uuid;

use crate::{
    models::get_client,
    server::{errors::LlmError, openai_types::CreateChatCompletionRequest},
};

use self::openai_types::{ChatCompletionStreamResponseDelta, CreateChatCompletionStreamResponse};

mod errors;
mod openai_types;

pub async fn start(port: Option<u16>) -> anyhow::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(completion)
            .service(Files::new("/config", "./src/config"))
    })
    .bind(("127.0.0.1", port.unwrap_or(8080)))?
    .run()
    .await
    .context("Server error")
}

#[post("/v1/chat/completions")]
async fn completion(
    req: web::Json<CreateChatCompletionRequest>,
) -> Result<impl Responder, LlmError> {
    let params = CompletionParams {
        temp: req.temperature.unwrap_or(0.8),
        max_tokens: req.max_tokens.unwrap_or(256),
    };

    let mut conv = Conversation { messages: vec![] };
    for msg in req.messages.iter() {
        conv.add_message(Message {
            role: match msg.role {
                openai_types::Role::System => Role::System,
                openai_types::Role::User => Role::User,
                openai_types::Role::Assistant => Role::Assistant,
                openai_types::Role::Function => {
                    return Err(LlmError::InvalidInput(
                        "Function role is not supported".to_string(),
                    ))
                }
            },
            content: msg.content.clone().unwrap_or_default(),
        })
    }
    let model = req.model.clone();
    let client = get_client(&model)?;

    let end_event = sse::Event::Data(
        sse::Data::new_json(new_stop_chunk(model.clone())).context("serialization error")?,
    );
    let stream1 = client
        .completion_stream(conv, params)
        .await?
        .map(move |resp| {
            let content = match resp {
                Ok(resp) => resp.content,
                Err(err) => {
                    error!("Error in stream: {}", err);
                    let event = sse::Event::Data(sse::Data::new(err.to_string()).event("error"));
                    return Ok(event);
                }
            };
            let chunk = new_chunk(content, model.clone());
            Ok::<_, LlmError>(sse::Event::Data(
                sse::Data::new_json(chunk).context("serialization error")?,
            ))
        });

    let stream2 = stream::iter(vec![Ok(end_event)]);
    let stream = stream1.chain(stream2);

    Ok(sse::Sse::from_stream(stream).with_keep_alive(Duration::from_secs(5)))
}

fn new_chunk(content: String, model: String) -> CreateChatCompletionStreamResponse {
    CreateChatCompletionStreamResponse {
        id: Uuid::new_v4().to_string(),
        object: "chat.completion.chunk".to_string(),
        created: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap(),
        model: model,
        choices: vec![openai_types::ChatCompletionResponseStreamMessage {
            index: 0,
            delta: ChatCompletionStreamResponseDelta {
                role: Some(openai_types::Role::Assistant),
                content: Some(content.to_string()),
                function_call: None,
            },
            finish_reason: None,
        }],
    }
}

fn new_stop_chunk(model: String) -> CreateChatCompletionStreamResponse {
    CreateChatCompletionStreamResponse {
        id: Uuid::new_v4().to_string(),
        object: "chat.completion.chunk".to_string(),
        created: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap(),
        model: model,
        choices: vec![openai_types::ChatCompletionResponseStreamMessage {
            index: 0,
            delta: ChatCompletionStreamResponseDelta {
                role: None,
                content: None,
                function_call: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
    }
}
