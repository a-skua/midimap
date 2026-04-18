use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub port: Option<String>,
    #[serde(default, rename = "map")]
    pub mappings: Vec<MappingConfig>,
}

#[derive(Debug, Deserialize)]
pub struct MappingConfig {
    pub note: Option<u8>,
    pub cc: Option<u8>,
    /// MIDI channel 1-16; omit to match any channel
    pub channel: Option<u8>,
    /// For CC: only trigger when value >= min_value
    pub min_value: Option<u8>,
    /// Key combo string, e.g. "cmd+c", "ctrl+shift+z", "f5"
    pub keys: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read '{}': {}", path, e))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Invalid config: {}", e))?;
        Ok(config)
    }
}
