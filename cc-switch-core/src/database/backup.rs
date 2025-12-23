//! Database backup and restore
//!
//! Provides SQL export/import functionality for database backup.

use super::{lock_conn, Database};
use crate::config::get_app_config_dir;
use crate::error::{CoreError, Result};
use chrono::Utc;
use rusqlite::backup::Backup;
use rusqlite::types::ValueRef;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Number of database backups to retain
const DB_BACKUP_RETAIN: usize = 5;

impl Database {
    /// Export database as SQLite-compatible SQL text
    pub fn export_sql(&self, target_path: &Path) -> Result<()> {
        let snapshot = self.snapshot_to_memory()?;
        let dump = Self::dump_sql(&snapshot)?;

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        crate::config::atomic_write(target_path, dump.as_bytes())
    }

    /// Import from SQL file, returns backup ID (empty string if no backup was made)
    pub fn import_sql(&self, source_path: &Path) -> Result<String> {
        if !source_path.exists() {
            return Err(CoreError::Config(format!(
                "SQL file not found: {}",
                source_path.display()
            )));
        }

        let sql_raw = fs::read_to_string(source_path)?;
        let sql_content = Self::sanitize_import_sql(&sql_raw);

        // Backup existing database before import
        let backup_path = self.backup_database_file()?;

        // Execute import in a temp database to avoid polluting main db on failure
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_path_buf();
        let temp_conn = Connection::open(&temp_path)
            .map_err(|e| CoreError::Database(e.to_string()))?;

        temp_conn
            .execute_batch(&sql_content)
            .map_err(|e| CoreError::Database(format!("SQL import failed: {e}")))?;

        // Apply missing tables/indexes and basic validation
        Self::create_tables_on_conn(&temp_conn)?;
        Self::apply_schema_migrations_on_conn(&temp_conn)?;
        Self::validate_basic_state(&temp_conn)?;

        // Atomically write temp db back to main db using Backup
        {
            let mut main_conn = lock_conn!(self.conn);
            let backup = Backup::new(&temp_conn, &mut main_conn)
                .map_err(|e| CoreError::Database(e.to_string()))?;
            backup
                .step(-1)
                .map_err(|e| CoreError::Database(e.to_string()))?;
        }

        let backup_id = backup_path
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_default();

        Ok(backup_id)
    }

    /// Create in-memory snapshot to avoid holding db lock for long
    fn snapshot_to_memory(&self) -> Result<Connection> {
        let conn = lock_conn!(self.conn);
        let mut snapshot = Connection::open_in_memory()
            .map_err(|e| CoreError::Database(e.to_string()))?;

        {
            let backup = Backup::new(&conn, &mut snapshot)
                .map_err(|e| CoreError::Database(e.to_string()))?;
            backup
                .step(-1)
                .map_err(|e| CoreError::Database(e.to_string()))?;
        }

        Ok(snapshot)
    }

    /// Remove SQLite reserved object statements (like sqlite_sequence) to avoid import errors
    fn sanitize_import_sql(sql: &str) -> String {
        let mut cleaned = String::new();
        let lower_keyword = "sqlite_sequence";

        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.to_ascii_lowercase().contains(lower_keyword) {
                continue;
            }

            cleaned.push_str(trimmed);
            cleaned.push_str(";\n");
        }

