// Placeholder - will be populated from src-tauri/src/config.rs

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

pub fn get_app_config_dir() -> std::path::PathBuf {
    dirs::config_dir().unwrap_or_default().join("cc-switch")
}
