use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::path::Path;

pub fn run(root: &Path, skip_permissions: bool, dry_run: bool) -> Result<()> {
    let config = Config::load(root)?;
    let max_concurrent = config.agents.max_concurrent.max(1);

    if dry_run {
        return run_dry(root, &config);
    }

    let mut workers: Vec<(u32, std::process::Child)> = Vec::new();
    let mut started_ids: Vec<u32> = Vec::new();
    let mut no_more = false;

    loop {
        // Reap finished workers.
        workers.retain_mut(|(_, child)| {
            !matches!(child.try_wait(), Ok(Some(_)))
        });

        if no_more && workers.is_empty() {
            break;
        }

        if !no_more && workers.len() < max_concurrent {
            match super::start::spawn_next_worker(root, true, skip_permissions) {
                Ok(None) => { no_more = true; }
                Ok(Some((id, child))) => {
                    started_ids.push(id);
                    workers.push((id, child));
                }
                Err(e) => {
                    eprintln!("warning: dispatch failed: {e:#}");
                    no_more = true;
                }
            }
        } else {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    if started_ids.is_empty() {
        println!("No tickets to work.");
        return Ok(());
    }

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let good_states = ["implemented", "specd"];
    let mut any_bad = false;
    println!("\nSummary:");
    for id in &started_ids {
        if let Some(t) = tickets.iter().find(|t| &t.frontmatter.id == id) {
            let state = &t.frontmatter.state;
            let ok = good_states.contains(&state.as_str());
            if !ok { any_bad = true; }
            println!("  #{id} {} — {state}", t.frontmatter.title);
        }
    }

    if any_bad {
        std::process::exit(1);
    }
    Ok(())
}

fn run_dry(root: &Path, config: &Config) -> Result<()> {
    let pw = config.workflow.prioritization.priority_weight;
    let ew = config.workflow.prioritization.effort_weight;
    let rw = config.workflow.prioritization.risk_weight;

    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();
    let actionable = config.actionable_states_for("agent");

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let mut candidates: Vec<_> = tickets.iter()
        .filter(|t| {
            let fm = &t.frontmatter;
            fm.agent.is_none()
                && actionable.contains(&fm.state.as_str())
                && (startable.is_empty() || startable.contains(&fm.state.as_str()))
        })
        .collect();
    candidates.sort_by(|a, b| {
        b.score(pw, ew, rw)
            .partial_cmp(&a.score(pw, ew, rw))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if candidates.is_empty() {
        println!("dry-run: no actionable tickets");
        return Ok(());
    }

    println!("dry-run: would start {} ticket(s):", candidates.len());
    for t in &candidates {
        println!("  #{} [{}] {}", t.frontmatter.id, t.frontmatter.state, t.frontmatter.title);
    }
    Ok(())
}
