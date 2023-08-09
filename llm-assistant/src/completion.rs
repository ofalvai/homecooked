use crate::output::stream_to_stdout;

use llm_toolkit::{
    prompt,
    provider::{
        openai::{CompletionArgs, OpenAIClient, OpenAIConfig},
        Client,
    },
};

pub async fn completion(user_prompt: String) -> anyhow::Result<()> {
    let conv = prompt::with_user(user_prompt.into());
    let config = OpenAIConfig::default();
    let client = OpenAIClient::with_config(config);

    let args = CompletionArgs::default();

    let stream = client.completion_stream(conv.messages, args).await?;
    stream_to_stdout(stream).await?;

    // let config = LlamaConfig {
    //     model_path: "/Users/oliverfalvai/.cache/lm-studio/models/TheBloke/StableBeluga-7B-GGML/stablebeluga-7b.ggmlv3.q4_K_M.bin".to_string(),
    //     n_threads: 6,
    //     mlock: false,
    //     n_gpu_layers: 1,
    // };
    // let client = LlamaClient::with_config(config);
    // let args = CompletionArgs::default();

    // let response = client.completion(conv.messages, args).await?;
    // println!("response: {:?}", response);

    Ok(())
}
