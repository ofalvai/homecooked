use futures::StreamExt;
use owo_colors::OwoColorize;
use std::{
    io::{stdout, Write},
    pin::Pin,
};

use futures::Stream;

use llm_toolkit::provider::{CompletionError, CompletionResponseDelta};

pub async fn stream_to_stdout(
    mut stream: Pin<
        Box<dyn Stream<Item = Result<CompletionResponseDelta, CompletionError>> + Send>,
    >,
) -> anyhow::Result<()> {
    let mut lock = stdout().lock();

    while let Some(result) = stream.next().await {
        match result {
            Ok(resp) => {
                write!(lock, "{}", resp.content.yellow())?;
            }
            Err(e) => eprintln!("{}", e),
        }
        stdout().flush()?;
    }
    Ok(())
}
