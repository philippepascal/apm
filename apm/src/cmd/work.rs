use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::path::Path;

const IDLE_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

pub fn run(root: &Path, skip_permissions: bool, dry_run: bool) -> Result<()> {
    let config = Config::load(root)?;
    let max_concurrent = config.agents.max_concurrent.max(1);

    if dry_run {
        return run_dry(root, &config);
    }

    let mut workers: Vec<(String, std::process::Child, std::path::PathBuf)> = Vec::new();
    let mut started_ids: Vec<String> = Vec::new();
    let mut last_poll_empty = false;

    loop {
        // Reap finished workers.
        let before = workers.len();
        workers.retain_mut(|(_, child, pid_path)| {
            let done = matches!(child.try_wait(), Ok(Some(_)));
            if done {
                let _ = std::fs::remove_file(pid_path);
            }
            !done
        });
        if workers.len() < before {
            last_poll_empty = false;
        }

        if workers.is_empty() && last_poll_empty {
            break;
        }

        if workers.len() < max_concurrent {
            match super::start::spawn_next_worker(root, true, skip_permissions) {
                Ok(Some((id, child, pid_path))) => {
                    started_ids.push(id.clone());
                    workers.push((id, child, pid_path));
                    last_poll_empty = false;
                }
                Ok(None) => {
                    last_poll_empty = true;
                    std::thread::sleep(IDLE_POLL_INTERVAL);
                }
                Err(e) => {
                    eprintln!("warning: dispatch failed: {e:#}");
                    last_poll_empty = true;
                    std::thread::sleep(IDLE_POLL_INTERVAL);
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
    let good_states: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();
    let mut any_bad = false;
    println!("\nSummary:");
    for id in &started_ids {
        if let Some(t) = tickets.iter().find(|t| t.frontmatter.id == *id) {
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
    let max_concurrent = config.agents.max_concurrent.max(1);

    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();
    let actionable = config.actionable_states_for("agent");

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let mut candidates: Vec<&ticket::Ticket> = tickets
        .iter()
        .filter(|t| {
            let state = t.frontmatter.state.as_str();
            actionable.contains(&state) && (startable.is_empty() || startable.contains(&state))
        })
        .collect();
    candidates.sort_by(|a, b| {
        b.score(pw, ew, rw)
            .partial_cmp(&a.score(pw, ew, rw))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if candidates.is_empty() {
        println!("dry-run: no actionable tickets");
    } else {
        for t in candidates.into_iter().take(max_concurrent) {
            println!(
                "dry-run: would start next: #{} [{}] {}",
                t.frontmatter.id, t.frontmatter.state, t.frontmatter.title
            );
        }
    }
    Ok(())
}
