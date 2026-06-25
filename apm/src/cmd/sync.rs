use anyhow::Result;
use apm_core::{config::Config, git, sync};
use std::io::IsTerminal;
use std::path::Path;

pub fn run(root: &Path, offline: bool, quiet: bool, no_aggressive: bool, auto_close: bool, push_default: bool, push_refs: bool) -> Result<()> {
    // Bail early if the repo is mid-merge, mid-rebase, or mid-cherry-pick.
    // Any sync work done in this state would compound the incomplete operation.
    // Let the user resolve the pending operation first.
    if git::detect_mid_merge_state(root).is_some() {
        eprintln!("{}", apm_core::sync_guidance::MID_MERGE_IN_PROGRESS);
        return Ok(());
    }

    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;
    let is_tty = std::io::stdin().is_terminal();

    // Hoisted so Block 2 can merge closed_branches into ahead_refs and Block 3 can push.
    let mut ahead_refs: Vec<String> = Vec::new();
    let mut default_is_ahead = false;
    let mut sync_warnings: Vec<String> = Vec::new();
    let mut wt_result: Option<git::WorktreeSyncResult> = None;

    // Block 1: network I/O and ref reconciliation (non-offline only, no push).
    if !offline {
        crate::util::fetch_if_aggressive(root, true);

        ahead_refs = git::sync_non_checked_out_refs(root, &mut sync_warnings);
        default_is_ahead = git::sync_default_branch(root, &config.project.default_branch, &mut sync_warnings);

        // Reconcile ticket worktrees: fast-forward clean Behind worktrees,
        // warn about dirty/ahead/diverged ones.
        let wt = git::sync_checked_out_worktrees(root, &mut sync_warnings);
        if !quiet {
            for (wt_path, _branch) in &wt.fast_forwarded {
                println!("fast-forwarded worktree: {}", wt_path.display());
            }
        }
        if !quiet {
            for (wt_path, branch, dirty_files) in &wt.skipped_dirty {
                let files_list = dirty_files
                    .iter()
                    .map(|f| format!("    {f}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                sync_warnings.push(
                    apm_core::sync_guidance::WORKTREE_DIRTY_SKIP
                        .replace("<path>", &wt_path.display().to_string())
                        .replace("<branch>", branch)
                        .replace("<files>", &files_list),
                );
            }
            for (wt_path, branch) in &wt.skipped_ahead {
                sync_warnings.push(
                    apm_core::sync_guidance::WORKTREE_AHEAD
                        .replace("<path>", &wt_path.display().to_string())
                        .replace("<branch>", branch),
                );
            }
            for (wt_path, branch) in &wt.skipped_diverged {
                sync_warnings.push(
                    apm_core::sync_guidance::WORKTREE_DIVERGED
                        .replace("<path>", &wt_path.display().to_string())
                        .replace("<branch>", branch),
                );
            }
        }
        wt_result = Some(wt);
    }

    // Block 2: detect and apply (unconditional — runs in offline mode too).
    let candidates = sync::detect(root, &config)?;

    let branches = git::ticket_branches(root)?;
    if !quiet {
        println!(
            "sync: {} ticket branch{} visible",
            branches.len(),
            if branches.len() == 1 { "" } else { "es" },
        );
    }

    for hint in &candidates.hints {
        eprintln!("{hint}");
    }

    if !candidates.close.is_empty() {
        let confirmed = auto_close || (!quiet && prompt_close(&candidates.close)?);
        if confirmed {
            let caller = apm_core::config::resolve_caller_name();
            let actor = format!("{}(apm-sync)", caller);
            let apply_out = sync::apply(root, &config, &candidates, &actor, aggressive)?;
            for (id, err) in &apply_out.failed {
                eprintln!("warning: could not close {id:?}: {err}");
            }
            for msg in &apply_out.messages {
                println!("{msg}");
            }
            // Merge branches that became ahead due to closure into ahead_refs so
            // Block 3 pushes them to origin in the same sync invocation.
            for branch in &apply_out.closed_branches {
                if !ahead_refs.contains(branch) {
                    ahead_refs.push(branch.clone());
                }
            }
        }
    }

    if !quiet && !candidates.epic_submit_hints.is_empty() {
        println!("\nEpics ready to submit (apm epic submit <id>):");
        for (id, title) in &candidates.epic_submit_hints {
            println!("  {id:<8}  {title}");
        }
    }
    if !quiet && !candidates.epic_close_hints.is_empty() {
        println!("\nEpics ready to close (apm epic close <id>):");
        for (id, title) in &candidates.epic_close_hints {
            println!("  {id:<8}  {title}");
        }
    }

    // Block 3: push and output (non-offline only).
    // Push appears after detect+apply so closed branches are included in the same push.
    if !offline {
        // Handle default branch push.
        if default_is_ahead {
            let should_push = push_default || (is_tty && !quiet && {
                // Remove the MAIN_AHEAD warning — if user says yes, we push instead of warning.
                let prompt = format!("push {} to origin now? [y/N] ", config.project.default_branch);
                crate::util::prompt_yes_no(&prompt)?
            });
            if should_push {
                // Remove the MAIN_AHEAD warning from sync_warnings since we're pushing.
                sync_warnings.retain(|w| !w.contains(&config.project.default_branch) || !w.contains("ahead"));
                if let Err(e) = git::push_branch(root, &config.project.default_branch) {
                    eprintln!("warning: push failed: {e:#}");
                } else if !quiet {
                    println!("pushed {} to origin", config.project.default_branch);
                }
            }
        }

        // Handle ticket/epic branch push (includes branches newly ahead due to closure).
        if !ahead_refs.is_empty() {
            let n = ahead_refs.len();
            let should_push = push_refs || (is_tty && !quiet && {
                let prompt = format!("push {n} ahead branch{} to origin now? [y/N] ", if n == 1 { "" } else { "es" });
                crate::util::prompt_yes_no(&prompt)?
            });
            if should_push {
                for branch in &ahead_refs {
                    if let Err(e) = git::push_branch(root, branch) {
                        eprintln!("warning: push {branch} failed: {e:#}");
                    }
                }
                if !quiet {
                    println!("pushed {n} ahead branch{} to origin", if n == 1 { "" } else { "es" });
                }
            }
        }

        for w in &sync_warnings {
            eprintln!("{w}");
        }

        if let Some(wt) = wt_result {
            // Worktree summary line — omit when quiet or when no worktrees were processed.
            let total_wt = wt.fast_forwarded.len()
                + wt.skipped_dirty.len()
                + wt.skipped_ahead.len()
                + wt.skipped_diverged.len();
            if !quiet && total_wt > 0 {
                let mut parts: Vec<String> = Vec::new();
                let ff = wt.fast_forwarded.len();
                if ff > 0 {
                    parts.push(format!(
                        "{ff} worktree{} fast-forwarded",
                        if ff == 1 { "" } else { "s" }
                    ));
                }
                let dirty = wt.skipped_dirty.len();
                if dirty > 0 {
                    parts.push(format!("{dirty} skipped (local changes)"));
                }
                let ad = wt.skipped_ahead.len() + wt.skipped_diverged.len();
                if ad > 0 {
                    parts.push(format!("{ad} skipped (ahead/diverged)"));
                }
                println!("worktrees: {}", parts.join(", "));
            }
        }
    }

    Ok(())
}

fn prompt_close(candidates: &[sync::CloseCandidate]) -> Result<bool> {
    println!("\nTickets ready to close:");
    for c in candidates {
        println!("  #{}  {}  ({})", c.ticket.frontmatter.id, c.ticket.frontmatter.title, c.reason);
    }
    Ok(crate::util::prompt_yes_no("\nClose all? [y/N] ")?)
}
