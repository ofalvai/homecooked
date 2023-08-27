use anyhow::Context;
use llm_toolkit::{
    conversation::Conversation,
    document::loader::youtube::fetch_transcript,
    provider::CompletionParams,
    template::{render_prompt, TemplateContext},
};
use owo_colors::OwoColorize;

use crate::{models::get_client, output::stream_to_stdout};

const DEFAULT_PROMPT: &str = r#"Summarize a video based on a transcript.
Your response should match the spoken language of the video, but your response style should not mimic the speakers.
Video transcript:
>>>
{input}
>>>
"#;

const DEFAULT_MODEL: &str = "claude-instant-1";

pub async fn ask(url: String, prompt: Option<String>, model: Option<String>) -> anyhow::Result<()> {
    let transcript = fetch_transcript(url)
        .await
        .context("Youtube transcript fetch error")?;

    let prompt = prompt.unwrap_or(DEFAULT_PROMPT.to_string());
    let ctx = TemplateContext {
        input: transcript.text,
    };
    let rendered_prompt = render_prompt(&prompt, &ctx).context("prompt error")?;
    println!("{}", rendered_prompt.dimmed());

    let model = model.unwrap_or(DEFAULT_MODEL.to_string());
    let client = get_client(model.as_str())?;
    let conversation = Conversation::new(rendered_prompt);
    let params = CompletionParams {
        max_tokens: 500,
        temp: 0.2,
    };
    let stream = client.completion_stream(conversation, params).await?;
    stream_to_stdout(stream).await?;

    Ok(())
}
