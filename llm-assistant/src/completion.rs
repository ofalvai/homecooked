use crate::{
    output::stream_to_stdout,
    prompt,
    provider::{
        openai::{CompletionArgs, OpenAIClient, OpenAIConfig},
        Client,
    },
};

pub async fn completion(user_prompt: String) -> anyhow::Result<()> {
    let config = OpenAIConfig::default();
    let client = OpenAIClient::with_config(config);

    let conv = prompt::with_user(user_prompt.into());
    let args = CompletionArgs::default();
    let stream = client.completion_stream(conv.messages, args).await?;
    stream_to_stdout(stream).await?;

    Ok(())
}
