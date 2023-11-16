use anyhow::Context;
use llm_toolkit::{
    conversation::Conversation,
    document::loader::web_article::WebArticleLoader,
    provider::CompletionParams,
    template::{render_prompt, TemplateContext},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config::Config,
    models::get_client,
    tools::{ErrorEvent, IntermediateOutput, ToolUseMetadata, WorkingEvent},
};

use super::{ToolEventStream, ToolUseEvent};

const DEFAULT_PROMPT: &str = r#"Summarize an article extracted from a web page. Your response should match the language of the article.
Article content:
{input}
"#;

const DEFAULT_MODEL: &str = "claude-instant-1";

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
        label: "Loading webpage".to_string(),
    }))?;

    let loader = WebArticleLoader {};
    let html = loader.load(&url).await?;

    let model = model.unwrap_or(DEFAULT_MODEL.to_string());
    let client = get_client(model.as_str(), &config)?;

    let user_prompt = create_prompt(html, prompt)?;
    tx.send(ToolUseEvent::IntermediateOutput(IntermediateOutput {
        label: "Prompt".to_string(),
        content: user_prompt.clone(),
    }))?;

    tx.send(ToolUseEvent::Working(WorkingEvent {
        label: "Generating final answer".to_string(),
    }))?;
    let conv = Conversation::new(user_prompt);
    let params = CompletionParams {
        max_tokens: 500,
        temp: 0.6,
    };
    let resp = client
        .completion(conv, params)
        .await
        .context("completion error")?;

    tx.send(ToolUseEvent::Output(super::OutputEvent {
        content: resp.content,
    }))?;
    Ok(())
}

fn create_prompt(html: String, prompt: Option<String>) -> anyhow::Result<String> {
    let prompt = prompt.unwrap_or(DEFAULT_PROMPT.to_string());
    let ctx = TemplateContext { input: html };
    render_prompt(&prompt, &ctx).context("prompt error")
}
