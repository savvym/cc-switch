//! Database module - SQLite data persistence
//!
//! This module provides core data storage functionality, including:
//! - Provider configuration management
//! - Settings storage
//!
//! ## Architecture
//!
//! ```text
//! database/
//! ├── mod.rs        - Database struct + initialization
//! ├── schema.rs     - Table structure + schema migration
//! └── dao/          - Data access objects
//!     └── providers.rs
//! ```

mod backup;
mod dao;
mod schema;

use crate::config::get_database_path;
use crate::error::{CoreError, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;

/// Current Schema version
/// Increment this when modifying table structure
pub(crate) const SCHEMA_VERSION: i32 = 2;

/// Safely serialize JSON
pub(crate) fn to_json_string<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|e| CoreError::Config(format!("JSON serialization failed: {e}")))
}

/// Safely acquire Mutex lock
macro_rules! lock_conn {
    ($mutex:expr) => {
        $mutex
            .lock()
            .map_err(|e| CoreError::Database(format!("Mutex lock failed: {}", e)))?
    };
}

// Export macro for submodules
pub(crate) use lock_conn;

/// Database connection wrapper
///
/// Uses Mutex to wrap Connection for thread-safe sharing.
pub struct Database {
    pub(crate) conn: Mutex<Connection>,
}

impl Database {
    /// Initialize database connection and create tables
    ///
    /// Database file located at `~/.cc-switch/cc-switch.db`
    pub fn init() -> Result<Self> {
        let db_path = get_database_path();

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path).map_err(|e| CoreError::Database(e.to_string()))?;

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON;", [])
            .map_err(|e| CoreError::Database(e.to_string()))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.create_tables()?;
        db.apply_schema_migrations()?;

        Ok(db)
    }

    /// Create in-memory database (for testing)
    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| CoreError::Database(e.to_string()))?;

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON;", [])
            .map_err(|e| CoreError::Database(e.to_string()))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.create_tables()?;

        Ok(db)
    }
}
