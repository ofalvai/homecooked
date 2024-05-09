#![allow(dead_code, unused)]

use std::{fmt::Display, str::FromStr};

use async_openai::{
    error::OpenAIError,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs, CreateChatCompletionStreamResponse,
    },
};
use async_trait::async_trait;
use futures::StreamExt;
use log::{info, warn};

use crate::conversation::{Conversation, Message, Role};

use super::{
    Client, CompletionError, CompletionParams, CompletionResponse, CompletionResponseDelta,
    CompletionResponseStream,
};

#[derive(Debug)]
pub enum Model {
    Gpt35Turbo,
    Gpt4,
    Gpt4Turbo,
    Custom(String),
}

pub struct OpenAIConfig {
    pub api_key: String,
    pub api_base: String,
    pub model: Model,
}

pub struct OpenAIClient {
    config: OpenAIConfig,
}

impl Model {
    fn model_id(&self) -> &str {
        match self {
            Model::Gpt35Turbo => "gpt-3.5-turbo-1106",
            Model::Gpt4 => "gpt-4",
            Model::Gpt4Turbo => "gpt-4-1106-preview",
            Model::Custom(model) => model,
        }
    }
}

impl FromStr for Model {
    type Err = OpenAIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gpt-3.5-turbo" => Ok(Model::Gpt35Turbo),
            "gpt-4-1106-preview" => Ok(Model::Gpt4Turbo),
            "gpt-4" => Ok(Model::Gpt4),
            model => Ok(Model::Custom(model.to_string())),
        }
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model_id())
    }
}

impl From<Model> for String {
    fn from(model: Model) -> Self {
        model.model_id().to_string()
    }
}

impl OpenAIClient {
    pub fn with_config(config: OpenAIConfig) -> OpenAIClient {
        OpenAIClient { config }
    }
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_base: "https://api.openai.com/v1".to_string(),
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            model: Model::Gpt35Turbo,
        }
    }
}

#[async_trait]
impl Client for OpenAIClient {
    async fn completion(
        &self,
        conversation: Conversation,
        params: CompletionParams,
    ) -> Result<CompletionResponse, CompletionError> {
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_base(self.config.api_base.clone())
            .with_api_key(self.config.api_key.clone());
        let client = async_openai::Client::with_config(config);

        let request = completion_request(conversation.messages, &self.config.model, params, false)?;
        let response = client.chat().create(request).await.unwrap(); // only error is a stream arg validation, we take care of that

        let usage = response.usage.ok_or(CompletionError::InvalidResponse(
            "no usage info returned".to_string(),
        ))?;
        info!(
            "Token usage: {} prompt + {} completion = {} total",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );

        let content = response
            .choices
            .first()
            .ok_or(CompletionError::InvalidResponse(
                "no choices returned".to_string(),
            ))?
            .message
            .content
            .clone()
            .ok_or(CompletionError::InvalidResponse(
                "no message content of response".to_string(),
            ))?;

        Ok(CompletionResponse {
            id: response.id,
            content,
        })
    }

    async fn completion_stream(
        &self,
        conversation: Conversation,
        params: CompletionParams,
    ) -> Result<CompletionResponseStream, CompletionError> {
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_base(self.config.api_base.clone())
            .with_api_key(self.config.api_key.clone());
        let client = async_openai::Client::with_config(config);

        let request = completion_request(conversation.messages, &self.config.model, params, true)?;
        let stream = client
            .chat()
            .create_stream(request)
            .await
            .unwrap() // only error is a stream arg validation, we take care of that
            .map(|item| match item {
                Ok(resp) => map_stream_response(resp),
                Err(err) => Err(map_stream_error(err)),
            });

        Ok(Box::pin(stream))
    }
}

fn map_stream_response(
    resp: CreateChatCompletionStreamResponse,
) -> Result<CompletionResponseDelta, CompletionError> {
    // https://community.openai.com/t/usage-info-in-api-responses/18862/3?u=ofalvai
    // if let Some(usage) = resp.usage {
    //     info!(
    //         "Token usage: {} prompt + {} completion = {} total",
    //         usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
    //     );
    // }

    let choice = resp
        .choices
        .first()
        .ok_or(CompletionError::InvalidResponse(
            "no choice in response".to_string(),
        ))?;
    let delta = choice.delta.content.clone().unwrap_or_default();
    Ok(CompletionResponseDelta {
        id: resp.id,
        content: delta,
    })
}

fn map_stream_error(err: OpenAIError) -> CompletionError {
    match err {
        OpenAIError::InvalidArgument(e) => CompletionError::InvalidArgument(e),
        OpenAIError::ApiError(e) => {
            let error_str = format!(
                "{}: {}",
                e.r#type.unwrap_or("unknown".to_string()),
                e.message
            );
            CompletionError::ApiError("OpenAI".to_string(), error_str)
        }
        OpenAIError::StreamError(e) => CompletionError::StreamError(e),
        _ => CompletionError::UnknownError(format!("unknown error: {err}")),
    }
}

fn completion_request(
    messages: Vec<Message>,
    model: &Model,
    params: CompletionParams,
    stream: bool,
) -> Result<CreateChatCompletionRequest, CompletionError> {
    let mapped_messages: Vec<ChatCompletionRequestMessage> = messages
        .into_iter()
        .map(|message| {
            match message.role {
                Role::System => {
                    let args = ChatCompletionRequestSystemMessageArgs::default()
                        .content(message.content)
                        .build()
                        .unwrap(); // TODO: proper error handling
                    ChatCompletionRequestMessage::System(args)
                }
                Role::User => {
                    let args = ChatCompletionRequestUserMessageArgs::default()
                        .content(message.content)
                        .build()
                        .unwrap();
                    ChatCompletionRequestMessage::User(args)
                }
                Role::Assistant => {
                    let args = ChatCompletionRequestAssistantMessageArgs::default()
                        .content(message.content)
                        .build()
                        .unwrap();
                    ChatCompletionRequestMessage::Assistant(args)
                }
            }
        })
        .collect();

    let request = CreateChatCompletionRequestArgs::default()
        .messages(mapped_messages)
        .model(model.to_string())
        .max_tokens(params.max_tokens)
        .temperature(params.temp)
        .n(1)
        .stream(stream)
        .build();

    match request {
        Ok(req) => Ok(req),
        Err(err) => Err(CompletionError::InvalidArgument(format!("{err}"))),
    }
}
