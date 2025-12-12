use anyhow::Result;
use cc_switch_core::{AppType, Database};
use clap::Subcommand;
use dialoguer::{theme::ColorfulTheme, Select};
use std::io::{self, Write};

#[derive(Subcommand)]
pub enum ProviderCommands {
    /// List all providers (interactive selection by default)
    #[command(alias = "ls")]
    List {
        /// App type: claude, codex, or gemini
        #[arg(short, long, default_value = "claude")]
        app: String,

        /// Output format: json (default is interactive selection)
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Show detailed provider information
    #[command(alias = "info")]
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
    #[command(alias = "rm")]
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
    #[command(alias = "new")]
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

fn list(app: String, format: Option<String>) -> Result<()> {
    let db = Database::init()?;
    let app_type =
        AppType::from_str(&app).ok_or_else(|| anyhow::anyhow!("Invalid app type: {}", app))?;

    let providers = db.get_all_providers(app_type.as_str())?;
    let current = db.get_current_provider(app_type.as_str())?;

    if providers.is_empty() {
        println!("No providers found for {}. Use 'cc-switch provider add' to add one.", app);
        return Ok(());
    }

    // If JSON format is requested, output JSON
    if format.as_deref() == Some("json") {
        println!("{}", serde_json::to_string_pretty(&providers)?);
        return Ok(());
    }

    // Default: Interactive mode - allow selection to view details
    interactive_list(providers, current, app)
}

/// Interactive list with arrow key selection to view details
fn interactive_list(
    providers: indexmap::IndexMap<String, cc_switch_core::Provider>,
    current: Option<String>,
    app: String,
) -> Result<()> {
    // Build display items
    let items: Vec<String> = providers
        .iter()
        .map(|(id, p)| {
            let marker = if current.as_ref() == Some(id) { " ✓" } else { "" };
            let category = p.category.as_deref().unwrap_or("-");
            format!("{} [{}]{}", p.name, category, marker)
        })
        .collect();

    let ids: Vec<&String> = providers.keys().collect();

    // Find current selection index
    let mut default = current
        .as_ref()
        .and_then(|c| ids.iter().position(|id| *id == c))
        .unwrap_or(0);

    // Loop to allow continuous browsing
    loop {
        // Show interactive selection
        let selection = match Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Select {} provider (↑↓ to navigate, Enter to view details, Esc to exit)", app))
            .items(&items)
            .default(default)
            .interact_opt()
        {
            Ok(Some(idx)) => idx,
            Ok(None) => {
                // User pressed Esc on the list - exit
                return Ok(());
            }
            Err(e) => {
                // If interaction fails (e.g., not a TTY), fall back to simple list
                eprintln!("Interactive mode not available: {}", e);
                eprintln!("Showing provider list instead:");
                for (i, (id, provider)) in providers.iter().enumerate() {
                    let marker = if current.as_ref() == Some(id) { " ✓" } else { "" };
                    println!("{}. {} ({}){}", i + 1, provider.name, id, marker);
                }
                return Ok(());
            }
        };

        let selected_id = ids[selection];
        let provider = providers.get(selected_id).unwrap();

        // Display detailed information
        println!("\n{}", "=".repeat(60));
        println!("Provider Details");
        println!("{}", "=".repeat(60));
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
        println!("{}", "=".repeat(60));

        // Wait for user to press Enter to return to list
        println!("\nPress Enter to return to list...");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Ok(());
        }

        // Remember the last selection for next iteration
        default = selection;
        println!();
    }
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

    // Set as current in database
    db.set_current_provider(app_type.as_str(), &id)?;

    // Write live config
    write_live_config(&app_type, &provider)?;

    println!("✓ Switched to provider: {} ({})", provider.name, id);
    Ok(())
}

/// Write provider config to live configuration files
fn write_live_config(app_type: &AppType, provider: &cc_switch_core::Provider) -> Result<()> {
    match app_type {
        AppType::Claude => {
            let path = cc_switch_core::get_claude_settings_path();
            cc_switch_core::write_json_file(&path, &provider.settings_config)?;
            println!("  Updated: {}", path.display());
        }
        AppType::Codex => {
            // Codex uses auth.json and config.toml
            let obj = provider
                .settings_config
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("Codex config must be a JSON object"))?;

            if let Some(auth) = obj.get("auth") {
                let auth_path = cc_switch_core::get_codex_config_dir().join("auth.json");
                cc_switch_core::write_json_file(&auth_path, auth)?;
                println!("  Updated: {}", auth_path.display());
            }

            if let Some(config) = obj.get("config").and_then(|v| v.as_str()) {
                let config_path = cc_switch_core::get_codex_config_dir().join("config.toml");
                cc_switch_core::write_text_file(&config_path, config)?;
                println!("  Updated: {}", config_path.display());
            }
        }
        AppType::Gemini => {
            // Gemini uses .env file and settings.json
            let config_dir = cc_switch_core::get_gemini_config_dir();

            if let Some(env_obj) = provider.settings_config.get("env").and_then(|v| v.as_object()) {
                let env_path = config_dir.join(".env");
                let env_content: String = env_obj
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or("")))
                    .collect::<Vec<_>>()
                    .join("\n");
                cc_switch_core::write_text_file(&env_path, &env_content)?;
                println!("  Updated: {}", env_path.display());
            }

            if let Some(config) = provider.settings_config.get("config") {
                if config.is_object() {
                    let settings_path = config_dir.join("settings.json");
                    cc_switch_core::write_json_file(&settings_path, config)?;
                    println!("  Updated: {}", settings_path.display());
                }
            }
        }
    }
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

/// Interactive provider switch with arrow key selection
pub fn interactive_switch(app: String) -> Result<()> {
    let db = Database::init()?;
    let app_type =
        AppType::from_str(&app).ok_or_else(|| anyhow::anyhow!("Invalid app type: {}", app))?;

    let providers = db.get_all_providers(app_type.as_str())?;
    let current = db.get_current_provider(app_type.as_str())?;

    if providers.is_empty() {
        println!("No providers found for {}. Use 'cc-switch provider add' to add one.", app);
        return Ok(());
    }

    // Build display items
    let items: Vec<String> = providers
        .iter()
        .map(|(id, p)| {
            let marker = if current.as_ref() == Some(id) { " ✓" } else { "" };
            format!("{} ({}){}", p.name, id, marker)
        })
        .collect();

    let ids: Vec<&String> = providers.keys().collect();

    // Find current selection index
    let default = current
        .as_ref()
        .and_then(|c| ids.iter().position(|id| *id == c))
        .unwrap_or(0);

    // Show interactive selection
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Select {} provider (↑↓ to move, Enter to select)", app))
        .items(&items)
        .default(default)
        .interact()?;

    let selected_id = ids[selection];

    // If already current, skip
    if current.as_ref() == Some(selected_id) {
        println!("Already using: {}", providers[selected_id].name);
        return Ok(());
    }

    // Get provider and switch
    let provider = providers.get(selected_id).unwrap();

    db.set_current_provider(app_type.as_str(), selected_id)?;
    write_live_config(&app_type, provider)?;

    println!("✓ Switched to: {}", provider.name);
    Ok(())
}
