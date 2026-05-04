use anyhow::Result;
use apm_core::clean;
use std::path::Path;
use crate::ctx::CmdContext;

#[allow(clippy::too_many_arguments)]
pub fn run(
    root: &Path,
    dry_run: bool,
    _yes: bool,
    force: bool,
    branches: bool,
    older_than: Option<String>,
    untracked: bool,
    epics: bool,
) -> Result<()> {
    let config = CmdContext::load_config_only(root)?;
    let (mut candidates, dirty, candidate_warnings) = clean::candidates(root, &config, force, untracked, dry_run)?;
    for w in &candidate_warnings {
        eprintln!("{w}");
    }

    // When --branches, also enumerate remote-only ticket branches (origin
    // has them, no local head) whose ticket on the default branch is in a
    // terminal state. These are common after an earlier clean removed
    // local branches but the remote was still up.
    if branches {
        let local_branch_set: std::collections::HashSet<String> =
            candidates.iter().map(|c| c.branch.clone()).collect();
        candidates.extend(clean::remote_only_candidates(root, &config, &local_branch_set)?);
    }

    // Apply --older-than filter (if set) by ticket frontmatter updated_at.
    // Tickets with no updated_at are conservatively kept (we can't verify age).
    if let Some(threshold_str) = older_than.as_deref() {
        let threshold = clean::parse_older_than(threshold_str)?;
        candidates.retain(|c| match c.updated_at {
            Some(ts) => ts < threshold,
            None => false,
        });
    }

    // Refuse to remove any worktree that contains the current working directory.
    // Check both clean candidates and dirty candidates (dirty worktrees are skipped later,
    // but we must still refuse if the caller is inside one of them).
    let cwd = std::env::current_dir().unwrap_or_default();
    let canonical_cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.clone());
    for candidate in &candidates {
        if let Some(ref wt_path) = candidate.worktree {
            let canonical_wt = wt_path.canonicalize().unwrap_or_else(|_| wt_path.clone());
            if canonical_cwd.starts_with(&canonical_wt) {
                eprintln!(
                    "refusing to remove worktree containing the current working directory: {}",
                    wt_path.display()
                );
                anyhow::bail!(
                    "refusing to remove worktree containing the current working directory: {}",
                    wt_path.display()
                );
            }
        }
    }
    for dw in &dirty {
        let canonical_wt = dw.path.canonicalize().unwrap_or_else(|_| dw.path.clone());
        if canonical_cwd.starts_with(&canonical_wt) {
            eprintln!(
                "refusing to remove worktree containing the current working directory: {}",
                dw.path.display()
            );
            anyhow::bail!(
                "refusing to remove worktree containing the current working directory: {}",
                dw.path.display()
            );
        }
    }

    if candidates.is_empty() && dirty.is_empty() && !epics {
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
        let scope = match (candidate.local_branch_exists, candidate.remote_branch_exists) {
            (true, true) => "local + remote",
            (true, false) => "local",
            (false, true) => "remote",
            (false, false) => "registry only",
        };
        if dry_run {
            if let Some(ref path) = candidate.worktree {
                println!(
                    "would remove worktree {} (ticket #{}, state: {})",
                    path.display(),
                    candidate.ticket_id,
                    candidate.reason
                );
            }
            if branches {
                println!(
                    "would remove branch {} ({}, state: {})",
                    candidate.branch, scope, candidate.reason
                );
            }
        } else if force {
            if crate::util::prompt_yes_no(&format!("Remove {}? [y/N] ", candidate.branch))? {
                if let Some(ref path) = candidate.worktree {
                    println!("removed worktree {}", path.display());
                }
                if branches {
                    println!("removed branch {} ({})", candidate.branch, scope);
                }
                let remove_out = clean::remove(root, candidate, true, branches)?;
                for w in &remove_out.warnings {
                    eprintln!("{w}");
                }
            } else {
                eprintln!("skipping {}", candidate.branch);
            }
        } else {
            if let Some(ref path) = candidate.worktree {
                println!("removed worktree {}", path.display());
            }
            if branches {
                println!("removed branch {} ({})", candidate.branch, scope);
            }
            let remove_out = clean::remove(root, candidate, false, branches)?;
            for w in &remove_out.warnings {
                eprintln!("{w}");
            }
        }
    }

    if epics {
        crate::cmd::epic::run_epic_clean(root, &config, dry_run, _yes)?;
    }

    Ok(())
}
