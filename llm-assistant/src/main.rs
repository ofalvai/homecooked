use clap::{Parser, Subcommand};
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

mod completion;
mod models;
mod output;
mod readwise;
mod summary;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(value_name = "LEVEL", short, long, default_value_t = LevelFilter::Warn)]
    log_level: LevelFilter,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Generate completions for a prompt")]
    Completion {
        #[arg(value_name = "STRING", short, long)]
        prompt: String,

        #[arg(value_name = "TEMPLATE", short, long)]
        template: Option<String>,
    },

    #[command(about = "Summarize input like a file, web URL or string")]
    Summary {
        input: String,
        prompt: Option<String>,
    },

    #[command(about = "Ask questions about a Readwise document list")]
    Readwise {
        #[arg(short, long)]
        prompt: String,
    },

    #[command(about = "Manage available models")]
    Models,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let term_config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Debug)
        .set_level_padding(simplelog::LevelPadding::Right)
        .build();
    TermLogger::init(
        cli.log_level,
        term_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    return match cli.command {
        Commands::Completion { prompt, template } => completion::completion(prompt, template).await,
        Commands::Summary { input, prompt } => summary::summarize(input, prompt).await,
        Commands::Models => models::models(),
        Commands::Readwise { prompt } => readwise::ask(prompt).await,
    };
}
