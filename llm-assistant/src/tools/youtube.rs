use anyhow::Context;
use llm_toolkit::{
    conversation::Conversation,
    document::loader::youtube::fetch_transcript,
    provider::CompletionParams,
    template::{render_prompt, TemplateContext},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{config::Config, models::get_client};

use super::{
    ErrorEvent, IntermediateOutput, ToolEventStream, ToolUseEvent, ToolUseMetadata, WorkingEvent,
};

const DEFAULT_PROMPT: &str = r#"Summarize a video based on a transcript.
Your response should match the spoken language of the video, but your response style should not mimic the speakers.
Video transcript:
>>>
{input}
>>>
"#;

pub const DEFAULT_MODEL: &str = "claude-instant-1";

pub fn run(
    config: Config,
    url: String,
    prompt: Option<String>,
    model: Option<String>,
) -> ToolEventStream {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        match run_inner(&tx, config, url, prompt, model).await {
            Ok(_) => {
                tx.send(ToolUseEvent::Finished(ToolUseMetadata {})).unwrap();
            }
            Err(e) => {
                tx.send(ToolUseEvent::Error(ErrorEvent {
                    label: "Failed to run tool".to_string(),
                    error: Some(e.to_string()),
                }))
                .unwrap();
            }
        }
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}

async fn run_inner(
    tx: &UnboundedSender<ToolUseEvent>,
    config: Config,
    url: String,
    prompt: Option<String>,
    model: Option<String>,
) -> anyhow::Result<()> {
    tx.send(ToolUseEvent::Working(WorkingEvent {
        label: "Fetching video transcript...".to_string(),
    }))?;

    let transcript = fetch_transcript(url)
        .await
        .context("fetch video transcript")?;

    let prompt = prompt.unwrap_or(DEFAULT_PROMPT.to_string());
    let ctx = TemplateContext {
        input: transcript.text,
    };
    let rendered_prompt = render_prompt(&prompt, &ctx).context("prompt error")?;
    tx.send(ToolUseEvent::IntermediateOutput(IntermediateOutput {
        label: "Prompt".to_string(),
        content: rendered_prompt.clone(),
    }))?;

    tx.send(ToolUseEvent::Working(WorkingEvent {
        label: "Generating final answer...".to_string(),
    }))?;
    let conversation = Conversation::new(rendered_prompt);
    let params = CompletionParams {
        max_tokens: 500,
        temp: 0.2,
    };
    let model = model.unwrap_or(DEFAULT_MODEL.to_string());
    let resp = get_client(&model, &config)?
        .completion(conversation, params)
        .await
        .context("completion error")?;
    tx.send(ToolUseEvent::Output(super::OutputEvent {
        content: resp.content,
    }))?;

    Ok(())
}
