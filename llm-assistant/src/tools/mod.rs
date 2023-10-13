use std::pin::Pin;

use futures::Stream;

pub mod readwise;
pub mod youtube;
pub mod web;

// TODO: move these to llm-toolkit
pub type ToolEventStream = Pin<Box<dyn Stream<Item = ToolUseEvent> + Send>>;

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum ToolUseEvent {
    Working(WorkingEvent),
    Error(ErrorEvent),
    IntermediateOutput(IntermediateOutput),
    Output(OutputEvent),
    Finished(ToolUseMetadata), // TODO: add metrics here
}

#[derive(serde::Serialize)]
pub struct IntermediateOutput {
    pub content: String,
    pub label: String,
}

#[derive(serde::Serialize)]
pub struct WorkingEvent {
    pub label: String,
}

#[derive(serde::Serialize)]
pub struct ErrorEvent {
    pub label: String,
    pub error: Option<String>,
}

#[derive(serde::Serialize)]
pub struct OutputEvent {
    pub content: String,
}

#[derive(serde::Serialize)]
pub struct ToolUseMetadata {}
