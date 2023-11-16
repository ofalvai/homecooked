use crate::{config::Config, output::print_tool_stream, tools};

pub async fn prompt(
    config: Config,
    input: String,
    prompt: Option<String>,
    model: Option<String>,
) -> anyhow::Result<()> {
    let stream = tools::web::run(config, input, prompt, model);
    print_tool_stream(stream).await?;
    Ok(())
}
