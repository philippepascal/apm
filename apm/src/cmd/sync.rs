use anyhow::Result;
use std::path::Path;

pub fn run(_root: &Path) -> Result<()> {
    // TODO: poll provider for PR events, detect merged branches
    println!("sync: not yet implemented");
    Ok(())
}
