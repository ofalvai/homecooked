use crate::output::stream_to_stdout;

use anyhow::Context;
use llm_toolkit::{
    prompt,
    provider::{
        openai::{CompletionArgs, OpenAIClient, OpenAIConfig},
        Client,
    },
    template::{render_prompt, TemplateContext},
};
use owo_colors::OwoColorize;
use serde::Deserialize;

pub async fn completion(user_prompt: String, template: Option<String>) -> anyhow::Result<()> {
    let mut args = CompletionArgs::default();
    let mut user_prompt = user_prompt;

    match template {
        Some(template_name) => {
            let template = read_template(template_name).context("Cannot read template")?;
            let ctx = TemplateContext { input: user_prompt };
            user_prompt = render_prompt(&template.prompt, &ctx).context("Cannot render prompt")?;

            if let Some(model_id) = template.model {
                args.model = model_id.parse()?;
            }

            println!("{}", format!("Model: {}", args.model.cyan()));
            println!("Prompt: {}", user_prompt.dimmed());
        }
        None => {}
    };

    let conv = prompt::with_user(user_prompt.into());
    let config = OpenAIConfig::default();
    let client = OpenAIClient::with_config(config);

    let stream = client.completion_stream(conv.messages, args).await?;
    println!();
    stream_to_stdout(stream).await?;

    // let resp = client.completion(conv.messages, args).await?;
    // println!("{}", resp.content);

    // let config = LlamaConfig {
    //     model_path: "/Users/oliverfalvai/.cache/lm-studio/models/TheBloke/StableBeluga-7B-GGML/stablebeluga-7b.ggmlv3.q4_K_M.bin".to_string(),
    //     n_threads: 6,
    //     mlock: false,
    //     n_gpu_layers: 1,
    // };
    // let client = LlamaClient::with_config(config);
    // let args = CompletionArgs::default();

    // let response = client.completion(conv.messages, args).await?;
    // println!("response: {:?}", response);

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Template {
    prompt: String,
    model: Option<String>,
}

const TEMPLATE_FOLDER: &str = "templates";

fn read_template(name: String) -> anyhow::Result<Template> {
    let path = format!("{}/{}.yml", TEMPLATE_FOLDER, name);
    let template = std::fs::read_to_string(&path)?;
    let template: Template = serde_yaml::from_str(&template)?;
    Ok(template)
}
