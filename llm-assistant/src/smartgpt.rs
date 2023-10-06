use anyhow::{Context, Ok};
use llm_toolkit::{
    conversation::Conversation,
    provider::{Client, CompletionParams},
    template::{render, render_prompt, TemplateContext},
};
use owo_colors::OwoColorize;
use serde::Serialize;

use crate::{config::Config, models::get_client};

const DEFAULT_MODEL: &str = "claude-instant-1";
const N_OPTION: usize = 3;

const PROMPT_OPTION: &str = r#"Question:
{input}
Answer: Let's work this out in a step by step way to be sure we have the right answer:
"#;

const PROMPT_REFLECTION: &str = r#"You are a researcher tasked with investigating the {n_option} response options provided to a question.
List the flaws and faulty logic of each answer option.
Let's work this out in a step by step way to be sure we have all the errors.
Original question:
>>>
{prompt}
>>>

Answer options:
>>>
{options}
>>>
"#;

const PROMPT_RESOLVER: &str = r#"You are given question and { n_option } answer options, as well as a critique of the answers.
You are a resolver tasked with:
1) finding which of the { n_option } answer options the researcher thought was best
2) improving that answer
3) printing the improved answer in full.
Let's work this out in a step by step way to be sure we have the right answer.

### Original question:
>>>
{ prompt }
>>>

### Answer options:
{{ for option in options }}
#### Option { option.0 }:
>>>
{ option.1 }
>>>
{{ endfor }}

### Researcher's critique of each option:
>>>
{ critique }
>>>
"#;

#[derive(Serialize)]
struct ReflectionContext {
    n_option: usize,
    prompt: String,
    options: String,
}

#[derive(Serialize)]
struct ResolverContext {
    n_option: usize,
    prompt: String,

    // index-option pairs
    options: Vec<(usize, String)>,
    critique: String,
}

pub async fn prompt(config: Config, prompt: String, model: Option<String>) -> anyhow::Result<()> {
    let model = model.unwrap_or(DEFAULT_MODEL.to_string());
    let client = get_client(&model, &config)?;

    let mut options = Vec::<String>::new();
    for i in 0..N_OPTION {
        let option = generate_option(&client, prompt.clone()).await?;
        println!("{}", format!("Option {}:", i + 1).green());
        println!("{}", option.yellow());
        options.push(option);
    }

    let reflection = generate_reflection(&client, prompt.clone(), options.clone()).await?;
    println!("\n\n{}", "Reflection:".green());
    println!("{}", reflection.yellow());

    let resolver = generate_resolver_response(&client, prompt, options.clone(), reflection).await?;
    println!("\n\n{}", "Final answer:".green());
    println!("{}", resolver.yellow());

    Ok(())
}

async fn generate_option(client: &Box<dyn Client>, prompt: String) -> anyhow::Result<String> {
    let ctx = TemplateContext { input: prompt };
    let rendered_prompt = render_prompt(&PROMPT_OPTION, &ctx).context("prompt error")?;
    println!("{}", rendered_prompt.dimmed());
    let conversation = Conversation::new(rendered_prompt);
    let params = CompletionParams {
        max_tokens: 1000,
        temp: 0.8,
    };
    let response = client
        .completion(conversation, params)
        .await
        .context("completion error")?;
    Ok(response.content)
}

async fn generate_reflection(
    client: &Box<dyn Client>,
    prompt: String,
    options: Vec<String>,
) -> anyhow::Result<String> {
    let ctx = ReflectionContext {
        n_option: options.len(),
        prompt: prompt,
        options: options.join("\n---ANSWER OPTION SEPARATOR---\n"), // TODO
    };
    let rendered_prompt = render(PROMPT_REFLECTION, ctx, "reflection").context("prompt error")?;
    println!("{}", rendered_prompt.dimmed());
    let conversation = Conversation::new(rendered_prompt);
    let params = CompletionParams {
        max_tokens: 1500,
        temp: 0.2,
    };
    let response = client
        .completion(conversation, params)
        .await
        .context("completion error")?;

    Ok(response.content)
}

async fn generate_resolver_response(
    client: &Box<dyn Client>,
    prompt: String,
    options: Vec<String>,
    critique: String,
) -> anyhow::Result<String> {
    let options: Vec<(usize, String)> = options
        .into_iter()
        .enumerate()
        .map(|(i, option)| (i + 1, option))
        .collect();
    let ctx = ResolverContext {
        n_option: options.len(),
        prompt: prompt,
        options: options,
        critique: critique,
    };
    let rendered_prompt = render(PROMPT_RESOLVER, ctx, "resolver").context("prompt error")?;
    println!("{}", rendered_prompt.dimmed());
    let conversation = Conversation::new(rendered_prompt);
    let params = CompletionParams {
        max_tokens: 800,
        temp: 0.2,
    };
    let response = client
        .completion(conversation, params)
        .await
        .context("completion error")?;

    Ok(response.content)
}
