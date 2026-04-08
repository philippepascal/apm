use anyhow::{bail, Result};
use std::path::Path;

pub fn resolve() -> String {
    std::env::var("VISUAL")
        .ok()
        .filter(|e| !e.is_empty())
        .or_else(|| std::env::var("EDITOR").ok().filter(|e| !e.is_empty()))
        .unwrap_or_else(|| "vi".to_string())
}

pub fn open(path: &Path) -> Result<()> {
    let editor = resolve();
    let mut parts = editor.split_whitespace();
    let bin = parts.next().unwrap();
    let status = std::process::Command::new(bin)
        .args(parts)
        .arg(path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("could not launch editor '{editor}': {e}"))?;

    if !status.success() {
        bail!("editor exited with non-zero status");
    }
    Ok(())
}
