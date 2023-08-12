use anyhow::Context;
use llm_toolkit::{
    document::loader::readwise::{Location, ReadwiseClient},
    prompt,
    provider::{
        openai::{CompletionArgs, Model, OpenAIClient, OpenAIConfig},
        Client,
    },
    template::render,
};
use owo_colors::OwoColorize;
use serde::Serialize;
use url::Url;

use crate::output::stream_to_stdout;

const PROMPT: &str = r#"You are given a list of articles from a reading list.
Your task is to choose 3 articles to recommend reading based on a question from me.
If an article is irrelevant to the question, feel free to respond with less than 3 articles or no articles at all.
Do not make up facts about the article contents, accuracy is more important than recommending an article at all costs.
Think step by step why you chose the article. Respond with 3 fields:
1. Title
2. Your reasoning
3. URL

Articles:
{ context }

Question from me: { question }
"#;

const DOCUMENT_CONTEXT: &str = r#"{{ if title }}Title: { title }{{ endif }}
{{ if summary }}Summary: { summary }{{ endif }}
Author: { author }
Source: { source }
Word count: { word_count }
URL: { url }
"#;

#[derive(Serialize)]
struct ReadwisePromptContext {
    question: String,
    context: String,
}

#[derive(Serialize, Clone)]
struct DocumentContext {
    title: Option<String>,
    summary: Option<String>,
    author: String,
    source: String,
    url: String,
    word_count: Option<u32>,
}

pub async fn ask(question: String) -> anyhow::Result<()> {
    let token = std::env::var("READWISE_TOKEN").context("Missing READWISE_TOKEN")?;
    let client = ReadwiseClient::new(token);
    let documents = client.fetch_documents(None, Some(Location::New)).await?;

    let context: String = documents
        .into_iter()
        .map(|d| {
            let url = Url::parse(&d.url);
            let domain = match &url {
                Ok(url) => url.domain().unwrap_or_default(),
                Err(_) => "",
            };
            DocumentContext {
                title: d.title,
                summary: d.summary,
                author: d.author,
                url: d.url,
                source: domain.into(),
                word_count: d.word_count,
            }
        })
        .map(
            |ctx| match render(DOCUMENT_CONTEXT, ctx.clone(), "document") {
                Ok(s) => s,
                Err(e) => {
                    println!(
                        "{}",
                        format!("Invalid document: {}: {}", ctx.title.unwrap_or_default(), e)
                            .yellow()
                    );
                    "".to_string()
                }
            },
        )
        .collect::<Vec<String>>()
        .join("\n");

    let ctx = ReadwisePromptContext {
        question,
        context: context,
    };

    let prompt = render(PROMPT, ctx, "readwise")?;
    println!("{}", prompt.dimmed());

    let conv = prompt::with_user(prompt);
    let config = OpenAIConfig::default();
    let client = OpenAIClient::with_config(config);
    let args = CompletionArgs {
        model: Model::Gpt35Turbo16K,
        max_tokens: 300,
    };

    let stream = client.completion_stream(conv.messages, args).await?;
    println!();
    stream_to_stdout(stream).await?;

    Ok(())
}
