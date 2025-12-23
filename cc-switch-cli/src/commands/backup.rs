//! Database backup and restore commands

use anyhow::Result;
use cc_switch_core::Database;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum BackupCommands {
    /// Export database to SQL file
    Export {
        /// Output file path (default: cc-switch-backup-{timestamp}.sql)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Import database from SQL file
    Import {
        /// Input SQL file path
        input: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

pub fn handle(cmd: BackupCommands) -> Result<()> {
    match cmd {
        BackupCommands::Export { output } => export(output),
        BackupCommands::Import { input, yes } => import(input, yes),
    }
}

fn export(output: Option<String>) -> Result<()> {
    let db = Database::init()?;

    // Generate default filename with timestamp
    let output_path = if let Some(path) = output {
        PathBuf::from(path)
    } else {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        PathBuf::from(format!("cc-switch-backup-{}.sql", timestamp))
    };

    db.export_sql(&output_path)?;
    println!("✓ Database exported to: {}", output_path.display());
    Ok(())
}

fn import(input: String, yes: bool) -> Result<()> {
    let input_path = PathBuf::from(&input);

    if !input_path.exists() {
        anyhow::bail!("File not found: {}", input);
    }

    // Confirm import
    if !yes {
        use std::io::{self, Write};

        println!("Warning: This will overwrite your current database.");
        println!("A backup will be created automatically before import.");
        print!("Continue? [y/N]: ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let db = Database::init()?;
    let backup_id = db.import_sql(&input_path)?;

    if !backup_id.is_empty() {
        println!("✓ Previous database backed up as: {}", backup_id);
    }
    println!("✓ Database imported from: {}", input);
    Ok(())
}
