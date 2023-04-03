use anyhow::{Context, Ok};
use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

mod builder;
mod common;
mod config;
mod cost;
mod prompt;
mod search;
mod types;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Build and store embeddings of all notes")]
    Build {
        #[arg(long)]
        dry_run: bool,
    },

    #[command(about = "Calculate cost of embeddings")]
    Cost,

    #[command(about = "Search embeddings")]
    Search {
        #[arg(value_name = "STRING")]
        query: Option<String>,
    },

    #[command(about = "Get related notes to a specific note")]
    Related {
        #[arg(value_name = "RELATIVE_PATH")]
        path: Option<String>,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    init_logging()?;

    let cli = Cli::parse();

    let config = config::load_config().context("Can't load config")?;

    match &cli.command {
        Commands::Build { dry_run } => builder::build(&config, dry_run.to_owned()).await?,
        Commands::Search { query } => search::query(&config, query.as_deref()).await?,
        Commands::Cost => cost::calculate_cost(&config)?,
        Commands::Related { path } => search::related(&config, path)?,
    }
    Ok(())
}

fn init_logging() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()?
        .add_directive(LevelFilter::ERROR.into())
        .add_directive("async-openai::client=warn".parse()?); // Rate limit traces

    tracing_subscriber::fmt().with_env_filter(filter).init();
    Ok(())
}
