use std::{
    io::{self, Write},
    sync::Mutex,
    vec,
};

use async_trait::async_trait;
use lazy_static::lazy_static;
use rs_llama_cpp::{gpt_params_c, run_inference, str_to_mut_i8};

use crate::conversation::{Conversation, Message};

use super::{
    Client, CompletionError, CompletionParams, CompletionResponse, CompletionResponseStream,
};

lazy_static! {
    // Ugly hack because `run_completion` expects a closure callback, not Fn
    static ref COMPLETION_TOKENS: Mutex<Vec<String>> = Mutex::new(vec![]);
}

pub enum Model {
    Todo,
}

impl From<Model> for String {
    fn from(model: Model) -> Self {
        match model {
            Model::Todo => "todo".to_string(),
        }
    }
}

pub struct LlamaConfig {
    pub model_path: String,
    pub n_threads: u16,
    pub mlock: bool,
    pub n_gpu_layers: u16,
}

pub struct LlamaClient {
    config: LlamaConfig,
}

impl LlamaClient {
    pub fn with_config(config: LlamaConfig) -> LlamaClient {
        LlamaClient { config }
    }
}

#[async_trait]
impl Client for LlamaClient {
    async fn completion(
        &self,
        conversation: Conversation,
        params: CompletionParams,
    ) -> Result<CompletionResponse, CompletionError> {
        COMPLETION_TOKENS.lock().unwrap().clear();
        let params = self.make_gpt_params_c(conversation.messages, params);
        run_inference(params, |token| {
            if token.ends_with("\"") {
                print!("{}", token.replace("\"", ""));
                io::stdout().flush().unwrap();

                return true; // stop inference
            }

            print!("{}", token);
            io::stdout().flush().unwrap();
            COMPLETION_TOKENS.lock().unwrap().push(token.to_string());

            return true; // continue inference
        });
        let completion = COMPLETION_TOKENS.lock().unwrap().join("");
        Ok(CompletionResponse {
            id: "".to_string(),
            content: completion,
        })
    }

    async fn completion_stream(
        &self,
        _conversation: Conversation,
        _params: CompletionParams,
    ) -> Result<CompletionResponseStream, CompletionError> {
        todo!(); // need to figure out token_callback closure variable capturing
    }
}

impl LlamaClient {
    fn make_gpt_params_c(&self, messages: Vec<Message>, params: CompletionParams) -> gpt_params_c {
        // TODO: better prompt template
        let prompt = messages.first().unwrap().content.as_ref();

        gpt_params_c {
            n_threads: self.config.n_threads.into(),
            n_gpu_layers: self.config.n_gpu_layers.into(),
            use_mlock: self.config.mlock,

            model: str_to_mut_i8(&self.config.model_path),
            n_ctx: params.max_tokens.into(),
            temp: params.temp,

            input_prefix: str_to_mut_i8(""), // TODO
            input_suffix: str_to_mut_i8(""), // TODO
            prompt: str_to_mut_i8(prompt),

            ..Default::default()
        }
    }
}
