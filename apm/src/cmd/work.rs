use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::path::Path;

pub fn run(root: &Path, skip_permissions: bool, dry_run: bool) -> Result<()> {
    let config = Config::load(root)?;
    let max_concurrent = config.agents.max_concurrent.max(1);

    if dry_run {
        return run_dry(root, &config);
    }

    let mut workers: Vec<(String, std::process::Child, std::path::PathBuf)> = Vec::new();
    let mut started_ids: Vec<String> = Vec::new();
    let mut no_more = false;

    loop {
        // Reap finished workers.
        workers.retain_mut(|(_, child, pid_path)| {
            let done = matches!(child.try_wait(), Ok(Some(_)));
            if done {
                let _ = std::fs::remove_file(pid_path);
            }
            !done
        });

        if no_more && workers.is_empty() {
            break;
        }

        if !no_more && workers.len() < max_concurrent {
            match super::start::spawn_next_worker(root, true, skip_permissions) {
                Ok(None) => { no_more = true; }
                Ok(Some((id, child, pid_path))) => {
                    started_ids.push(id.clone());
                    workers.push((id, child, pid_path));
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

    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();
    let actionable = config.actionable_states_for("agent");

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    match ticket::pick_next(&tickets, &actionable, &startable, pw, ew, rw) {
        None => println!("dry-run: no actionable tickets"),
        Some(t) => println!(
            "dry-run: would start next: #{} [{}] {}",
            t.frontmatter.id, t.frontmatter.state, t.frontmatter.title
        ),
    }
    Ok(())
}
