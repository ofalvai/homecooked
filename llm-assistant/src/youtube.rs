use crate::{output::stream_to_stdout, tools};

pub async fn ask(url: String, prompt: Option<String>, model: Option<String>) -> anyhow::Result<()> {
    let stream = tools::youtube::run(url, prompt, model.as_deref(), std::io::stdout()).await?;

    stream_to_stdout(stream).await?;

    Ok(())
}
