use anyhow::Result;
use cc_switch_core::{AppType, Database};
use clap::Subcommand;
use std::io::{self, Write};

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

    /// Show detailed provider information
    Show {
        /// Provider ID
        id: String,

        /// App type: claude, codex, or gemini
        #[arg(short, long, default_value = "claude")]
        app: String,
    },

    /// Switch to a different provider
    Switch {
        /// Provider ID to switch to
        id: String,

        /// App type: claude, codex, or gemini
        #[arg(short, long, default_value = "claude")]
        app: String,
    },

    /// Delete a provider
    Delete {
        /// Provider ID to delete
        id: String,

        /// App type: claude, codex, or gemini
        #[arg(short, long, default_value = "claude")]
        app: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Export providers to JSON file
    Export {
        /// App type: claude, codex, or gemini
        #[arg(short, long, default_value = "claude")]
        app: String,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Import providers from JSON file
    Import {
        /// App type: claude, codex, or gemini
        #[arg(short, long, default_value = "claude")]
        app: String,

        /// Input file path (stdin if not specified)
        #[arg(short, long)]
        input: Option<String>,
    },

    /// Add a new provider
    Add {
        /// App type: claude, codex, or gemini
        #[arg(short, long, default_value = "claude")]
        app: String,

        /// Provider name
        #[arg(long)]
        name: Option<String>,

        /// API key
        #[arg(long)]
        api_key: Option<String>,

        /// Base URL
        #[arg(long)]
        base_url: Option<String>,

        /// Interactive mode (prompt for all values)
        #[arg(short, long)]
        interactive: bool,
    },
}

pub fn handle(cmd: ProviderCommands) -> Result<()> {
    match cmd {
        ProviderCommands::List { app, format } => list(app, format),
        ProviderCommands::Show { id, app } => show(id, app),
        ProviderCommands::Switch { id, app } => switch(id, app),
        ProviderCommands::Delete { id, app, yes } => delete(id, app, yes),
        ProviderCommands::Export { app, output } => export(app, output),
        ProviderCommands::Import { app, input } => import(app, input),
        ProviderCommands::Add {
            app,
            name,
            api_key,
            base_url,
            interactive,
        } => add(app, name, api_key, base_url, interactive),
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

fn show(id: String, app: String) -> Result<()> {
    let db = Database::init()?;
    let app_type =
        AppType::from_str(&app).ok_or_else(|| anyhow::anyhow!("Invalid app type: {}", app))?;

    let provider = db
        .get_provider_by_id(&id, app_type.as_str())?
        .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", id))?;

    println!("ID: {}", provider.id);
    println!("Name: {}", provider.name);
    if let Some(category) = &provider.category {
        println!("Category: {}", category);
    }
    if let Some(website) = &provider.website_url {
        println!("Website: {}", website);
    }
    if let Some(notes) = &provider.notes {
        println!("Notes: {}", notes);
    }

    println!("\nConfiguration:");
    println!(
        "{}",
        serde_json::to_string_pretty(&provider.settings_config)?
    );

    Ok(())
}

fn switch(id: String, app: String) -> Result<()> {
    let db = Database::init()?;
    let app_type =
        AppType::from_str(&app).ok_or_else(|| anyhow::anyhow!("Invalid app type: {}", app))?;

    // Verify provider exists
    let provider = db
        .get_provider_by_id(&id, app_type.as_str())?
        .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", id))?;

    // Set as current
    db.set_current_provider(app_type.as_str(), &id)?;

    println!("✓ Switched to provider: {} ({})", provider.name, id);
    Ok(())
}

fn delete(id: String, app: String, yes: bool) -> Result<()> {
    let db = Database::init()?;
    let app_type =
        AppType::from_str(&app).ok_or_else(|| anyhow::anyhow!("Invalid app type: {}", app))?;

    // Check if provider exists
    let provider = db
        .get_provider_by_id(&id, app_type.as_str())?
        .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", id))?;

    // Check if current
    let current = db.get_current_provider(app_type.as_str())?;
    if current.as_ref().map(|c| c == &id).unwrap_or(false) {
        anyhow::bail!("Cannot delete current provider. Switch to another provider first.");
    }

    // Confirm deletion
    if !yes {
        print!("Delete provider '{}' ({})? [y/N]: ", provider.name, id);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    db.delete_provider(app_type.as_str(), &id)?;
    println!("✓ Deleted provider: {}", id);

    Ok(())
}

fn export(app: String, output: Option<String>) -> Result<()> {
    let db = Database::init()?;
    let app_type =
        AppType::from_str(&app).ok_or_else(|| anyhow::anyhow!("Invalid app type: {}", app))?;

    let providers = db.get_all_providers(app_type.as_str())?;
    let json = serde_json::to_string_pretty(&providers)?;

    if let Some(path) = output {
        std::fs::write(&path, json)?;
        println!("✓ Exported {} providers to {}", providers.len(), path);
    } else {
        println!("{}", json);
    }

    Ok(())
}

fn import(app: String, input: Option<String>) -> Result<()> {
    let db = Database::init()?;
    let app_type =
        AppType::from_str(&app).ok_or_else(|| anyhow::anyhow!("Invalid app type: {}", app))?;

    let json = if let Some(path) = input {
        std::fs::read_to_string(&path)?
    } else {
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let providers: indexmap::IndexMap<String, cc_switch_core::Provider> =
        serde_json::from_str(&json)?;

    let mut count = 0;
    for (_, provider) in providers {
        db.save_provider(app_type.as_str(), &provider)?;
        count += 1;
    }

    println!("✓ Imported {} providers", count);
    Ok(())
}

fn prompt(message: &str) -> io::Result<String> {
    print!("{}: ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_optional(message: &str, default: &str) -> io::Result<String> {
    print!("{} [{}]: ", message, default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn add(
    app: String,
    name: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    interactive: bool,
) -> Result<()> {
    let db = Database::init()?;
    let app_type =
        AppType::from_str(&app).ok_or_else(|| anyhow::anyhow!("Invalid app type: {}", app))?;

    let provider_name = if interactive || name.is_none() {
        prompt("Provider name")?
    } else {
        name.unwrap()
    };

    if provider_name.is_empty() {
        anyhow::bail!("Provider name cannot be empty");
    }

    let api_key_val = if interactive || api_key.is_none() {
        prompt("API key")?
    } else {
        api_key.unwrap()
    };

    let default_url = match app_type {
        AppType::Claude => "https://api.anthropic.com",
        AppType::Codex => "https://api.openai.com/v1",
        AppType::Gemini => "https://generativelanguage.googleapis.com",
    };

    let base_url_val = if interactive || base_url.is_none() {
        prompt_optional("Base URL", default_url)?
    } else {
        base_url.unwrap()
    };

    // Build provider config based on app type
    let settings_config = match app_type {
        AppType::Claude => serde_json::json!({
            "env": {
                "ANTHROPIC_API_KEY": api_key_val,
                "ANTHROPIC_BASE_URL": base_url_val
            }
        }),
        AppType::Codex => serde_json::json!({
            "env": {
                "OPENAI_API_KEY": api_key_val,
                "OPENAI_BASE_URL": base_url_val
            }
        }),
        AppType::Gemini => serde_json::json!({
            "apiKey": api_key_val,
            "baseUrl": base_url_val
        }),
    };

    let provider = cc_switch_core::Provider {
        id: uuid::Uuid::new_v4().to_string(),
        name: provider_name.clone(),
        settings_config,
        website_url: None,
        category: Some("custom".to_string()),
        created_at: Some(chrono::Utc::now().timestamp_millis()),
        sort_index: None,
        notes: None,
        meta: None,
        icon: None,
        icon_color: None,
        is_proxy_target: None,
    };

    let provider_id = provider.id.clone();
    db.save_provider(app_type.as_str(), &provider)?;

    println!("✓ Added provider: {} ({})", provider_name, provider_id);
    Ok(())
}
