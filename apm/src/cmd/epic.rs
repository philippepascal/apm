use anyhow::Result;
use std::path::Path;

pub fn run_new(root: &Path, title: String) -> Result<()> {
    let branch = apm_core::epic::create(root, &title)?;
    println!("{branch}");
    Ok(())
}
