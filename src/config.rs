use crate::theme::ThemeVariant;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeVariant,

    #[serde(default)]
    pub skip_confirm_dialog: bool,

    #[serde(default)]
    pub animation_duration_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeVariant::Dark,
            skip_confirm_dialog: false,
            animation_duration_ms: 1000,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        match Self::config_path() {
            Some(path) => {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(config) = toml::from_str(&contents) {
                        return config;
                    }
                }
            }
            None => {}
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Err("Could not determine config directory".into()),
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        let mut path = dirs::config_dir()?;
        path.push("portzap");
        path.push("config.toml");
        Some(path)
    }
}
