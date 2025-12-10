//! Schema definition and migrations
//!
//! Responsible for database table creation and version migrations.

use super::{lock_conn, Database, SCHEMA_VERSION};
use crate::error::{CoreError, Result};
use rusqlite::Connection;

impl Database {
    /// Create all database tables
    pub(crate) fn create_tables(&self) -> Result<()> {
        let conn = lock_conn!(self.conn);
        Self::create_tables_on_conn(&conn)
    }

    /// Create tables on a specific connection (for migration and testing)
    pub(crate) fn create_tables_on_conn(conn: &Connection) -> Result<()> {
        // 1. Providers table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS providers (
                id TEXT NOT NULL,
                app_type TEXT NOT NULL,
                name TEXT NOT NULL,
                settings_config TEXT NOT NULL,
                website_url TEXT,
                category TEXT,
                created_at INTEGER,
                sort_index INTEGER,
                notes TEXT,
                icon TEXT,
                icon_color TEXT,
                meta TEXT NOT NULL DEFAULT '{}',
                is_current BOOLEAN NOT NULL DEFAULT 0,
                is_proxy_target BOOLEAN NOT NULL DEFAULT 0,
                PRIMARY KEY (id, app_type)
            )",
            [],
        )
        .map_err(|e| CoreError::Database(e.to_string()))?;

        // Try adding is_proxy_target column if table exists but column is missing
        let _ = conn.execute(
            "ALTER TABLE providers ADD COLUMN is_proxy_target BOOLEAN NOT NULL DEFAULT 0",
            [],
        );

        // 2. Provider Endpoints table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS provider_endpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                provider_id TEXT NOT NULL,
                app_type TEXT NOT NULL,
                url TEXT NOT NULL,
                added_at INTEGER,
                FOREIGN KEY (provider_id, app_type) REFERENCES providers(id, app_type) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| CoreError::Database(e.to_string()))?;

        // 3. Settings table (general config)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(())
    }

    /// Apply Schema migrations
    pub(crate) fn apply_schema_migrations(&self) -> Result<()> {
        let conn = lock_conn!(self.conn);
        Self::apply_schema_migrations_on_conn(&conn)
    }

    /// Apply Schema migrations on a specific connection
    pub(crate) fn apply_schema_migrations_on_conn(conn: &Connection) -> Result<()> {
        conn.execute("SAVEPOINT schema_migration;", [])
            .map_err(|e| CoreError::Database(format!("Failed to create savepoint: {e}")))?;

        let mut version = Self::get_user_version(conn)?;

        if version > SCHEMA_VERSION {
            conn.execute("ROLLBACK TO schema_migration;", []).ok();
            conn.execute("RELEASE schema_migration;", []).ok();
            return Err(CoreError::Database(format!(
                "Database version ({version}) is newer than supported ({SCHEMA_VERSION}). Please upgrade the application."
            )));
        }

        let result = (|| {
            while version < SCHEMA_VERSION {
                match version {
                    0 => {
                        log::info!("Detected user_version=0, migrating to 1");
                        Self::migrate_v0_to_v1(conn)?;
                        Self::set_user_version(conn, 1)?;
                    }
                    1 => {
                        log::info!("Migrating database from v1 to v2");
                        Self::migrate_v1_to_v2(conn)?;
                        Self::set_user_version(conn, 2)?;
                    }
                    _ => {
                        return Err(CoreError::Database(format!(
                            "Unknown database version {version}, cannot migrate to {SCHEMA_VERSION}"
                        )));
                    }
                }
                version = Self::get_user_version(conn)?;
            }
            Ok(())
        })();

        match result {
            Ok(_) => {
                conn.execute("RELEASE schema_migration;", [])
                    .map_err(|e| CoreError::Database(format!("Failed to commit migration: {e}")))?;
                Ok(())
            }
            Err(e) => {
                conn.execute("ROLLBACK TO schema_migration;", []).ok();
                conn.execute("RELEASE schema_migration;", []).ok();
                Err(e)
            }
        }
    }

    /// v0 -> v1 migration: add missing columns
    fn migrate_v0_to_v1(conn: &Connection) -> Result<()> {
        // providers table
        Self::add_column_if_missing(conn, "providers", "category", "TEXT")?;
        Self::add_column_if_missing(conn, "providers", "created_at", "INTEGER")?;
        Self::add_column_if_missing(conn, "providers", "sort_index", "INTEGER")?;
        Self::add_column_if_missing(conn, "providers", "notes", "TEXT")?;
        Self::add_column_if_missing(conn, "providers", "icon", "TEXT")?;
        Self::add_column_if_missing(conn, "providers", "icon_color", "TEXT")?;
        Self::add_column_if_missing(conn, "providers", "meta", "TEXT NOT NULL DEFAULT '{}'")?;
        Self::add_column_if_missing(
            conn,
            "providers",
            "is_current",
            "BOOLEAN NOT NULL DEFAULT 0",
        )?;

        // provider_endpoints table
        Self::add_column_if_missing(conn, "provider_endpoints", "added_at", "INTEGER")?;

        Ok(())
    }

    /// v1 -> v2 migration
    fn migrate_v1_to_v2(conn: &Connection) -> Result<()> {
        // providers table fields
        Self::add_column_if_missing(
            conn,
            "providers",
            "is_proxy_target",
            "BOOLEAN NOT NULL DEFAULT 0",
        )?;
        Ok(())
    }

    // --- Helper methods ---

    pub(crate) fn get_user_version(conn: &Connection) -> Result<i32> {
        conn.query_row("PRAGMA user_version;", [], |row| row.get(0))
            .map_err(|e| CoreError::Database(format!("Failed to read user_version: {e}")))
    }

    pub(crate) fn set_user_version(conn: &Connection, version: i32) -> Result<()> {
        if version < 0 {
            return Err(CoreError::Database("user_version cannot be negative".to_string()));
        }
        let sql = format!("PRAGMA user_version = {version};");
        conn.execute(&sql, [])
            .map_err(|e| CoreError::Database(format!("Failed to write user_version: {e}")))?;
        Ok(())
    }

    fn validate_identifier(s: &str, kind: &str) -> Result<()> {
        if s.is_empty() {
            return Err(CoreError::Database(format!("{kind} cannot be empty")));
        }
        if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(CoreError::Database(format!(
                "Invalid {kind}: {s}, only letters, numbers and underscores allowed"
            )));
        }
        Ok(())
    }

    pub(crate) fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
        Self::validate_identifier(table, "table name")?;

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .map_err(|e| CoreError::Database(format!("Failed to read table names: {e}")))?;
        let mut rows = stmt
            .query([])
            .map_err(|e| CoreError::Database(format!("Failed to query table names: {e}")))?;
        while let Some(row) = rows.next().map_err(|e| CoreError::Database(e.to_string()))? {
            let name: String = row
                .get(0)
                .map_err(|e| CoreError::Database(format!("Failed to parse table name: {e}")))?;
            if name.eq_ignore_ascii_case(table) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn has_column(
        conn: &Connection,
        table: &str,
        column: &str,
    ) -> Result<bool> {
        Self::validate_identifier(table, "table name")?;
        Self::validate_identifier(column, "column name")?;

        let sql = format!("PRAGMA table_info(\"{table}\");");
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CoreError::Database(format!("Failed to read table info: {e}")))?;
        let mut rows = stmt
            .query([])
            .map_err(|e| CoreError::Database(format!("Failed to query table info: {e}")))?;
        while let Some(row) = rows.next().map_err(|e| CoreError::Database(e.to_string()))? {
            let name: String = row
                .get(1)
                .map_err(|e| CoreError::Database(format!("Failed to read column name: {e}")))?;
            if name.eq_ignore_ascii_case(column) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn add_column_if_missing(
        conn: &Connection,
        table: &str,
        column: &str,
        definition: &str,
    ) -> Result<bool> {
        Self::validate_identifier(table, "table name")?;
        Self::validate_identifier(column, "column name")?;

        if !Self::table_exists(conn, table)? {
            return Err(CoreError::Database(format!(
                "Table {table} does not exist, cannot add column {column}"
            )));
        }
        if Self::has_column(conn, table, column)? {
            return Ok(false);
        }

        let sql = format!("ALTER TABLE \"{table}\" ADD COLUMN \"{column}\" {definition};");
        conn.execute(&sql, [])
            .map_err(|e| CoreError::Database(format!("Failed to add column {column} to {table}: {e}")))?;
        log::info!("Added missing column {column} to table {table}");
        Ok(true)
    }
}
