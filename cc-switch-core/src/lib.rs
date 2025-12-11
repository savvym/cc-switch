//! CC Switch Core Library
//!
//! Shared business logic for provider management, database access,
//! and configuration handling. Used by both Tauri GUI and CLI.

pub mod config;
pub mod database;
pub mod error;
pub mod provider;

// Re-export commonly used types
pub use config::{
    get_app_config_dir, get_claude_settings_path, get_codex_config_dir, get_database_path,
    get_gemini_config_dir, write_json_file, write_text_file, AppType,
};
pub use database::Database;
pub use error::{CoreError, Result};
pub use provider::{Provider, ProviderManager, ProviderMeta};
