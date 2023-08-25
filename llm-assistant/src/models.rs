use llm_toolkit::provider::{anthropic, openai, Client};

pub fn get_client(model_name: &str) -> anyhow::Result<Box<dyn Client>> {
    let client: Box<dyn Client> = match model_name {
        "gpt3" | "gpt-3" | "gpt-3.5" | "gpt-3.5-turbo" => {
            let config = openai::OpenAIConfig {
                model: openai::Model::Gpt35Turbo,
                ..Default::default()
            };
            Box::new(openai::OpenAIClient::with_config(config))
        }
        "16k" | "gpt-3.5-turbo-16k" => {
            let config = openai::OpenAIConfig {
                model: openai::Model::Gpt35Turbo16K,
                ..Default::default()
            };
            Box::new(openai::OpenAIClient::with_config(config))
        }
        "gpt4" | "gpt-4" => {
            let config = openai::OpenAIConfig {
                model: openai::Model::Gpt4,
                ..Default::default()
            };
            Box::new(openai::OpenAIClient::with_config(config))
        }
        "claude" | "claude-instant" | "claude-instant-1" => {
            let config = anthropic::AnthropicConfig {
                model: anthropic::Model::ClaudeInstant1,
                ..Default::default()
            };
            Box::new(anthropic::AnthropicClient::with_config(config))
        }
        "claude-2" | "claude2" => {
            let config = anthropic::AnthropicConfig {
                model: anthropic::Model::Claude2,
                ..Default::default()
            };
            Box::new(anthropic::AnthropicClient::with_config(config))
        }
        // "llama" | "llama-cpp" | "llamacpp" | "llama.cpp" => {
        //     let config = llama::LlamaConfig {
        //         // TODO 
        //         model_path: "/Users/oliverfalvai/.cache/lm-studio/models/TheBloke/StableBeluga-7B-GGML/stablebeluga-7b.ggmlv3.q4_K_M.bin".to_string(),
        //         n_threads: 6,
        //         mlock: false,
        //         n_gpu_layers: 1,
        //     };
        //     Box::new(llama::LlamaClient::with_config(config))
        // }
        _ => anyhow::bail!("Unknown model name: {}", model_name),
    };
    Ok(client)
}

pub fn models() -> anyhow::Result<()> {
    Ok(())
}
