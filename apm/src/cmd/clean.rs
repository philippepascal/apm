use anyhow::Result;
use apm_core::{clean, config::Config, git};
use std::io::IsTerminal;
use std::path::Path;

pub fn run(
    root: &Path,
    dry_run: bool,
    yes: bool,
    force: bool,
    branches: bool,
    remote: bool,
    older_than: Option<String>,
    untracked: bool,
) -> Result<()> {
    // Validate flag combinations.
    if remote && older_than.is_none() {
        anyhow::bail!("--remote requires --older-than <THRESHOLD>");
    }

    let config = Config::load(root)?;
    let (candidates, dirty) = clean::candidates(root, &config, force, untracked, dry_run)?;

    if candidates.is_empty() && dirty.is_empty() && !remote {
        println!("Nothing to clean.");
        return Ok(());
    }

    // Warn about dirty worktrees that can't be auto-cleaned.
    for dw in &dirty {
        if !dw.modified_tracked.is_empty() {
            for f in &dw.modified_tracked {
                eprintln!("  M {}", f.display());
            }
            eprintln!(
                "warning: {} has modified tracked files — manual cleanup required — skipping",
                dw.branch
            );
        } else {
            for f in &dw.other_untracked {
                eprintln!("  ? {}", f.display());
            }
            eprintln!(
                "warning: {} has untracked files — re-run with --untracked to remove — skipping",
                dw.branch
            );
        }
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
            if branches && candidate.local_branch_exists && (candidate.branch_merged || force) {
                println!(
                    "would remove branch {} (state: {})",
                    candidate.branch, candidate.reason
                );
            } else if branches && candidate.local_branch_exists && !candidate.branch_merged {
                println!(
                    "would keep branch {} (not merged into main)",
                    candidate.branch
                );
            }
        } else if force {
            eprintln!(
                "warning: force-removing {} — branch may not be merged",
                candidate.branch
            );
            eprint!("Force-remove {}? [y/N] ", candidate.branch);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().eq_ignore_ascii_case("y") {
                if let Some(ref path) = candidate.worktree {
                    println!("removed worktree {}", path.display());
                }
                if branches && candidate.local_branch_exists {
                    println!("removed branch {}", candidate.branch);
                }
                clean::remove(root, candidate, true, branches)?;
            } else {
                eprintln!("skipping {}", candidate.branch);
            }
        } else {
            if let Some(ref path) = candidate.worktree {
                println!("removed worktree {}", path.display());
            }
            if branches && candidate.local_branch_exists && candidate.branch_merged {
                println!("removed branch {}", candidate.branch);
            } else if branches && candidate.local_branch_exists && !candidate.branch_merged {
                println!("kept branch {} (not merged into main)", candidate.branch);
            }
            clean::remove(root, candidate, false, branches)?;
        }
    }

    // --remote --older-than path.
    if remote {
        let threshold_str = older_than.as_deref().unwrap();
        let threshold = clean::parse_older_than(threshold_str)?;
        let remote_candidates = clean::remote_candidates(root, &config, threshold)?;

        if remote_candidates.is_empty() {
            println!("No remote branches to clean.");
        }

        for rc in &remote_candidates {
            if dry_run {
                println!(
                    "would delete remote branch {} (last commit: {})",
                    rc.branch,
                    rc.last_commit.format("%Y-%m-%d")
                );
                continue;
            }
            let should_delete = if yes {
                true
            } else if std::io::stdout().is_terminal() {
                eprint!(
                    "Delete remote branch {} (last commit: {})? [y/N] ",
                    rc.branch,
                    rc.last_commit.format("%Y-%m-%d")
                );
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.trim().eq_ignore_ascii_case("y")
            } else {
                eprintln!(
                    "skipping {} — non-interactive (use --yes to auto-confirm)",
                    rc.branch
                );
                false
            };
            if should_delete {
                git::delete_remote_branch(root, &rc.branch)?;
                println!("deleted remote branch {}", rc.branch);
            }
        }
    }

    Ok(())
}
