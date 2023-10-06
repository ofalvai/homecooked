use crate::{output::stream_to_stdout, tools, config::Config};

pub async fn ask(config: Config, url: String, prompt: Option<String>, model: Option<String>) -> anyhow::Result<()> {
    let stream = tools::youtube::run(&config, url, prompt, model.as_deref(), std::io::stdout()).await?;

    stream_to_stdout(stream).await?;

    Ok(())
}
