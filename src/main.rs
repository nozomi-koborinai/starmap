mod commands;
mod generator;
mod github;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "starmap", about = "Generate Awesome Lists from your GitHub Stars, organized by Lists")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Export the awesome list to a file
    Export {
        /// Output file path
        path: String,
    },
    /// Push the awesome list to a GitHub repository
    Push {
        /// Target repository (owner/name)
        #[arg(long)]
        repo: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => commands::show::run().await?,
        Some(Commands::Export { path }) => commands::export::run(&path).await?,
        Some(Commands::Push { repo }) => commands::push::run(&repo).await?,
    }

    Ok(())
}
