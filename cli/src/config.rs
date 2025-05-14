use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::CliError;

/// Configuration for the CLI
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// API key for the Exchange Rate API
    pub api_key: Option<String>,
    /// Authentication method (bearer or url)
    pub auth_method: Option<String>,
    /// Default output format (text, json, csv)
    pub default_format: Option<String>,
    /// Whether to use colored output
    pub use_color: Option<bool>,
    /// Whether to use caching
    pub use_cache: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            auth_method: Some("bearer".to_string()),
            default_format: Some("text".to_string()),
            use_color: Some(true),
            use_cache: Some(true),
        }
    }
}

impl Config {
    /// Load configuration from the default location
    pub fn load() -> Result<Self, CliError> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| CliError::UnexpectedError(format!("Failed to read config file: {}", e)))?;

        let config: Config = serde_json::from_str(&config_str).map_err(|e| {
            CliError::UnexpectedError(format!("Failed to parse config file: {}", e))
        })?;

        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<(), CliError> {
        let config_path = get_config_path()?;

        // Ensure the directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CliError::UnexpectedError(format!("Failed to create config directory: {}", e))
            })?;
        }

        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| CliError::UnexpectedError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, config_str).map_err(|e| {
            CliError::UnexpectedError(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }
}

/// Get the path to the configuration file
fn get_config_path() -> Result<PathBuf, CliError> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        CliError::UnexpectedError("Could not determine home directory".to_string())
    })?;

    let config_dir = home_dir.join(".config").join("exchangerate");
    let config_path = config_dir.join("config.json");

    Ok(config_path)
}

/// Create a new configuration file with default values
pub fn create_default_config() -> Result<(), CliError> {
    let config = Config::default();
    config.save()
}
