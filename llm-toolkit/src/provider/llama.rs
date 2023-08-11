use std::{
    io::{self, Write},
    sync::Mutex,
    vec,
};

use async_trait::async_trait;
use lazy_static::lazy_static;
use rs_llama_cpp::{gpt_params_c, run_inference, str_to_mut_i8};

use crate::prompt::Message;

use super::{Client, CompletionError, CompletionResponse, CompletionResponseStream};

lazy_static! {
    // Ugly hack because `run_completion` expects a closure callback, not Fn
    static ref COMPLETION_TOKENS: Mutex<Vec<String>> = Mutex::new(vec![]);
}

pub struct CompletionArgs {
    pub temp: f32,
    pub context_length: u16,
}

impl Default for CompletionArgs {
    fn default() -> Self {
        Self {
            temp: 0.8,
            context_length: 1500,
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
    type CompletionArgs = CompletionArgs;

    async fn completion(
        &self,
        messages: Vec<Message>,
        args: CompletionArgs,
    ) -> Result<CompletionResponse, CompletionError> {
        COMPLETION_TOKENS.lock().unwrap().clear();
        let params = self.make_gpt_params_c(messages, args);
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
        _messages: Vec<Message>,
        _args: CompletionArgs,
    ) -> Result<CompletionResponseStream, CompletionError> {
        todo!(); // need to figure out token_callback closure variable capturing
    }
}

impl LlamaClient {
    fn make_gpt_params_c(&self, messages: Vec<Message>, args: CompletionArgs) -> gpt_params_c {
        // TODO: better prompt template
        let prompt = messages.first().unwrap().content.as_ref();

        gpt_params_c {
            n_threads: self.config.n_threads.into(),
            n_ctx: args.context_length.into(),
            n_gpu_layers: self.config.n_gpu_layers.into(),
            use_mlock: self.config.mlock,

            model: str_to_mut_i8(&self.config.model_path),

            input_prefix: str_to_mut_i8(""), // TODO
            input_suffix: str_to_mut_i8(""), // TODO
            prompt: str_to_mut_i8(prompt),

            ..Default::default()
        }
    }
}
