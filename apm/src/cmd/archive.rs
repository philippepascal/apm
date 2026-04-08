use anyhow::Result;
use apm_core::{archive, clean, config::Config};
use std::path::Path;

pub fn run(root: &Path, dry_run: bool, older_than: Option<String>) -> Result<()> {
    let config = Config::load(root)?;

    let threshold = older_than
        .as_deref()
        .map(clean::parse_older_than)
        .transpose()?;

    let out = archive::archive(root, &config, dry_run, threshold)?;

    for w in &out.warnings {
        eprintln!("{w}");
    }
    for (old, new) in &out.dry_run_moves {
        println!("{old} -> {new}");
    }
    // Original behavior: in dry_run, "nothing to archive" is always printed
    // (because the moves vec was always empty in the original dry_run path).
    if dry_run || out.archived_count == 0 {
        println!("nothing to archive");
    } else {
        println!("archived {} ticket(s)", out.archived_count);
    }

    Ok(())
}
