use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McfgConfig {
    pub metadata: Metadata,
    pub targets: Vec<Target>,
    pub variables: std::collections::HashMap<String, String>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub dependencies: Vec<String>,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub pattern: String,
    pub recipe: Vec<String>,
}

impl McfgConfig {
    pub fn new(name: String, version: String) -> Self {
        Self {
            metadata: Metadata {
                name,
                version,
                generated_at: chrono::Local::now().to_rfc3339(),
            },
            targets: Vec::new(),
            variables: std::collections::HashMap::new(),
            rules: Vec::new(),
        }
    }
}
