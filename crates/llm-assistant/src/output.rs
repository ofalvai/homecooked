use futures::StreamExt;
use owo_colors::OwoColorize;
use std::{
    io::{stdout, Write},
    pin::Pin,
};

use futures::Stream;

use llm_toolkit::provider::{CompletionError, CompletionResponseDelta};

use crate::tools::{ToolEventStream, ToolUseEvent};

pub async fn print_completion_stream(
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
            Err(e) => eprintln!("{e}"),
        }
        stdout().flush()?;
    }
    Ok(())
}

pub async fn print_tool_stream(mut stream: ToolEventStream) -> anyhow::Result<()> {
    while let Some(result) = stream.next().await {
        match result {
            ToolUseEvent::Output(output) => {
                println!("{}", "[OUTPUT]".magenta());
                println!("{}", output.content.yellow());
            }
            ToolUseEvent::IntermediateOutput(content) => {
                println!("{}", "[INTERMEDIATE OUTPUT]".magenta());
                println!("{}", content.label.dimmed());
                println!("{}", content.content.dimmed());
            }
            ToolUseEvent::Error(e) => {
                println!("{}", "[ERROR]".magenta());
                eprintln!("{}", e.label.red());
                if let Some(error) = e.error {
                    eprintln!("{}", error.red());
                }
            }
            ToolUseEvent::Working(e) => {
                println!("{}", "[WORKING]".magenta());
                eprintln!("{}", e.label.blue());
            }
            ToolUseEvent::Finished(_) => {
                println!("{}", "[FINISHED]".magenta());
            }
        }
    }
    Ok(())
}
