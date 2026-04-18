use anyhow::Result;
use apm_core::{config::Config, git, sync};
use std::path::Path;

pub fn run(root: &Path, offline: bool, quiet: bool, no_aggressive: bool, auto_close: bool) -> Result<()> {
    // Bail early if the repo is mid-merge, mid-rebase, or mid-cherry-pick.
    // Any sync work done in this state would compound the incomplete operation.
    // Let the user resolve the pending operation first.
    if git::detect_mid_merge_state(root).is_some() {
        eprintln!("{}", apm_core::sync_guidance::MID_MERGE_IN_PROGRESS);
        return Ok(());
    }

    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    if !offline {
        let mut sync_warnings: Vec<String> = Vec::new();
        crate::util::fetch_if_aggressive(root, true);
        git::sync_non_checked_out_refs(root, &mut sync_warnings);
        git::sync_default_branch(root, &config.project.default_branch, &mut sync_warnings);
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
            let caller = apm_core::config::resolve_caller_name();
            let actor = format!("{}(apm-sync)", caller);
            let apply_out = sync::apply(root, &config, &candidates, &actor, aggressive)?;
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
