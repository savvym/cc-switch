use clap::{Parser, Subcommand};

mod commands;
mod output;

#[derive(Parser)]
#[command(name = "cc-switch")]
#[command(author, version, about = "Manage AI provider configurations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Provider management
    #[command(subcommand)]
    Provider(commands::provider::ProviderCommands),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Provider(cmd) => commands::provider::handle(cmd),
    }
}
