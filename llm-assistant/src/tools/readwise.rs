use llm_toolkit::{
    conversation::Conversation,
    document::loader::readwise::{Document, Location, ReadwiseClient},
    provider::{
        anthropic::{AnthropicClient, AnthropicConfig, Model},
        Client, CompletionParams, CompletionResponse,
    },
    template::{render, render_prompt, TemplateContext},
};
use owo_colors::OwoColorize;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;
use url::Url;

use crate::{
    config::Config,
    tools::{IntermediateOutput, WorkingEvent},
};

use super::{ErrorEvent, ToolEventStream, ToolUseEvent, ToolUseMetadata};

const DESCRIPTION_PROMPT: &str = r#"What are the common characteristics of online articles for the following criteria:
```
{input}
```

Focus on the content only.
Respond with 5 examples where each item is a short and concise description.
You do not need to look up articles.
Do not provide any commentary, just the 5 items.
"#;

const FINAL_PROMPT: &str = r#"Your task is choosing articles to recommend reading.
You are given a list of articles from a reading list. You are also given a description of what I'm looking for in the recommendations.
Think step by step why one article matches the criteria.
If an article is irrelevant to the question, feel free to respond with less than 3 articles or no articles at all.
Do not make up facts about the article contents, accuracy is more important than recommending an article at all costs.
Limit your response to 3 choices.
For each choice, the output format should be:
1. Title
2. Author
3. Relevance
4. Article URL

Use Markdown formatting for highlighting certain parts of your response.

Description of what I'm looking for:
{ description }

Articles:
{ articles }
"#;

const DOCUMENT_CONTEXT: &str = r#"{{ if title }}Title: { title }{{ endif }}
{{ if summary }}Summary: { summary }{{ endif }}
Author: { author }
Source: { source }
Word count: { word_count }
URL: { url }
"#;

#[derive(Serialize)]
struct FinalPromptContext {
    description: String,
    articles: String,
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

pub fn run(config: Config, query: String) -> ToolEventStream {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        match run_inner(&tx, config, query).await {
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
    query: String,
) -> anyhow::Result<()> {
    tx.send(ToolUseEvent::Working(WorkingEvent {
        label: "Thinking about the request".to_string(),
    }))?;
    let description = create_description(&config, query).await?;
    tx.send(ToolUseEvent::IntermediateOutput(IntermediateOutput {
        label: "Common themes".to_string(),
        content: description.clone(),
    }))?;

    tx.send(ToolUseEvent::Working(WorkingEvent {
        label: "Fetching Readwise documents".to_string(),
    }))?;
    let client = ReadwiseClient::new(config.readwise_api_key.clone());
    let documents = client.fetch_documents(None, Some(Location::New)).await?;
    let document_ctx = create_document_context(documents)?;
    tx.send(ToolUseEvent::IntermediateOutput(IntermediateOutput {
        label: "Readwise documents".to_string(),
        content: document_ctx.clone(),
    }))?;

    tx.send(ToolUseEvent::Working(WorkingEvent {
        label: "Generating final answer".to_string(),
    }))?;
    let ctx = FinalPromptContext {
        description: description,
        articles: document_ctx,
    };

    let final_prompt = render(FINAL_PROMPT, ctx, "readwise")?;
    let final_resp = create_final_response(&config, final_prompt).await?;
    tx.send(ToolUseEvent::Output(super::OutputEvent {
        content: final_resp.content,
    }))?;
    Ok(())
}

async fn create_description(config: &Config, question: String) -> anyhow::Result<String> {
    let config = AnthropicConfig {
        model: Model::ClaudeInstant1,
        api_key: config.anthropic_api_key.clone(),
        ..Default::default()
    };
    let client = AnthropicClient::with_config(config);
    let params = CompletionParams {
        max_tokens: 200,
        temp: 1.0,
    };
    let prompt = render_prompt(DESCRIPTION_PROMPT, &TemplateContext { input: question })?;
    let conv = Conversation::new(prompt);
    let resp = client.completion(conv, params).await?;
    Ok(resp.content)
}

fn create_document_context(documents: Vec<Document>) -> anyhow::Result<String> {
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
    Ok(context)
}

async fn create_final_response(
    config: &Config,
    prompt: String,
) -> anyhow::Result<CompletionResponse> {
    let conv = Conversation::new(prompt);

    let config = AnthropicConfig {
        model: Model::Claude2,
        api_key: config.anthropic_api_key.clone(),
        ..Default::default()
    };
    let client = AnthropicClient::with_config(config);
    let params = CompletionParams {
        max_tokens: 500,
        temp: 0.5,
    };

    let resp = client.completion(conv, params).await?;
    Ok(resp)
}
