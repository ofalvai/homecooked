use anyhow::Context;
use clap::{Parser, Subcommand};
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

mod commands;
mod config;
mod models;
mod output;
mod server;
mod templates;
mod tools;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(value_name = "LEVEL", short, long, default_value_t = LevelFilter::Info)]
    log_level: LevelFilter,

    #[arg(value_name = "FILE", short, long)]
    config: Option<String>,
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

    #[command(about = "Start the web server")]
    Server {
        #[arg(short, long)]
        port: Option<u16>,
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

    let config = config::load_config(cli.config).context("Load config")?;

    return match cli.command {
        Commands::Completion {
            prompt,
            template,
            model,
        } => commands::completion::completion(config, prompt, template, model).await,
        Commands::Web { url, prompt, model } => {
            commands::web::prompt(config, url, prompt, model).await
        }
        Commands::Models => models::models(),
        Commands::Readwise { prompt } => commands::readwise::ask(config, prompt).await,
        Commands::Youtube { url, prompt, model } => {
            commands::youtube::ask(config, url, prompt, model).await
        }
        Commands::SmartGPT { prompt, model } => {
            commands::smartgpt::prompt(config, prompt, model).await
        }
        Commands::Server { port } => server::start(config, port).await,
    };
}
