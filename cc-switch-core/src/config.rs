//! Configuration utilities for CC Switch
//!
//! Provides path resolution and config file I/O without Tauri dependencies.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{CoreError, Result};

/// Application type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    Claude,
    Codex,
    Gemini,
}

impl AppType {
    pub fn as_str(&self) -> &str {
        match self {
            AppType::Claude => "claude",
            AppType::Codex => "codex",
            AppType::Gemini => "gemini",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(AppType::Claude),
            "codex" => Some(AppType::Codex),
            "gemini" => Some(AppType::Gemini),
            _ => None,
        }
    }

    /// Get all app types
    pub fn all() -> &'static [AppType] {
        &[AppType::Claude, AppType::Codex, AppType::Gemini]
    }
}

impl std::fmt::Display for AppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Get CC Switch app config directory (~/.cc-switch)
pub fn get_app_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot get home directory")
        .join(".cc-switch")
}

/// Get CC Switch config file path (~/.cc-switch/config.json)
pub fn get_app_config_path() -> PathBuf {
    get_app_config_dir().join("config.json")
}

/// Get database path (~/.cc-switch/cc-switch.db)
pub fn get_database_path() -> PathBuf {
    get_app_config_dir().join("cc-switch.db")
}

/// Get Claude Code config directory (~/.claude)
pub fn get_claude_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot get home directory")
        .join(".claude")
}

/// Get Claude MCP config path (~/.claude.json)
pub fn get_claude_mcp_path() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot get home directory")
        .join(".claude.json")
}

/// Get Claude settings path (~/.claude/settings.json)
pub fn get_claude_settings_path() -> PathBuf {
    let dir = get_claude_config_dir();
    let settings = dir.join("settings.json");
    if settings.exists() {
        return settings;
    }
    // Compatibility: use legacy file if exists
    let legacy = dir.join("claude.json");
    if legacy.exists() {
        return legacy;
    }
    settings
}

/// Get Codex config directory (~/.codex)
pub fn get_codex_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot get home directory")
        .join(".codex")
}

/// Get Gemini config directory (~/.gemini)
pub fn get_gemini_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot get home directory")
        .join(".gemini")
}

/// Get config directory for the given app type
pub fn get_config_dir_for_app(app: &AppType) -> PathBuf {
    match app {
        AppType::Claude => get_claude_config_dir(),
        AppType::Codex => get_codex_config_dir(),
        AppType::Gemini => get_gemini_config_dir(),
    }
}

/// Read JSON configuration file
pub fn read_json_file<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Err(CoreError::Config(format!(
            "File does not exist: {}",
            path.display()
        )));
    }

    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(CoreError::from)
}

/// Write JSON configuration file
pub fn write_json_file<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(data)?;
    atomic_write(path, json.as_bytes())
}

/// Write text file (for TOML/plain text)
pub fn write_text_file(path: &Path, data: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    atomic_write(path, data.as_bytes())
}

/// Atomic write: write to temp file then rename to avoid partial writes
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let parent = path
        .parent()
        .ok_or_else(|| CoreError::Config("Invalid path".to_string()))?;
    let mut tmp = parent.to_path_buf();
    let file_name = path
        .file_name()
        .ok_or_else(|| CoreError::Config("Invalid filename".to_string()))?
        .to_string_lossy()
        .to_string();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    tmp.push(format!("{file_name}.tmp.{ts}"));

    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(data)?;
        f.flush()?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let perm = meta.permissions().mode();
            let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(perm));
        }
    }

    #[cfg(windows)]
    {
        // Windows rename fails if target exists, remove first
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    fs::rename(&tmp, path)?;
    Ok(())
}

/// Copy file
pub fn copy_file(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to).map_err(|e| CoreError::Io(e))?;
    Ok(())
}

/// Delete file
pub fn delete_file(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
