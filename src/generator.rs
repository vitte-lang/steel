use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Generator;

impl Generator {
    pub fn generate_mcfg(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(output_path)?;
        let content = format!("# Muffin Configuration\n# Generated at {}\n", chrono::Local::now());
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
