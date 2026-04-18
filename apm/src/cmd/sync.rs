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

    if !offline {
        let mut sync_warnings: Vec<String> = Vec::new();
        crate::util::fetch_if_aggressive(root, true);

        let ahead_refs = git::sync_non_checked_out_refs(root, &mut sync_warnings);
        let default_is_ahead = git::sync_default_branch(root, &config.project.default_branch, &mut sync_warnings);

        // Handle default branch push.
        if default_is_ahead {
            let should_push = push_default || (is_tty && !quiet && {
                // Remove the MAIN_AHEAD warning we just pushed — if user says yes, we push instead.
                // The warning was already added; we'll print it only if user says no.
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

        // Handle ticket/epic branch push.
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
    }

    let candidates = sync::detect(root, &config)?;

    let branches = git::ticket_branches(root)?;
    if !quiet {
        println!(
            "sync: {} ticket branch{} visible",
            branches.len(),
            if branches.len() == 1 { "" } else { "es" },
        );
    }

    if !candidates.close.is_empty() {
        let confirmed = auto_close || (!quiet && prompt_close(&candidates.close)?);
        if confirmed {
            let apply_out = sync::apply(root, &config, &candidates, "apm-sync", aggressive)?;
            for (id, err) in &apply_out.failed {
                eprintln!("warning: could not close {id:?}: {err}");
            }
            for msg in &apply_out.messages {
                println!("{msg}");
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
