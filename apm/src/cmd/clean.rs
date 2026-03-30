use anyhow::Result;
use apm_core::{clean, clean::CleanCandidate, config::Config};
use std::io::IsTerminal;
use std::path::Path;

pub fn run(root: &Path, dry_run: bool, yes: bool) -> Result<()> {
    let config = Config::load(root)?;
    let (candidates, dirty) = clean::candidates(root, &config)?;

    if candidates.is_empty() && dirty.is_empty() {
        println!("Nothing to clean.");
        return Ok(());
    }

    for dw in &dirty {
        if !dw.modified_tracked.is_empty() {
            for f in &dw.modified_tracked {
                eprintln!("  M {}", f.display());
            }
            eprintln!(
                "warning: {} has modified tracked files — manual cleanup required — skipping",
                dw.branch
            );
            continue;
        }

        for f in &dw.known_temp {
            println!("  [temp] {}", f.display());
        }
        for f in &dw.other_untracked {
            println!("  [user] {}", f.display());
        }

        let n = dw.known_temp.len() + dw.other_untracked.len();

        if dry_run {
            println!(
                "would remove {} file(s) — re-run without --dry-run to be prompted",
                n
            );
            continue;
        }

        let mut all_files: Vec<std::path::PathBuf> = Vec::new();
        all_files.extend_from_slice(&dw.known_temp);
        all_files.extend_from_slice(&dw.other_untracked);

        if yes {
            clean::remove_untracked(&dw.path, &all_files)?;
            let candidate = CleanCandidate {
                ticket_id: dw.ticket_id.clone(),
                ticket_title: dw.ticket_title.clone(),
                branch: dw.branch.clone(),
                worktree: Some(dw.path.clone()),
                reason: String::new(),
                local_branch_exists: dw.local_branch_exists,
            };
            println!("removed worktree {}", dw.path.display());
            if candidate.local_branch_exists {
                println!("removed branch {}", candidate.branch);
            }
            clean::remove(root, &candidate)?;
        } else if std::io::stdout().is_terminal() {
            eprint!("Remove {} file(s) and clean? [y/N] ", n);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().eq_ignore_ascii_case("y") {
                clean::remove_untracked(&dw.path, &all_files)?;
                let candidate = CleanCandidate {
                    ticket_id: dw.ticket_id.clone(),
                    ticket_title: dw.ticket_title.clone(),
                    branch: dw.branch.clone(),
                    worktree: Some(dw.path.clone()),
                    reason: String::new(),
                    local_branch_exists: dw.local_branch_exists,
                };
                println!("removed worktree {}", dw.path.display());
                if candidate.local_branch_exists {
                    println!("removed branch {}", candidate.branch);
                }
                clean::remove(root, &candidate)?;
            } else {
                eprintln!("skipping {}", dw.branch);
            }
        } else {
            eprintln!(
                "skipping {} — untracked files present (use --yes to auto-remove)",
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
