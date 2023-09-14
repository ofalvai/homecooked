use std::time::{Duration, SystemTime};

use actix_web::Responder;
use actix_web_lab::sse;
use anyhow::Context;
use futures::{stream, StreamExt};
use llm_toolkit::{
    conversation::{Conversation, Message},
    provider::{CompletionParams, CompletionResponseStream},
};
use log::error;
use uuid::Uuid;

use super::{
    errors::LlmError,
    openai_types::{
        ChatCompletionResponseStreamMessage, ChatCompletionStreamResponseDelta,
        CreateChatCompletionRequest, CreateChatCompletionStreamResponse, Role,
    },
};

pub fn completion_params(req: &CreateChatCompletionRequest) -> CompletionParams {
    CompletionParams {
        temp: req.temperature.unwrap_or(0.8),
        max_tokens: req.max_tokens.unwrap_or(256),
    }
}

pub fn conversation(req: &CreateChatCompletionRequest) -> Result<Conversation, LlmError> {
    let mut conv = Conversation { messages: vec![] };
    for msg in req.messages.iter() {
        conv.add_message(Message {
            role: match msg.role {
                Role::System => llm_toolkit::conversation::Role::System,
                Role::User => llm_toolkit::conversation::Role::User,
                Role::Assistant => llm_toolkit::conversation::Role::Assistant,
                Role::Function => {
                    return Err(LlmError::InvalidInput(
                        "Function role is not supported".to_string(),
                    ))
                }
            },
            content: msg.content.clone().unwrap_or_default(),
        })
    }
    Ok(conv)
}

pub fn adapt_stream(stream: CompletionResponseStream, model: String) -> impl Responder {
    let end_event = sse::Event::Data(sse::Data::new_json(new_stop_chunk(&model)).unwrap());
    let stream = stream.map(move |resp| {
        let content = match resp {
            Ok(resp) => resp.content,
            Err(err) => {
                error!("Error in stream: {}", err);
                let event = sse::Event::Data(sse::Data::new(err.to_string()).event("error"));
                return Ok(event);
            }
        };
        let chunk = new_chunk(content, &model);
        Ok::<_, LlmError>(sse::Event::Data(
            sse::Data::new_json(chunk).context("serialization error")?,
        ))
    });
    
    let stream2 = stream::iter(vec![Ok(end_event)]);
    let stream = stream.chain(stream2);

    sse::Sse::from_stream(stream).with_keep_alive(Duration::from_secs(5))
}

fn new_chunk(content: String, model: &str) -> CreateChatCompletionStreamResponse {
    CreateChatCompletionStreamResponse {
        id: Uuid::new_v4().to_string(),
        object: "chat.completion.chunk".to_string(),
        created: SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap(),
        model: model.to_string(),
        choices: vec![ChatCompletionResponseStreamMessage {
            index: 0,
            delta: ChatCompletionStreamResponseDelta {
                role: Some(Role::Assistant),
                content: Some(content.to_string()),
                function_call: None,
            },
            finish_reason: None,
        }],
    }
}

fn new_stop_chunk(model: &str) -> CreateChatCompletionStreamResponse {
    CreateChatCompletionStreamResponse {
        id: Uuid::new_v4().to_string(),
        object: "chat.completion.chunk".to_string(),
        created: SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap(),
        model: model.to_string(),
        choices: vec![ChatCompletionResponseStreamMessage {
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
