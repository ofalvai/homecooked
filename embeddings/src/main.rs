use anyhow::{Context, Ok};
use clap::{Parser, Subcommand};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

mod builder;
mod common;
mod config;
mod cost;
mod graph;
mod plot;
mod prompt;
mod search;
mod types;
mod unlinked;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create or update embeddings for all notes")]
    Build {
        #[arg(long)]
        dry_run: bool,
    },

    #[command(about = "Calculate the cost of creating embeddings for all notes")]
    Cost,

    #[command(about = "Search relevant notes for a query string")]
    Search {
        #[arg(value_name = "STRING")]
        query: Option<String>,
    },

    #[command(about = "Get related notes to a specific note")]
    Related {
        #[arg(value_name = "RELATIVE_PATH")]
        path: Option<String>,
    },

    #[command(about = "Plot embeddings in 2D and open result in browser")]
    Plot,

    #[command(about = "Prune embeddings of no longer existing notes")]
    Prune,

    #[command(about = "Find similar notes that are not linked")]
    Unlinked {
        #[arg(long, value_name = "FILE")]
        output: Option<String>,

        #[arg(long, value_name = "PERCENT", default_value = "70")]
        threshold: u8,

        #[arg(long, value_name = "PATH")]
        exclude: Vec<String>,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    init_logging().context("Failed to init logging")?;

    let cli = Cli::parse();

    let config = config::load_config().context("Can't load config")?;

    match &cli.command {
        Commands::Build { dry_run } => builder::build(&config, dry_run.to_owned()).await?,
        Commands::Search { query } => search::query(&config, query.as_deref()).await?,
        Commands::Cost => cost::calculate_cost(&config)?,
        Commands::Related { path } => search::related(&config, path)?,
        Commands::Plot => plot::plot(&config)?,
        Commands::Prune => builder::prune(&config)?,
        Commands::Unlinked { output, threshold, exclude } => unlinked::handle_unlinked(&config, output.as_deref(), *threshold, exclude).await?,
    }
    Ok(())
}

fn init_logging() -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .parse("async-openai::client=warn")?; // Rate limit traces

    tracing_subscriber::fmt().with_env_filter(filter).init();
    Ok(())
}
