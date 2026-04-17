use anyhow::Result;
use std::path::Path;
use apm_core::config::Config;

pub fn run(root: &Path, ticket_id: &str, target: &str) -> Result<()> {
    let config = Config::load(root)?;
    let target_opt = if target == "-" { None } else { Some(target) };
    let msg = apm_core::ticket::move_to_epic(root, &config, ticket_id, target_opt)?;
    println!("{msg}");
    Ok(())
}
