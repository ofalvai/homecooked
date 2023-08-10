use clap::{Parser, Subcommand};

mod completion;
mod models;
mod output;
mod summary;

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

        #[arg(value_name = "TEMPLATE", short, long)]
        template: Option<String>
    },

    #[command(about = "Summarize input like a file, web URL or string")]
    Summary {
        input: String,
        prompt: Option<String>,
    },

    #[command(about = "Manage available models")]
    Models,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    return match cli.command {
        Commands::Completion { prompt, template} => completion::completion(prompt, template).await,
        Commands::Summary { input, prompt } => summary::summarize(input, prompt).await,
        Commands::Models => models::models(),
    };
}
