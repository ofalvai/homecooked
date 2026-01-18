use std::process::Command;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub timestamp: DateTime<Utc>,
    pub ping: Ping,
    pub download: DirectionMeasurement,
    pub upload: DirectionMeasurement,
    pub packet_loss: Option<f32>, // not always available
    pub isp: String,
    pub interface: Interface,
    pub server: Server,
    pub result: ResultMetadata,
}

#[derive(Deserialize, Debug)]
pub struct Ping {
    pub jitter: f32,
    pub latency: f32,
    pub low: f32,
    pub high: f32,
}

#[derive(Deserialize, Debug)]
pub struct Latency {
    pub iqm: f32,
    pub low: f32,
    pub high: f32,
    pub jitter: f32,
}

#[derive(Deserialize, Debug)]
pub struct DirectionMeasurement {
    pub bandwidth: u32, // bytes
    pub latency: Latency,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Interface {
    pub internal_ip: String,
    pub mac_addr: String,
}

#[derive(Deserialize, Debug)]
pub struct ResultMetadata {
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub host: String,
    pub location: String,
}

pub fn run_measurement() -> anyhow::Result<Output> {
    let mut command = Command::new("speedtest");
    command.arg("--format").arg("json-pretty");
    let command_output = command
        .output()
        .context("Failed to execute speedtest command")?;
    let raw_output = String::from_utf8_lossy(&command_output.stdout);
    let output: Output = serde_json::from_str(&raw_output)
        .with_context(|| format!("Failed to parse JSON output: {raw_output}"))?;

    Ok(output)
}
