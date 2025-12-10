use anyhow::Result;
use cc_switch_core::{AppType, Database};
use clap::Subcommand;

use crate::output::create_table;

#[derive(Subcommand)]
pub enum ProviderCommands {
    /// List all providers
    List {
        /// App type: claude, codex, or gemini
        #[arg(short, long, default_value = "claude")]
        app: String,

        /// Output format: table or json
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

pub fn handle(cmd: ProviderCommands) -> Result<()> {
    match cmd {
        ProviderCommands::List { app, format } => list(app, format),
    }
}

fn list(app: String, format: String) -> Result<()> {
    let db = Database::init()?;
    let app_type =
        AppType::from_str(&app).ok_or_else(|| anyhow::anyhow!("Invalid app type: {}", app))?;

    let providers = db.get_all_providers(app_type.as_str())?;
    let current = db.get_current_provider(app_type.as_str())?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&providers)?);
        return Ok(());
    }

    // Table format
    let mut table = create_table(vec!["ID", "Name", "Category", "Current"]);

    for (id, provider) in providers.iter() {
        let is_current = current.as_ref().map(|c| c == id).unwrap_or(false);
        table.add_row(vec![
            id.as_str(),
            &provider.name,
            provider.category.as_deref().unwrap_or("-"),
            if is_current { "✓" } else { "" },
        ]);
    }

    println!("{}", table);
    Ok(())
}
