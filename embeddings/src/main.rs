use std::path::PathBuf;

use anyhow::Ok;
use clap::{Parser, Subcommand};

mod builder;
mod config;
mod query;
mod types;
mod cost;
mod common;

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
        #[arg(short, long, value_name = "PATH")]
        notes: String,

        #[arg(long)]
        dry_run: bool,
    },

    #[command(about = "Calculate cost of embeddings")]
    Cost {
        #[arg(short, long, value_name = "PATH")]
        notes: String,
    },

    #[command(about = "Search embeddings")]
    Query {
        #[arg(value_name = "STRING")]
        query: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { notes, dry_run } => builder::build(PathBuf::from(notes), dry_run.to_owned()).await?,
        Commands::Query { query } => query::query(query).await?,
        Commands::Cost { notes } => cost::calculate_cost(PathBuf::from(notes))?,
    }

    Ok(())
}
