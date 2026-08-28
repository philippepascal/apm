use anyhow::{bail, Result};
use apm_core::{config::Config, denial, git, recovery, ticket, ticket_fmt, worker, worktree};
use std::path::Path;
use crate::util::worktree_for_ticket;

pub fn run(root: &Path, log_id: Option<&str>, kill_id: Option<&str>) -> Result<()> {
    if let Some(id_arg) = kill_id {
        return kill(root, id_arg);
    }
    if let Some(id_arg) = log_id {
        return tail_log(root, id_arg);
    }
    list(root)
}

pub fn run_diag(root: &Path, ticket_id: &str) -> Result<()> {
    let (wt, id) = worktree_for_ticket(root, ticket_id)?;
    let log_path = wt.join(".apm-worker.log");
    let summary_path = wt.join(".apm-worker.summary.json");

    let summary = if summary_path.exists() {
        denial::read_summary(&summary_path)
            .ok_or_else(|| anyhow::anyhow!("failed to parse {}", summary_path.display()))?
    } else if log_path.exists() {
        denial::scan_transcript(&log_path, &wt, &id)
    } else {
        bail!(
            "no worker log or summary found for ticket {id} (expected {} or {})",
            log_path.display(),
            summary_path.display()
        );
    };

    print_diag_report(&summary, &log_path);
    Ok(())
}

fn print_diag_report(summary: &denial::DenialSummary, log_path: &std::path::Path) {
    // Use the log_path recorded in the summary if it looks valid, otherwise
    // fall back to the path we derived from the worktree.
    let log_display = if !summary.log_path.is_empty() {
        summary.log_path.clone()
    } else {
        log_path.to_string_lossy().into_owned()
    };

    #[allow(clippy::print_stdout)]
    {
        println!("Worker denial report — {}", summary.ticket_id);
        println!("Log: {log_display}");
        println!();

        if summary.denial_count == 0 {
            println!("No denials detected.");
            return;
        }

        let apm_count = summary.denials.iter()
            .filter(|d| d.classification == denial::DenialClass::ApmCommandDenial)
            .count();
        let outside_count = summary.denials.iter()
            .filter(|d| d.classification == denial::DenialClass::OutsideWorktree)
            .count();
        let unknown_count = summary.denials.iter()
            .filter(|d| d.classification == denial::DenialClass::UnknownPattern)
            .count();

        println!("Total denials: {}", summary.denial_count);
        println!("  apm_command_denial : {apm_count}");
        println!("  outside_worktree   : {outside_count}");
        println!("  unknown_pattern    : {unknown_count}");

        if apm_count > 0 {
            println!();
            println!("APM command denials (allowlist gaps):");
            let unique_cmds = denial::collect_unique_apm_commands(summary);
            for cmd in &unique_cmds {
                // Find the first entry for this command to get its timestamp
                let ts = summary.denials.iter()
                    .find(|d| d.classification == denial::DenialClass::ApmCommandDenial && d.input == *cmd)
                    .map(|d| d.timestamp.as_str())
                    .unwrap_or("");
                if ts.is_empty() {
                    println!("  {cmd}");
                } else {
                    println!("  {cmd}  ({ts})");
                }
                println!("  \u{2192} Add \"Bash({cmd}*)\" to .claude/settings.json");
                println!("    and to APM_ALLOW_ENTRIES in apm-core/src/init.rs");
            }
        }
    }
}

/// One row of the crashed/running-worker scan shared by `list()` and `recover`.
struct WorkerRow {
    ticket_id: String,
    title: String,
    pid: Option<u32>,
    state: String,
    started_at: String,
}

/// Classify a dead worker's ticket state as either the ticket's real state
/// (if that state is terminal or worker_end) or "crashed" otherwise.
fn dead_worker_state(ticket_state: &str, ended_states: &std::collections::HashSet<&str>) -> String {
    if ended_states.contains(ticket_state) {
        ticket_state.to_string()
    } else {
        "crashed".to_string()
    }
}

