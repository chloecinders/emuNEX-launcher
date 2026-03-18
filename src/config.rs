use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Default)]
pub struct LauncherConfig {
    pub repository: Option<String>,
    pub github_token: Option<String>,
}

impl LauncherConfig {
    pub fn load() -> Result<Self, String> {
        let config_path = std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("Config.toml");

        if !config_path.exists() {
            return Ok(LauncherConfig::default());
        }

        let text = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read Config.toml: {e}"))?;

        toml::from_str(&text).map_err(|e| format!("Failed to parse Config.toml: {e}"))
    }
}
