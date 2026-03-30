use anyhow::Result;
use apm_core::{config::Config, git, ticket};
use std::path::Path;

pub fn run(root: &Path, json: bool, no_aggressive: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    if aggressive {
        if let Err(e) = git::fetch_all(root) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let actionable = config.actionable_states_for("agent");
    let p = &config.workflow.prioritization;

    match ticket::pick_next(&tickets, &actionable, &[], p.priority_weight, p.effort_weight, p.risk_weight) {
        None => {
            if json {
                println!("null");
            } else {
                println!("No actionable tickets.");
            }
        }
        Some(t) => {
            let fm = &t.frontmatter;
            if json {
                println!(
                    r#"{{"id":{:?}, "title":{:?}, "state":{:?}, "score":{}}}"#,
                    fm.id, fm.title, fm.state, t.score(p.priority_weight, p.effort_weight, p.risk_weight)
                );
            } else {
                println!("{} [{}] {}", fm.id, fm.state, fm.title);
            }
        }
    }
    Ok(())
}