        cleaned
    }

    /// Create consistent snapshot backup, returns backup file path (None if main db doesn't exist)
    fn backup_database_file(&self) -> Result<Option<PathBuf>> {
        let db_path = get_app_config_dir().join("cc-switch.db");
        if !db_path.exists() {
            return Ok(None);
        }

        let backup_dir = db_path
            .parent()
            .ok_or_else(|| CoreError::Config("Invalid database path".to_string()))?
            .join("backups");

        fs::create_dir_all(&backup_dir)?;

        let backup_id = format!("db_backup_{}", Utc::now().format("%Y%m%d_%H%M%S"));
        let backup_path = backup_dir.join(format!("{backup_id}.db"));

        {
            let conn = lock_conn!(self.conn);
            let mut dest_conn = Connection::open(&backup_path)
                .map_err(|e| CoreError::Database(e.to_string()))?;
            let backup = Backup::new(&conn, &mut dest_conn)
                .map_err(|e| CoreError::Database(e.to_string()))?;
            backup
                .step(-1)
                .map_err(|e| CoreError::Database(e.to_string()))?;
        }

        Self::cleanup_db_backups(&backup_dir)?;
        Ok(Some(backup_path))
    }

    /// Clean up old database backups, keeping only the newest N
    fn cleanup_db_backups(dir: &Path) -> Result<()> {
        let entries = match fs::read_dir(dir) {
            Ok(iter) => iter
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .map(|ext| ext == "db")
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>(),
            Err(_) => return Ok(()),
        };

        if entries.len() <= DB_BACKUP_RETAIN {
            return Ok(());
        }

        let remove_count = entries.len().saturating_sub(DB_BACKUP_RETAIN);
        let mut sorted = entries;
        sorted.sort_by_key(|entry| entry.metadata().and_then(|m| m.modified()).ok());

        for entry in sorted.into_iter().take(remove_count) {
            if let Err(err) = fs::remove_file(entry.path()) {
                eprintln!("Warning: failed to remove old backup {}: {}", entry.path().display(), err);
            }
        }
        Ok(())
    }

    /// Basic state validation
    fn validate_basic_state(conn: &Connection) -> Result<()> {
        let provider_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM providers", [], |row| row.get(0))
            .unwrap_or(0);

        // Allow empty databases - just warn
        if provider_count == 0 {
            eprintln!("Warning: imported SQL contains no provider data");
        }
        Ok(())
    }

    /// Export database as SQL text
    fn dump_sql(conn: &Connection) -> Result<String> {
        let mut output = String::new();
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let user_version: i64 = conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .unwrap_or(0);

        output.push_str(&format!(
            "-- CC Switch SQLite Export\n-- Generated: {timestamp}\n-- user_version: {user_version}\n"
        ));
        output.push_str("PRAGMA foreign_keys=OFF;\n");
        output.push_str(&format!("PRAGMA user_version={user_version};\n"));
        output.push_str("BEGIN TRANSACTION;\n");

        // Export schema
        let mut stmt = conn
            .prepare(
                "SELECT type, name, tbl_name, sql
                 FROM sqlite_master
                 WHERE sql NOT NULL AND type IN ('table','index','trigger','view')
                 ORDER BY type='table' DESC, name",
            )
            .map_err(|e| CoreError::Database(e.to_string()))?;

        let mut tables = Vec::new();
        let mut rows = stmt
            .query([])
            .map_err(|e| CoreError::Database(e.to_string()))?;
        while let Some(row) = rows.next().map_err(|e| CoreError::Database(e.to_string()))? {
            let obj_type: String = row.get(0).map_err(|e| CoreError::Database(e.to_string()))?;
            let name: String = row.get(1).map_err(|e| CoreError::Database(e.to_string()))?;
            let sql: String = row.get(3).map_err(|e| CoreError::Database(e.to_string()))?;

            // Skip SQLite internal objects (like sqlite_sequence)
            if name.starts_with("sqlite_") {
                continue;
            }

            output.push_str(&sql);
            output.push_str(";\n");

            if obj_type == "table" && !name.starts_with("sqlite_") {
                tables.push(name);
            }
        }

        // Export data
        for table in tables {
            let columns = Self::get_table_columns(conn, &table)?;
            if columns.is_empty() {
                continue;
            }

            let mut stmt = conn
                .prepare(&format!("SELECT * FROM \"{table}\""))
                .map_err(|e| CoreError::Database(e.to_string()))?;
            let mut rows = stmt
                .query([])
                .map_err(|e| CoreError::Database(e.to_string()))?;

            while let Some(row) = rows.next().map_err(|e| CoreError::Database(e.to_string()))? {
                let mut values = Vec::with_capacity(columns.len());
                for idx in 0..columns.len() {
                    let value = row
                        .get_ref(idx)
                        .map_err(|e| CoreError::Database(e.to_string()))?;
                    values.push(Self::format_sql_value(value)?);
                }

                let cols = columns
                    .iter()
                    .map(|c| format!("\"{c}\""))
                    .collect::<Vec<_>>()
                    .join(", ");
                output.push_str(&format!(
                    "INSERT INTO \"{table}\" ({cols}) VALUES ({});\n",
                    values.join(", ")
                ));
            }
        }

        output.push_str("COMMIT;\nPRAGMA foreign_keys=ON;\n");
        Ok(output)
    }

    /// Get table column names
    fn get_table_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info(\"{table}\")"))
            .map_err(|e| CoreError::Database(e.to_string()))?;
        let iter = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| CoreError::Database(e.to_string()))?;

        let mut columns = Vec::new();
        for col in iter {
            columns.push(col.map_err(|e| CoreError::Database(e.to_string()))?);
        }
        Ok(columns)
    }

    /// Format SQL value
    fn format_sql_value(value: ValueRef<'_>) -> Result<String> {
        match value {
            ValueRef::Null => Ok("NULL".to_string()),
            ValueRef::Integer(i) => Ok(i.to_string()),
            ValueRef::Real(f) => Ok(f.to_string()),
            ValueRef::Text(t) => {
                let text = std::str::from_utf8(t)
                    .map_err(|e| CoreError::Database(format!("Text field is not valid UTF-8: {e}")))?;
                let escaped = text.replace('\'', "''");
                Ok(format!("'{escaped}'"))
            }
            ValueRef::Blob(bytes) => {
                let mut s = String::from("X'");
                for b in bytes {
                    use std::fmt::Write;
                    let _ = write!(&mut s, "{b:02X}");
                }
                s.push('\'');
                Ok(s)
            }
        }
    }
}
