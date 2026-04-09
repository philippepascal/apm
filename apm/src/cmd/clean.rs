use anyhow::Result;
use apm_core::{clean, git};
use std::io::{IsTerminal, Write};
use std::path::Path;
use crate::ctx::CmdContext;

pub fn run(
    root: &Path,
    dry_run: bool,
    yes: bool,
    force: bool,
    branches: bool,
    remote: bool,
    older_than: Option<String>,
    untracked: bool,
    epics: bool,
) -> Result<()> {
    // Validate flag combinations.
    if remote && older_than.is_none() {
        anyhow::bail!("--remote requires --older-than <THRESHOLD>");
    }

    let config = CmdContext::load_config_only(root)?;
    let (candidates, dirty, candidate_warnings) = clean::candidates(root, &config, force, untracked, dry_run)?;
    for w in &candidate_warnings {
        eprintln!("{w}");
    }

    if candidates.is_empty() && dirty.is_empty() && !remote && !epics {
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
            if branches && candidate.local_branch_exists && candidate.branch_merged {
                println!("removed branch {}", candidate.branch);
            } else if branches && candidate.local_branch_exists && !candidate.branch_merged {
                println!("kept branch {} (not merged into main)", candidate.branch);
            }
            let remove_out = clean::remove(root, candidate, false, branches)?;
            for w in &remove_out.warnings {
                eprintln!("{w}");
            }
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

    if epics || remote {
        run_epic_clean(root, &config, dry_run, yes)?;
    }

    Ok(())
}

fn run_epic_clean(
    root: &Path,
    config: &apm_core::config::Config,
    dry_run: bool,
    yes: bool,
) -> Result<()> {
    // Get local epic branches.
    let local_output = std::process::Command::new("git")
        .current_dir(root)
        .args(["branch", "--list", "epic/*"])
        .output()?;

    let local_branches: Vec<String> = String::from_utf8_lossy(&local_output.stdout)
        .lines()
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    // Load all tickets.
    let tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;

    // Find epic branches whose derived state is "done".
    let mut candidates: Vec<String> = Vec::new();
    for branch in &local_branches {
        let after_prefix = branch.trim_start_matches("epic/");
        let id_end = after_prefix.find('-').unwrap_or(after_prefix.len()).min(8);
        let id = &after_prefix[..id_end];

        let epic_tickets: Vec<_> = tickets
            .iter()
            .filter(|t| t.frontmatter.epic.as_deref() == Some(id))
            .collect();

        let state_configs: Vec<&apm_core::config::StateConfig> = epic_tickets
            .iter()
            .filter_map(|t| config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
            .collect();

        if apm_core::epic::derive_epic_state(&state_configs) == "done" {
            candidates.push(branch.clone());
        }
    }

    if candidates.is_empty() {
        println!("Nothing to clean.");
        return Ok(());
    }

    // Print candidate list.
    println!("Would delete {} epic(s):", candidates.len());
    for branch in &candidates {
        let after_prefix = branch.trim_start_matches("epic/");
        let id_end = after_prefix.find('-').unwrap_or(after_prefix.len()).min(8);
        let id = &after_prefix[..id_end];
        let title = crate::cmd::epic::branch_to_title(branch);
        println!("  {id}  {title}");
    }

    if dry_run {
        println!("Dry run — no changes made.");
        return Ok(());
    }

    // Confirmation gate.
    if !yes {
        if std::io::stdout().is_terminal() {
            eprint!("Delete {} epic(s)? [y/N] ", candidates.len());
            let _ = std::io::stderr().flush();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        } else {
            println!("Skipping — non-interactive terminal. Use --yes to confirm.");
            return Ok(());
        }
    }

    // Delete each candidate.
    let epics_path = root.join(".apm").join("epics.toml");
    for branch in &candidates {
        let after_prefix = branch.trim_start_matches("epic/");
        let id_end = after_prefix.find('-').unwrap_or(after_prefix.len()).min(8);
        let id = after_prefix[..id_end].to_string();

        // Delete local branch.
        let del_local = std::process::Command::new("git")
            .current_dir(root)
            .args(["branch", "-d", branch])
            .output()?;
        if !del_local.status.success() {
            eprintln!(
                "error: failed to delete local branch {branch}: {}",
                String::from_utf8_lossy(&del_local.stderr).trim()
            );
            continue;
        }

        // Delete remote branch; suppress "remote ref does not exist".
        let del_remote = std::process::Command::new("git")
            .current_dir(root)
            .args(["push", "origin", "--delete", branch])
            .output()?;
        if !del_remote.status.success() {
            let stderr = String::from_utf8_lossy(&del_remote.stderr);
            if !stderr.contains("remote ref does not exist")
                && !stderr.contains("error: unable to delete")
            {
                eprintln!(
                    "warning: failed to delete remote {branch}: {}",
                    stderr.trim()
                );
            }
        }

        println!("deleted {branch}");

        // Remove the epic's entry from .apm/epics.toml.
        if epics_path.exists() {
            let raw = std::fs::read_to_string(&epics_path)?;
            let mut doc: toml_edit::DocumentMut = raw.parse()?;
            if doc.contains_key(&id) {
                doc.remove(&id);
                std::fs::write(&epics_path, doc.to_string())?;
            }
        }
    }

    Ok(())
}
