use std::{fmt::Display, str::FromStr};

use anthropic::error::AnthropicError;
use async_trait::async_trait;
use log::info;

use crate::prompt::Message;

use super::{Client, CompletionError, CompletionResponse, CompletionResponseStream};

#[derive(Debug)]
pub enum Model {
    Claude1,
    Claude2,
    ClaudeInstant1,
}

pub struct AnthropicConfig {
    api_key: String,
}

pub struct AnthropicClient {
    config: AnthropicConfig,
}

pub struct CompletionArgs {
    pub model: Model,
    pub max_tokens: u16,
}

impl AnthropicClient {
    pub fn with_config(config: AnthropicConfig) -> Self {
        Self { config }
    }
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
        }
    }
}

impl Default for CompletionArgs {
    fn default() -> Self {
        Self {
            model: Model::ClaudeInstant1,
            max_tokens: 256,
        }
    }
}

impl Model {
    fn model_id(&self) -> &str {
        match self {
            Model::Claude1 => "claude-1",
            Model::Claude2 => "claude-2",
            Model::ClaudeInstant1 => "claude-instant-1",
        }
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model_id())
    }
}

impl FromStr for Model {
    type Err = AnthropicError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "claude-1" => Ok(Model::Claude1),
            "claude-2" => Ok(Model::Claude2),
            "claude-instant-1" => Ok(Model::ClaudeInstant1),
            _ => Err(AnthropicError::InvalidArgument(format!(
                "model {} is not recognized",
                s
            ))),
        }
    }
}

#[async_trait]
impl Client for AnthropicClient {
    type CompletionArgs = CompletionArgs;

    async fn completion(
        &self,
        messages: Vec<Message>,
        args: CompletionArgs,
    ) -> Result<CompletionResponse, CompletionError> {
        let client = match anthropic::client::ClientBuilder::default()
            .api_key(self.config.api_key.clone())
            .build()
        {
            Ok(client) => client,
            Err(err) => return Err(CompletionError::InvalidArgument(err.to_string())),
        };
        let request = match anthropic::types::CompleteRequestBuilder::default()
            .prompt(make_prompt(messages))
            .model(args.model.model_id())
            .stream_response(false)
            .max_tokens_to_sample(args.max_tokens)
            .stop_sequences(vec![anthropic::HUMAN_PROMPT.to_string()]) // https://github.com/abdelhamidbakhta/anthropic-rs/issues/1
            .build()
        {
            Ok(req) => req,
            Err(err) => return Err(CompletionError::InvalidArgument(err.to_string())),
        };

        let result = match client.complete(request).await {
            Ok(res) => res,
            Err(err) => return Err(CompletionError::ApiError(err.to_string())),
        };

        Ok(CompletionResponse {
            id: "".to_string(),
            content: result.completion,
        })
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
        args: CompletionArgs,
    ) -> Result<CompletionResponseStream, CompletionError> {
        todo!()
    }
}

fn make_prompt(messages: Vec<Message>) -> String {
    // TODO: rethink all of this
    let prompt = format!(
        "{}{}{}",
        anthropic::HUMAN_PROMPT,
        messages.first().unwrap().content,
        anthropic::AI_PROMPT
    );
    info!("{prompt}");
    prompt
}
