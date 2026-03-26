use anyhow::Result;
use apm_core::config::Config;
use std::path::Path;

pub fn run(root: &Path) -> Result<()> {
    let config = Config::load(root)?;
    match config.agents.instructions {
        None => println!("No instructions file configured in [agents] instructions."),
        Some(rel_path) => {
            let path = root.join(&rel_path);
            match std::fs::read_to_string(&path) {
                Ok(contents) => print!("{}", contents),
                Err(e) => anyhow::bail!("cannot read {}: {}", path.display(), e),
            }
        }
    }
    Ok(())
}
