//! CC Switch Core Library
//!
//! Shared business logic for provider management, database access,
//! and configuration handling. Used by both Tauri GUI and CLI.

pub mod config;
pub mod database;
pub mod error;
pub mod provider;

// Re-export commonly used types
pub use config::{get_app_config_dir, AppType};
pub use database::Database;
pub use error::CoreError;
pub use provider::{Provider, ProviderMeta};
