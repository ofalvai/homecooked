use crate::{models::get_client, output::stream_to_stdout, config::Config};

use anyhow::Context;
use llm_toolkit::{
    conversation::Conversation,
    document::loader::web_article::WebArticleLoader,
    provider::CompletionParams,
    template::{render_prompt, TemplateContext},
};

const DEFAULT_PROMPT: &str = r#"Summarize an article extracted from a web page. Your response should match the language of the article.
Article content:
{input}
"#;

const DEFAULT_MODEL: &str = "claude-2";

pub async fn prompt(
    config: Config,
    input: String,
    prompt: Option<String>,
    model: Option<String>,
) -> anyhow::Result<()> {
    let loader = WebArticleLoader {};
    let html = loader.load(&input).await.unwrap();

    let model = model.unwrap_or(DEFAULT_MODEL.to_string());
    let client = get_client(model.as_str(), &config)?;

    let user_prompt = create_prompt(html, prompt)?;

    let conv = Conversation::new(user_prompt);
    let params = CompletionParams {
        max_tokens: 500,
        temp: 0.6,
    };
    let stream = client.completion_stream(conv, params).await?;
    stream_to_stdout(stream).await?;

    Ok(())
}

fn create_prompt(html: String, prompt: Option<String>) -> anyhow::Result<String> {
    let prompt = prompt.unwrap_or(DEFAULT_PROMPT.to_string());
    let ctx = TemplateContext { input: html };
    render_prompt(&prompt, &ctx).context("prompt error")
}
