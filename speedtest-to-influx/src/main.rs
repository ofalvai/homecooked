use clap::{Parser, Subcommand};
use duration_string::DurationString;
use influx::ConnectionParams;
use owo_colors::OwoColorize;
use speedtest::Output;
use tokio::{task, time};

mod influx;
mod speedtest;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, required = false)]
    url: String,

    #[arg(long, global = true, required = false)]
    username: String,

    #[arg(long, global = true, required = false)]
    password: String,

    #[arg(long, global = true, required = false)]
    database: String,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Run speedtest and report results to DB")]
    Run,

    #[command(about = "Run speedtest on a schedule")]
    Schedule {
        #[arg(short, long)]
        interval: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let params = ConnectionParams {
        url: cli.url,
        username: cli.username,
        password: cli.password,
        database: cli.database,
    };

    match cli.command {
        Commands::Run => run(params).await?,
        Commands::Schedule { interval } => run_scheduled(params, interval).await?,
    }

    Ok(())
}

async fn run(connection_params: ConnectionParams) -> anyhow::Result<()> {
    println!("{}", "Running speedtest...".cyan());
    let output = speedtest::run_measurement()?;

    print_output(&output);

    influx::store(connection_params, output).await?;

    Ok(())
}

async fn run_scheduled(
    connection_params: ConnectionParams,
    duration_string: String,
) -> anyhow::Result<()> {
    println!(
        "Scheduling speedtest to run every {}",
        &duration_string.cyan()
    );
    println!();

    let forever = task::spawn(async move {
        let mut interval =
            time::interval(DurationString::from_string(duration_string).unwrap().into());

        loop {
            interval.tick().await;
            match run(connection_params.clone()).await {
                Ok(_) => println!(),
                Err(e) => println!("{}: {}", "Error".red(), e),
            }

            println!();
        }
    });

    _ = forever.await;

    Ok(())
}

fn print_output(output: &Output) {
    println!();
    println!(
        "Bandwidth:\t\t🔻{} 🔺{}",
        format_bandwidth(output.download.bandwidth).cyan(),
        format_bandwidth(output.upload.bandwidth).cyan(),
    );

    // let packet_loss_style = if output.packet_loss > 0.0 {
    //     Style::new().yellow()
    // } else {
    //     Style::new().green()
    // };
    let packet_loss_value = match output.packet_loss {
        Some(val) => format!("{}%", val),
        None => String::from("not available"),
    };
    println!(
        "Packet loss:\t\t{}",
        format!("{}", packet_loss_value.yellow())
    );
    println!(
        "Idle latency:\t\t{} ({}/{}) {}",
        format!("{:.0}ms", output.ping.latency).yellow(),
        format!("{:.0}ms", output.ping.low).green(),
        format!("{:.0}ms", output.ping.high).red(),
        format!("{:.0}ms jitter", output.ping.jitter).yellow()
    );
    println!(
        "Download latency:\t{} ({}/{}) {}",
        format!("{:.0}ms", output.download.latency.iqm).yellow(),
        format!("{:.0}ms", output.download.latency.low).green(),
        format!("{:.0}ms", output.download.latency.high).red(),
        format!("{:.0}ms jitter", output.download.latency.jitter).yellow(),
    );
    println!(
        "Upload latency:\t\t{} ({}/{}) {}",
        format!("{:.0}ms", output.upload.latency.iqm).yellow(),
        format!("{:.0}ms", output.upload.latency.low).green(),
        format!("{:.0}ms", output.upload.latency.high).red(),
        format!("{:.0}ms jitter", output.upload.latency.jitter).yellow(),
    );
    println!("---");
    println!("Server:\t\t\t{}", output.server.host.dimmed());
    println!("Location:\t\t{}", output.server.location.dimmed());
    println!("Result URL:\t\t{}", output.result.url.dimmed().underline())
}

fn format_bandwidth(bytes: u32) -> String {
    let mbits = bytes as f32 * 8.0 / 1000.0 / 1000.0;
    format!("{:.0} Mbit/s", mbits)
}
