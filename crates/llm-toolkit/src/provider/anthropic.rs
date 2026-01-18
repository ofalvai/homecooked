use std::{fmt::Display, str::FromStr};

use anthropic::error::AnthropicError;
use async_trait::async_trait;
use futures::{future, StreamExt};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    conversation::{Conversation, Message, Role},
    provider::CompletionResponseDelta,
};

use super::{
    Client, CompletionError, CompletionParams, CompletionResponse, CompletionResponseStream,
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Model {
    Claude1,
    Claude2,
    ClaudeInstant1,
    Custom(String),
}

pub struct AnthropicConfig {
    pub api_key: String,
    pub model: Model,
}

pub struct AnthropicClient {
    config: AnthropicConfig,
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
            model: Model::ClaudeInstant1,
        }
    }
}

impl Model {
    fn model_id(&self) -> &str {
        match self {
            Model::Claude1 => "claude-1",
            Model::Claude2 => "claude-2",
            Model::ClaudeInstant1 => "claude-instant-1",
            Model::Custom(model) => model,
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
            model => Ok(Model::Custom(model.to_string())),
        }
    }
}

impl From<Model> for String {
    fn from(model: Model) -> Self {
        model.model_id().to_string()
    }
}

#[async_trait]
impl Client for AnthropicClient {
    async fn completion(
        &self,
        conversation: Conversation,
        params: CompletionParams,
    ) -> Result<CompletionResponse, CompletionError> {
        let client = match anthropic::client::ClientBuilder::default()
            .api_key(self.config.api_key.clone())
            .build()
        {
            Ok(client) => client,
            Err(err) => return Err(CompletionError::InvalidArgument(err.to_string())),
        };
        let request = match anthropic::types::CompleteRequestBuilder::default()
            .prompt(make_prompt(conversation.messages))
            .model(self.config.model.to_string())
            // TODO: set temp
            .stream(false)
            .max_tokens_to_sample(params.max_tokens)
            .stop_sequences(vec![anthropic::HUMAN_PROMPT.to_string()]) // https://github.com/abdelhamidbakhta/anthropic-rs/issues/1
            .build()
        {
            Ok(req) => req,
            Err(err) => return Err(CompletionError::InvalidArgument(err.to_string())),
        };

        let result = match client.complete(request).await {
            Ok(res) => res,
            Err(err) => {
                return Err(CompletionError::ApiError(
                    "Anthropic".to_string(),
                    err.to_string(),
                ))
            }
        };

        Ok(CompletionResponse {
            id: "".to_string(),
            content: result.completion,
        })
    }

    async fn completion_stream(
        &self,
        conversation: Conversation,
        params: CompletionParams,
    ) -> Result<CompletionResponseStream, CompletionError> {
        let client = match anthropic::client::ClientBuilder::default()
            .api_key(self.config.api_key.clone())
            .build()
        {
            Ok(client) => client,
            Err(err) => return Err(CompletionError::InvalidArgument(err.to_string())),
        };
        let prompt = make_prompt(conversation.messages);
        debug!("prompt: \n{}", prompt);
        let request = match anthropic::types::CompleteRequestBuilder::default()
            .prompt(prompt)
            .model(self.config.model.to_string())
            // TODO: set temp
            .stream(true)
            .max_tokens_to_sample(params.max_tokens)
            .stop_sequences(vec![anthropic::HUMAN_PROMPT.to_string()]) // https://github.com/abdelhamidbakhta/anthropic-rs/issues/1
            .build()
        {
            Ok(req) => req,
            Err(err) => return Err(CompletionError::InvalidArgument(err.to_string())),
        };

        let stream = match client.complete_stream(request).await {
            Ok(s) => s,
            Err(e) => {
                return Err(CompletionError::ApiError(
                    "Anthropic".to_string(),
                    e.to_string(),
                ))
            }
        };

        let mapped_stream = stream
            .take_while(|item| {
                future::ready(match item {
                    Ok(resp) => resp.stop_reason.is_none(),
                    Err(_) => true,
                })
            })
            .map(|item| match item {
                Ok(resp) => Ok(CompletionResponseDelta {
                    // TODO: reconsider ID field
                    id: "no-id".to_string(),
                    content: resp.completion,
                }),
                Err(err) => Err(CompletionError::StreamError(err.to_string())),
            });

        Ok(Box::pin(mapped_stream))
    }
}

fn make_prompt(messages: Vec<Message>) -> String {
    let mut str_chunks: Vec<String> = messages
        .into_iter()
        .map(|m| {
            let prefix = match m.role {
                Role::System => "",
                Role::User => anthropic::HUMAN_PROMPT,
                Role::Assistant => anthropic::AI_PROMPT,
            };
            format!("{}{}", prefix, m.content)
        })
        .collect();

    // String must end with AI prompt
    if !str_chunks.is_empty() {
        str_chunks.push(anthropic::AI_PROMPT.to_string());
    }

    str_chunks.join("") // linebreaks are already included in the prompt variables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_prompt_basic() {
        let messages: Vec<Message> = vec![Message {
            role: Role::User,
            content: "Tell me a joke".to_string(),
        }];
        let expected_output = format!(
            "{}Tell me a joke{}",
            anthropic::HUMAN_PROMPT,
            anthropic::AI_PROMPT,
        );
        assert_eq!(make_prompt(messages), expected_output);
    }

    #[test]
    fn test_make_prompt_empty() {
        // Test case 1: Empty messages vector
        let messages: Vec<Message> = vec![];
        assert_eq!(make_prompt(messages), "");
    }

    #[test]
    fn test_make_prompt_conversation() {
        let messages: Vec<Message> = vec![
            Message {
                role: Role::User,
                content: "Tell me a joke".to_string(),
            },
            Message {
                role: Role::Assistant,
                content: "Here's a silly joke for you:\n\nWhy was the math book sad? Because it had too many problems!".to_string(),
            },
            Message {
                role: Role::User,
                content: "Tell me another".to_string(),
            }
        ];
        let expected_output = format!(
            "{}{}{}{}{}{}{}",
            anthropic::HUMAN_PROMPT,
            "Tell me a joke",
            anthropic::AI_PROMPT,
            "Here's a silly joke for you:\n\nWhy was the math book sad? Because it had too many problems!",
            anthropic::HUMAN_PROMPT,
            "Tell me another",
            anthropic::AI_PROMPT,
        );
        assert_eq!(make_prompt(messages), expected_output);
    }

    #[test]
    fn test_make_system_prompt() {
        let messages: Vec<Message> = vec![
            Message {
                role: Role::System,
                content: "You are a helpful assistant.".to_string(),
            },
            Message {
                role: Role::User,
                content: "Tell me a joke".to_string(),
            },
        ];
        let expected_output = format!(
            "{}{}{}{}",
            "You are a helpful assistant.",
            anthropic::HUMAN_PROMPT,
            "Tell me a joke",
            anthropic::AI_PROMPT,
        );
        assert_eq!(make_prompt(messages), expected_output);
    }
}
