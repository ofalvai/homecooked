use crate::{config::Config, output::print_tool_stream, tools};

pub async fn ask(
    config: Config,
    url: String,
    prompt: Option<String>,
    model: Option<String>,
) -> anyhow::Result<()> {
    let stream = tools::youtube::run(config, url, prompt, model);
    print_tool_stream(stream).await?;
    Ok(())
}
