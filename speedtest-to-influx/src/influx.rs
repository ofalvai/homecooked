use anyhow::Context;
use chrono::{DateTime, Utc};
use influxdb::{Client, InfluxDbWriteable};

use crate::speedtest::Output;

#[derive(Clone)]
pub struct ConnectionParams {
    pub url: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(InfluxDbWriteable)]
struct PingMeasurement {
    time: DateTime<Utc>,
    ping: f32,
    ping_low: f32,
    ping_high: f32,
    jitter: f32,
}

#[derive(InfluxDbWriteable)]
struct BandwidthMeasurement {
    time: DateTime<Utc>,
    bandwidth: i32, // bytes
    latency_jitter: f32,
    latency_low: f32,
    latency_high: f32,
    latency_iqm: f32,
    packet_loss: Option<f32>, // not always available
}

pub async fn store(params: ConnectionParams, output: Output) -> anyhow::Result<()> {
    let queries = vec![
        PingMeasurement {
            time: output.timestamp,
            ping: output.ping.latency,
            ping_low: output.ping.low,
            ping_high: output.ping.high,
            jitter: output.ping.jitter,
        }
        .into_query("speedtest_ping")
        .add_tag("isp", output.isp.clone())
        .add_tag("internal_ip", output.interface.internal_ip.clone())
        .add_tag("mac_addr", output.interface.mac_addr.clone())
        .add_tag("server_host", output.server.host.clone())
        .add_tag("server_location", output.server.location.clone()),
        BandwidthMeasurement {
            time: output.timestamp,
            bandwidth: output.download.bandwidth as i32,
            latency_jitter: output.download.latency.jitter,
            latency_low: output.download.latency.low,
            latency_high: output.download.latency.high,
            latency_iqm: output.download.latency.iqm,
            packet_loss: output.packet_loss,
        }
        .into_query("speedtest_download")
        .add_tag("isp", output.isp.clone())
        .add_tag("internal_ip", output.interface.internal_ip.clone())
        .add_tag("mac_addr", output.interface.mac_addr.clone())
        .add_tag("server_host", output.server.host.clone())
        .add_tag("server_location", output.server.location.clone()),
        BandwidthMeasurement {
            time: output.timestamp,
            bandwidth: output.upload.bandwidth as i32,
            latency_jitter: output.upload.latency.jitter,
            latency_low: output.upload.latency.low,
            latency_high: output.upload.latency.high,
            latency_iqm: output.upload.latency.iqm,
            packet_loss: output.packet_loss,
        }
        .into_query("speedtest_upload")
        .add_tag("isp", output.isp.clone())
        .add_tag("internal_ip", output.interface.internal_ip.clone())
        .add_tag("mac_addr", output.interface.mac_addr.clone())
        .add_tag("server_host", output.server.host.clone())
        .add_tag("server_location", output.server.location.clone()),
    ];

    let client =
        Client::new(params.url, params.database).with_auth(params.username, params.password);

    let _ = client.query(queries).await.context("InfluxDB insert query failed");

    Ok(())
}
