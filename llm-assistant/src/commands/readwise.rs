use crate::{config::Config, output::print_tool_stream, tools};

pub async fn ask(config: Config, question: String) -> anyhow::Result<()> {
    let stream = tools::readwise::run(config, question);
    print_tool_stream(stream).await?;
    Ok(())
}
