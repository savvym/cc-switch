use clap::{Parser, Subcommand};

mod commands;
mod output;

#[derive(Parser)]
#[command(name = "cc-switch")]
#[command(author, version, about = "Manage AI provider configurations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// App type for interactive mode: claude, codex, or gemini
    #[arg(short, long, default_value = "claude", global = true)]
    app: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Provider management
    #[command(subcommand, alias = "p")]
    Provider(commands::provider::ProviderCommands),

    /// Interactive provider switch (alias for quick access)
    #[command(name = "s")]
    Switch {
        /// App type: claude, codex, or gemini
        #[arg(short, long, default_value = "claude")]
        app: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Provider(cmd)) => commands::provider::handle(cmd),
        Some(Commands::Switch { app }) => commands::provider::interactive_switch(app),
        None => commands::provider::interactive_switch(cli.app),
    }
}
