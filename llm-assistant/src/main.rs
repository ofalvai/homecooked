use clap::{Parser, Subcommand};
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

mod completion;
mod models;
mod output;
mod readwise;
mod web;
mod youtube;
mod smartgpt;

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

        #[arg(short, long)]
        model: Option<String>,
    },

    #[command(
        about = "Work with the contents of a web page. If no prompt is provided, it summarizes the page."
    )]
    Web {
        #[arg(value_name = "URL", short, long)]
        url: String,

        #[arg(short, long)]
        prompt: Option<String>,

        #[arg(short, long)]
        model: Option<String>,
    },

    #[command(about = "Ask questions about a Readwise document list")]
    Readwise {
        #[arg(short, long)]
        prompt: String,
    },

    #[command(
        about = "Ask questions about a Youtube video transcription. If no prompt is provided, it summarizes the transcription."
    )]
    Youtube {
        #[arg(short, long)]
        url: String,

        #[arg(short, long)]
        prompt: Option<String>,

        #[arg(short, long)]
        model: Option<String>,
    },

    #[command(about = "SmartGPT-style prompting")]
    SmartGPT {
        #[arg(short, long)]
        prompt: String,

        #[arg(short, long)]
        model: Option<String>,
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
        Commands::Completion {
            prompt,
            template,
            model,
        } => completion::completion(prompt, template, model).await,
        Commands::Web { url, prompt, model } => web::prompt(url, prompt, model).await,
        Commands::Models => models::models(),
        Commands::Readwise { prompt } => readwise::ask(prompt).await,
        Commands::Youtube { url, prompt, model } => youtube::ask(url, prompt, model).await,
        Commands::SmartGPT { prompt , model } => smartgpt::prompt(prompt, model).await,
    };
}
