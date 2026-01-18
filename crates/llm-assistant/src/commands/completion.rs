use crate::config::Config;
use crate::templates::read_template;
use crate::{models::get_client, output::print_completion_stream};

use anyhow::Context;
use llm_toolkit::provider::CompletionParams;
use llm_toolkit::{
    conversation::Conversation,
    template::{render_prompt, TemplateContext},
};
use owo_colors::OwoColorize;

pub async fn completion(
    config: Config,
    user_prompt: String,
    template: Option<String>,
    model: Option<String>,
) -> anyhow::Result<()> {
    let params = CompletionParams::default();
    let mut user_prompt = user_prompt;
    let mut model = model.unwrap_or("gpt-3.5-turbo".to_string());

    if let Some(template_id) = template {
        let template =
            read_template(&config.template_file, template_id).context("Cannot read template")?;
        let ctx = TemplateContext { input: user_prompt };
        user_prompt = render_prompt(&template.prompt, &ctx).context("Cannot render prompt")?;

        if let Some(model_id) = template.model {
            model = model_id;
        }

        println!("Model: {}", model.cyan());
        println!("Prompt: {}", user_prompt.dimmed());
    };

    let conv = Conversation::new(user_prompt);
    let client = get_client(model.as_str(), &config)?;

    let stream = client.completion_stream(conv, params).await?;
    println!();
    print_completion_stream(stream).await?;

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