/// Walk every permanent ticket worktree with a `.apm-worker.pid` file and
/// classify each as running or crashed. Shared by `list()` and `recover`.
fn scan_workers(root: &Path, config: &Config) -> Result<Vec<WorkerRow>> {
    let ended_states: std::collections::HashSet<&str> = config
        .workflow
        .states
        .iter()
        .filter(|s| s.terminal || s.worker_end)
        .map(|s| s.id.as_str())
        .collect();
    let worktrees = worktree::list_ticket_worktrees(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir).unwrap_or_default();

    let mut rows = Vec::new();

    for (wt_path, branch) in &worktrees {
        let pid_path = wt_path.join(".apm-worker.pid");
        if !pid_path.exists() {
            continue;
        }

        let (pid, pidfile) = match worker::read_pid_file(&pid_path) {
            Ok(w) => w,
            Err(_) => continue,
        };

        let alive = worker::is_alive(pid);

        let t = tickets.iter().find(|t| {
            t.frontmatter.branch.as_deref() == Some(branch.as_str())
                || ticket_fmt::branch_name_from_path(&t.path).as_deref() == Some(branch.as_str())
        });

        let title = t.map(|t| t.frontmatter.title.as_str()).unwrap_or("—").to_string();
        let ticket_state = t.map(|t| t.frontmatter.state.as_str()).unwrap_or("");
        let state = if alive {
            ticket_state.to_string()
        } else {
            dead_worker_state(ticket_state, &ended_states)
        };

        rows.push(WorkerRow {
            ticket_id: pidfile.ticket_id.clone(),
            title,
            pid: if alive { Some(pid) } else { None },
            state,
            started_at: pidfile.started_at.clone(),
        });
    }

    Ok(rows)
}

/// A ticket currently shown as "crashed" by `apm workers`.
struct CrashedWorker {
    ticket_id: String,
}

fn crashed_workers(root: &Path, config: &Config) -> Result<Vec<CrashedWorker>> {
    Ok(scan_workers(root, config)?
        .into_iter()
        .filter(|r| r.state == "crashed")
        .map(|r| CrashedWorker { ticket_id: r.ticket_id })
        .collect())
}

fn list(root: &Path) -> Result<()> {
    let config = Config::load(root)?;
    let scanned = scan_workers(root, &config)?;

    struct Row {
        id: String,
        title: String,
        pid: String,
        state: String,
        elapsed: String,
    }

    let rows: Vec<Row> = scanned
        .into_iter()
        .map(|r| Row {
            id: r.ticket_id,
            title: r.title,
            pid: r.pid.map(|p| p.to_string()).unwrap_or_else(|| "—".to_string()),
            state: r.state,
            elapsed: r.pid.map(|_| worker::elapsed_since(&r.started_at)).unwrap_or_else(|| "—".to_string()),
        })
        .collect();

    if rows.is_empty() {
        println!("No workers running.");
        return Ok(());
    }

    let id_w = rows.iter().map(|r| r.id.len()).max().unwrap_or(2).max(2);
    let title_w = rows.iter().map(|r| r.title.len()).max().unwrap_or(5).max(5);
    let pid_w = rows.iter().map(|r| r.pid.len()).max().unwrap_or(3).max(3);
    let state_w = rows.iter().map(|r| r.state.len()).max().unwrap_or(5).max(5);
    let elapsed_w = rows.iter().map(|r| r.elapsed.len()).max().unwrap_or(7).max(7);

    println!(
        "{:<id_w$}  {:<title_w$}  {:<pid_w$}  {:<state_w$}  {:<elapsed_w$}",
        "ID", "TITLE", "PID", "STATE", "ELAPSED",
        id_w = id_w,
        title_w = title_w,
        pid_w = pid_w,
        state_w = state_w,
        elapsed_w = elapsed_w,
    );

    for r in &rows {
        println!(
            "{:<id_w$}  {:<title_w$}  {:<pid_w$}  {:<state_w$}  {:<elapsed_w$}",
            r.id, r.title, r.pid, r.state, r.elapsed,
            id_w = id_w,
            title_w = title_w,
            pid_w = pid_w,
            state_w = state_w,
            elapsed_w = elapsed_w,
        );
    }

    Ok(())
}

fn tail_log(root: &Path, id_arg: &str) -> Result<()> {
    let (wt, id) = worktree_for_ticket(root, id_arg)?;
    let log_path = wt.join(".apm-worker.log");
    if !log_path.exists() {
        bail!("no log file for ticket {id}");
    }
    let status = std::process::Command::new("tail")
        .args(["-n", "50", "-f", &log_path.to_string_lossy()])
        .status()?;
    if !status.success() {
        bail!("tail exited with non-zero status");
    }
    Ok(())
}

