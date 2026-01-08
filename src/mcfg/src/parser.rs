use super::schema::McfgConfig;
use std::fs;
use std::path::Path;

pub struct McfgParser;

impl McfgParser {
    pub fn parse(path: &Path) -> Result<McfgConfig, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: McfgConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn parse_json(path: &Path) -> Result<McfgConfig, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: McfgConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn parse_yaml(path: &Path) -> Result<McfgConfig, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: McfgConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
