use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn log(msg: &str) {
    let ts = chrono::Local::now().format("%H:%M:%S");
    println!("[{ts}] {msg}");
}

pub fn run(root: &Path, skip_permissions: bool, dry_run: bool, daemon: bool, interval_secs: u64, epic: Option<String>) -> Result<()> {
    if daemon && dry_run {
        anyhow::bail!("--daemon and --dry-run cannot be used together");
    }

    let config = Config::load(root)?;
    let max_concurrent = config.agents.max_concurrent.max(1);
    let epic_filter: Option<String> = epic.or_else(|| config.work.epic.clone());

    if dry_run {
        return run_dry(root, &config, epic_filter.as_deref());
    }

    let sig_count = Arc::new(AtomicUsize::new(0));
    let sig_count_clone = Arc::clone(&sig_count);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime for signal handling");
        rt.block_on(async move {
            loop {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to listen for ctrl-c");
                sig_count_clone.fetch_add(1, Ordering::Relaxed);
            }
        });
    });

    let mut workers: Vec<(String, Option<String>, std::process::Child, std::path::PathBuf)> = Vec::new();
    let mut started_ids: Vec<String> = Vec::new();
    let mut no_more = false;
    // next_poll only used in daemon mode
    let mut next_poll = Instant::now();
    let mut drain_announced = false;

    loop {
        let sigs = sig_count.load(Ordering::Relaxed);

        if daemon {
            if sigs >= 2 {
                log(&format!("Forced exit; {} worker(s) may still be running", workers.len()));
                break;
            }
            if sigs == 1 {
                if workers.is_empty() {
                    log("Daemon stopped.");
                    break;
                }
                if !drain_announced {
                    log(&format!(
                        "Graceful shutdown: waiting for {} worker(s) to finish (Ctrl+C again to exit immediately)",
                        workers.len()
                    ));
                    drain_announced = true;
                }
                // Reap finished workers during drain.
                workers.retain_mut(|(id, _epic_id, child, _pid_path)| {
                    let done = matches!(child.try_wait(), Ok(Some(_)));
                    if done {
                        log(&format!("Worker for ticket #{id} finished"));
                    }
                    !done
                });
                if workers.is_empty() {
                    log("All workers finished; exiting.");
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
        } else if sigs >= 1 {
            break;
        }

        // Reap finished workers.
        let mut reaped = false;
        workers.retain_mut(|(id, _epic_id, child, _pid_path)| {
            let done = matches!(child.try_wait(), Ok(Some(_)));
            if done {
                log(&format!("Worker for ticket #{id} finished"));
                reaped = true;
            }
            !done
        });

        // In daemon mode: a reaped worker opens a slot — check immediately.
        if daemon && reaped {
            next_poll = Instant::now();
            no_more = false;
        }

        if !daemon && no_more && workers.is_empty() {
            break;
        }

        // In daemon mode: if no_more and not yet time to poll again, sleep and continue.
        if daemon && no_more {
            let now = Instant::now();
            if now < next_poll {
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            // Poll interval elapsed — try again.
            no_more = false;
        }

        if !no_more && workers.len() < max_concurrent {
            let blocked_epics: Vec<String> = {
                let epic_ids: Vec<Option<String>> = workers.iter()
                    .map(|(_, eid, _, _)| eid.clone())
                    .collect();
                config.blocked_epics(&epic_ids)
            };
            match super::start::spawn_next_worker(root, true, skip_permissions, epic_filter.as_deref(), &blocked_epics) {
                Ok(None) => {
                    if daemon {
                        let secs = interval_secs;
                        log(&format!("No actionable tickets; next check in {secs}s"));
                        next_poll = Instant::now() + Duration::from_secs(interval_secs);
                    }
                    no_more = true;
                }
                Ok(Some((id, epic_id, child, pid_path))) => {
                    log(&format!(
                        "Dispatched worker for ticket #{id}"
                    ));
                    started_ids.push(id.clone());
                    workers.push((id, epic_id, child, pid_path));
                    no_more = false;
                }
                Err(e) => {
                    eprintln!("warning: dispatch failed: {e:#}");
                    no_more = true;
                    std::thread::sleep(Duration::from_secs(30));
                }
            }
        } else {
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    // Wait for all remaining workers in non-daemon mode (they were already
    // reaped in the loop above for daemon mode; non-daemon exits when empty).
    // In daemon mode workers run independently — we just stop dispatching.

    if started_ids.is_empty() {
        println!("No tickets to work.");
        return Ok(());
    }

    if daemon {
        // Don't print summary — workers are still running independently.
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

fn run_dry(root: &Path, config: &Config, epic_filter: Option<&str>) -> Result<()> {
    let pw = config.workflow.prioritization.priority_weight;
    let ew = config.workflow.prioritization.effort_weight;
    let rw = config.workflow.prioritization.risk_weight;
    let max_concurrent = config.agents.max_concurrent.max(1);

    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();
    let actionable_owned = config.actionable_states_for("agent");
    let actionable: Vec<&str> = actionable_owned.iter().map(|s| s.as_str()).collect();

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let mut candidates: Vec<&ticket::Ticket> = tickets
        .iter()
        .filter(|t| {
            let state = t.frontmatter.state.as_str();
            actionable.contains(&state)
                && (startable.is_empty() || startable.contains(&state))
                && epic_filter
                    .map_or(true, |id| t.frontmatter.epic.as_deref() == Some(id))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_dry_run_is_error() {
        // We can't call run() against a real git repo here, but we can verify
        // the guard fires before any I/O by passing a non-existent path and
        // ensuring the error message mentions the flag combination.
        let result = run(
            std::path::Path::new("/nonexistent"),
            false,
            true,  // dry_run
            true,  // daemon
            30,
            None,
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("--daemon") && msg.contains("--dry-run"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn sig_count_increments_correctly() {
        let sig_count = Arc::new(AtomicUsize::new(0));
        assert_eq!(sig_count.load(Ordering::Relaxed), 0);

        sig_count.fetch_add(1, Ordering::Relaxed);
        assert_eq!(sig_count.load(Ordering::Relaxed), 1);

        sig_count.fetch_add(1, Ordering::Relaxed);
        let sigs = sig_count.load(Ordering::Relaxed);
        assert!(sigs >= 2);
    }
}
