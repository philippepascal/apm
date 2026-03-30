use anyhow::Result;
use apm_core::{clean, config::Config};
use std::path::Path;

pub fn run(root: &Path, dry_run: bool) -> Result<()> {
    let config = Config::load(root)?;
    let candidates = clean::candidates(root, &config)?;

    if candidates.is_empty() {
        println!("Nothing to clean.");
        return Ok(());
    }

    for candidate in &candidates {
        if dry_run {
            if let Some(ref path) = candidate.worktree {
                println!(
                    "would remove worktree {} (ticket #{}, state: {})",
                    path.display(),
                    candidate.ticket_id,
                    candidate.reason
                );
            }
            if candidate.local_branch_exists {
                println!(
                    "would remove branch {} (state: {})",
                    candidate.branch, candidate.reason
                );
            }
        } else {
            if let Some(ref path) = candidate.worktree {
                println!("removed worktree {}", path.display());
            }
            if candidate.local_branch_exists {
                println!("removed branch {}", candidate.branch);
            }
            clean::remove(root, candidate)?;
        }
    }

    Ok(())
}
