use clap::{Parser, Subcommand};

mod completion;
mod models;
mod prompt;
mod provider;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Generate completions for a prompt")]
    Completion {
        #[arg(value_name = "STRING", short, long)]
        prompt: String,
    },

    #[command(about = "Manage available models")]
    Models,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    return match &cli.command {
        Commands::Completion { prompt} => completion::completion(prompt).await,
        Commands::Models => models::models(),
    };
}
