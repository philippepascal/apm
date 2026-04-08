use anyhow::Result;
use apm_core::{config::{CompletionStrategy, Config}, git, ticket};
use std::collections::HashSet;
use std::path::Path;

pub fn run(root: &Path, fix: bool, no_aggressive: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    if aggressive {
        if let Err(e) = git::fetch_all(root) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;

    let merged = git::merged_into_main(root, &config.project.default_branch).unwrap_or_default();
    let merged_set: HashSet<String> = merged.into_iter().collect();

    let issues = apm_core::verify::verify_tickets(&config, &tickets, &merged_set);

    // Report completion strategies configured on transitions.
    for state in &config.workflow.states {
        for tr in &state.transitions {
            let label = match &tr.completion {
                CompletionStrategy::Pr => "pr",
                CompletionStrategy::Merge => "merge",
                CompletionStrategy::Pull => "pull",
                CompletionStrategy::PrOrEpicMerge => "pr_or_epic_merge",
                CompletionStrategy::None => continue,
            };
            println!("completion: {} → {} = {label}", state.id, tr.to);
        }
    }

    if config.logging.enabled {
        let log_path = apm_core::logger::resolve_log_path(
            &config.project.name,
            config.logging.file.as_deref(),
        );
        println!("logging: {}", log_path.display());
    }

    if issues.is_empty() {
        println!("verify: no issues found");
        return Ok(());
    }

    for issue in &issues {
        println!("{issue}");
    }

    if fix {
        let merged_refs: HashSet<&str> = merged_set.iter().map(|s| s.as_str()).collect();
        apply_fixes(root, &config, &tickets, &merged_refs)?;
    }

    std::process::exit(1);
}

fn apply_fixes(
    root: &Path,
    config: &Config,
    tickets: &[apm_core::ticket::Ticket],
    merged_set: &HashSet<&str>,
) -> Result<()> {
    for t in tickets {
        let fm = &t.frontmatter;
        let Some(branch) = &fm.branch else { continue };
        if (fm.state == "in_progress" || fm.state == "implemented")
            && merged_set.contains(branch.as_str())
        {
            let id = fm.id.clone();
            let old_state = fm.state.clone();
            match apm_core::ticket::close(root, config, &id, None, "verify --fix", false) {
                Ok(msgs) => {
                    for msg in &msgs {
                        println!("{msg}");
                    }
                    println!("  fixed {id}: {old_state} → closed");
                }
                Err(e) => eprintln!("  warning: could not fix {id}: {e:#}"),
            }
        }
    }
    Ok(())
}
