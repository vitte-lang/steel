use super::schema::McfgConfig;

pub struct McfgValidator;

impl McfgValidator {
    pub fn validate(config: &McfgConfig) -> Result<(), String> {
        if config.metadata.name.is_empty() {
            return Err("Package name is required".to_string());
        }

        if config.metadata.version.is_empty() {
            return Err("Version is required".to_string());
        }

        for target in &config.targets {
            if target.name.is_empty() {
                return Err("Target name cannot be empty".to_string());
            }
            if target.commands.is_empty() {
                return Err(format!("Target '{}' has no commands", target.name));
            }
        }

        Ok(())
    }
}
