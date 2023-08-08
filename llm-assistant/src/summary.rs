use crate::{
    document::loader::web_article::WebArticleLoader,
    output::stream_to_stdout,
    prompt,
    provider::{
        openai::{CompletionArgs, Model, OpenAIClient, OpenAIConfig},
        Client,
    },
    template::{render_prompt, TemplateContext},
};

const DEFAULT_PROMPT: &str = r#"
Summarize an article extracted from a web page. Your response should match the language of the article.
Article content:
{input}
"#;

pub async fn summarize(input: String, prompt: Option<String>) -> anyhow::Result<()> {
    let loader = WebArticleLoader {};
    let html = loader.load(&input).await.unwrap();

    let config = OpenAIConfig::default();
    let client = OpenAIClient::with_config(config);

    let user_prompt = create_prompt(html, prompt)?;

    let conv = prompt::with_user(user_prompt);
    let args = CompletionArgs {
        model: Model::Gpt35Turbo16K,
        max_tokens: 500,
    };
    let stream = client.completion_stream(conv.messages, args).await?;
    stream_to_stdout(stream).await?;

    Ok(())
}

fn create_prompt(html: String, prompt: Option<String>) -> anyhow::Result<String> {
    let prompt = prompt.unwrap_or(DEFAULT_PROMPT.to_string());
    let ctx = TemplateContext { input: html };
    render_prompt(&prompt, &ctx)
}
