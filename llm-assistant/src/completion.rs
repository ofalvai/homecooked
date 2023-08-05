use std::io::{stdout, Write};

use futures::StreamExt;

use crate::{
    prompt,
    provider::{
        openai::{CompletionArgs, OpenAIClient, OpenAIConfig},
        Client,
    },
};

pub async fn completion(user_prompt: &str) -> anyhow::Result<()> {
    let config = OpenAIConfig::default();
    let client = OpenAIClient::with_config(config);

    let conv = prompt::with_user(user_prompt.into());
    let args = CompletionArgs::default();
    let mut stream = client.completion_stream(conv.messages, args).await?;

    let mut lock = stdout().lock();
    let mut full_completion = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(resp) => {
                full_completion.push_str(&resp.content);
                write!(lock, "{}", resp.content).unwrap();
            }
            Err(e) => eprintln!("{}", e),
        }
        stdout().flush()?;
    }
    Ok(())
}
