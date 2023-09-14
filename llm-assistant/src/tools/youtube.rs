use std::io::Write;

use anyhow::{Context, anyhow};
use llm_toolkit::{
    document::loader::youtube::fetch_transcript,
    provider::{CompletionResponseStream, CompletionParams},
    template::{render_prompt, TemplateContext}, conversation::Conversation,
};
use owo_colors::OwoColorize;

use crate::models::get_client;

const DEFAULT_PROMPT: &str = r#"Summarize a video based on a transcript.
Your response should match the spoken language of the video, but your response style should not mimic the speakers.
Video transcript:
>>>
{input}
>>>
"#;

pub const DEFAULT_MODEL: &str = "claude-instant-1";

pub async fn run(
    url: String,
    prompt: Option<String>,
    model: Option<&str>,
    mut msg_writer: impl Write,
) -> anyhow::Result<CompletionResponseStream> {
    let transcript = fetch_transcript(url)
        .await
        .context("Youtube transcript fetch error")?;

    let prompt = prompt.unwrap_or(DEFAULT_PROMPT.to_string());
    let ctx = TemplateContext {
        input: transcript.text,
    };
    let rendered_prompt = render_prompt(&prompt, &ctx).context("prompt error")?;
    write!(msg_writer, "{}", rendered_prompt.dimmed())?;

    let client = get_client(model.unwrap_or(DEFAULT_MODEL))?;
    let conversation = Conversation::new(rendered_prompt);
    let params = CompletionParams {
        max_tokens: 500,
        temp: 0.2,
    };
    let stream = client.completion_stream(conversation, params).await;
    match stream {
        Ok(stream) => Ok(stream),
        Err(err) => Err(anyhow!("request error: {}", err)),
    }
}