fn kill(root: &Path, id_arg: &str) -> Result<()> {
    let (wt, id) = worktree_for_ticket(root, id_arg)?;
    let pid_path = wt.join(".apm-worker.pid");
    if !pid_path.exists() {
        bail!("worker for ticket {id} is not running (no .apm-worker.pid)");
    }
    let (pid, _) = worker::read_pid_file(&pid_path)?;
    if !worker::is_alive(pid) {
        let _ = std::fs::remove_file(&pid_path);
        bail!("worker for ticket {id} is not running (stale PID {})", pid);
    }
    let status = std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()?;
    if !status.success() {
        bail!("failed to send SIGTERM to PID {}", pid);
    }
    println!("killed worker for ticket #{id} (PID {})", pid);
    Ok(())
}

pub fn run_recover(root: &Path, id: Option<&str>, all: bool, dry_run: bool, to: Option<&str>) -> Result<()> {
    if all {
        if to.is_some() {
            bail!("--to cannot be combined with --all");
        }
        if id.is_some() {
            bail!("provide a ticket ID or --all, not both");
        }
        return recover_all(root, dry_run);
    }
    let Some(id) = id else {
        bail!("provide a ticket ID or use --all");
    };
    let msg = recover_one(root, id, dry_run, to)?;
    println!("{msg}");
    Ok(())
}

fn recover_all(root: &Path, dry_run: bool) -> Result<()> {
    let config = Config::load(root)?;
    let crashed = crashed_workers(root, &config)?;
    if crashed.is_empty() {
        println!("No crashed workers to recover.");
        return Ok(());
    }
    let total = crashed.len();
    let mut failures = 0usize;
    for c in &crashed {
        match recover_one(root, &c.ticket_id, dry_run, None) {
            Ok(msg) => println!("{msg}"),
            Err(e) => {
                #[allow(clippy::print_stderr)]
                {
                    eprintln!("{}: {e:#}", c.ticket_id);
                }
                failures += 1;
            }
        }
    }
    if failures > 0 {
        bail!("{failures} of {total} recoveries failed");
    }
    Ok(())
}

/// Recover a single crashed ticket: validate the worker is really dead,
/// resolve the pre-crash state, then roll the ticket back and drop the
/// stale pid file.
fn recover_one(root: &Path, id_arg: &str, dry_run: bool, to: Option<&str>) -> Result<String> {
    let config = Config::load(root)?;
    let (wt, id) = worktree_for_ticket(root, id_arg)?;
    let pid_path = wt.join(".apm-worker.pid");
    if !pid_path.exists() {
        bail!("nothing to recover for ticket {id}: no .apm-worker.pid file");
    }
    let (pid, _) = worker::read_pid_file(&pid_path)?;
    if worker::is_alive(pid) {
        bail!("worker for ticket {id} is still running (PID {pid}) — run `apm workers --kill {id}` first");
    }

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let t = tickets.iter().find(|t| t.frontmatter.id == id)
        .ok_or_else(|| anyhow::anyhow!("ticket {id:?} not found"))?;
    let current_state = t.frontmatter.state.clone();

    let target = recovery::resolve_recovery_target(&t.body, &current_state, &config.workflow, to)?;

    if git::is_worktree_dirty_for_sync(&wt) {
        #[allow(clippy::print_stderr)]
        {
            eprintln!("warning: worktree for ticket {id} has uncommitted changes — leaving them untouched");
        }
    }

    if dry_run {
        return Ok(format!(
            "{id}: would recover {current_state} → {target} (would remove {})",
            pid_path.display()
        ));
    }

    apm_core::state::transition(root, &id, target.clone(), true, true)?;
    let _ = std::fs::remove_file(&pid_path);
    Ok(format!("{id}: recovered {current_state} → {target}"))
}

#[cfg(test)]
mod tests {
    fn make_ended_states(ids: &[&'static str]) -> std::collections::HashSet<&'static str> {
        ids.iter().cloned().collect()
    }

    use super::dead_worker_state;

    #[test]
    fn dead_worker_end_state_shows_state() {
        let ended = make_ended_states(&["specd", "implemented"]);
        assert_eq!(dead_worker_state("specd", &ended), "specd");
        assert_eq!(dead_worker_state("implemented", &ended), "implemented");
    }

    #[test]
    fn dead_terminal_state_shows_state() {
        let ended = make_ended_states(&["closed", "specd", "implemented"]);
        assert_eq!(dead_worker_state("closed", &ended), "closed");
    }

    #[test]
    fn dead_non_ended_state_shows_crashed() {
        let ended = make_ended_states(&["specd", "implemented", "closed"]);
        assert_eq!(dead_worker_state("in_progress", &ended), "crashed");
        assert_eq!(dead_worker_state("ready", &ended), "crashed");
        assert_eq!(dead_worker_state("", &ended), "crashed");
    }
}
