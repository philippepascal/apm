use anyhow::Result;
use apm_core::{archive, clean, config::Config};
use std::path::Path;

pub fn run(root: &Path, dry_run: bool, older_than: Option<String>) -> Result<()> {
    let config = Config::load(root)?;

    let threshold = older_than
        .as_deref()
        .map(clean::parse_older_than)
        .transpose()?;

    archive::archive(root, &config, dry_run, threshold)
}
