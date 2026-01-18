use clap::{Parser, Subcommand};
use clean::run_clean;
use export::run_export;

mod clean;
mod export;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Export matching notes from Obsidian vault to plain Markdown notes")]
    #[command(arg_required_else_help = true)]
    Export {
        #[arg(short, long, value_name = "PATH")]
        root: String,

        #[arg(short, long, default_value_t = String::from("./output"), value_name = "PATH")]
        destination: String,
    },

    #[command(about = "Clean unreferenced attachments")]
    Clean {
        #[arg(short, long, value_name = "PATH")]
        root: String,

        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Export { root, destination } => match run_export(root, destination) {
            Ok(()) => println!("Success!"),
            Err(e) => println!("{e}"),
        },
        Commands::Clean { root, dry_run } => match run_clean(root, *dry_run) {
            Ok(()) => println!("Success"),
            Err(e) => println!("{e}"),
        },
    }
}
