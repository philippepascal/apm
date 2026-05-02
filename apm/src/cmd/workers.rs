use anyhow::{bail, Result};
use apm_core::{config::Config, denial, ticket, ticket_fmt, worker, worktree};
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

fn list(root: &Path) -> Result<()> {
    let config = Config::load(root)?;
    let ended_states: std::collections::HashSet<&str> = config
        .workflow
        .states
        .iter()
        .filter(|s| s.terminal || s.worker_end)
        .map(|s| s.id.as_str())
        .collect();
    let worktrees = worktree::list_ticket_worktrees(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir).unwrap_or_default();

    struct Row {
        id: String,
        title: String,
        pid: String,
        state: String,
        elapsed: String,
    }

    let mut rows: Vec<Row> = Vec::new();

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
        let state = if alive {
            t.map(|t| t.frontmatter.state.as_str()).unwrap_or("—").to_string()
        } else {
            let ticket_state = t.map(|t| t.frontmatter.state.as_str()).unwrap_or("");
            if ended_states.contains(ticket_state) {
                ticket_state.to_string()
            } else {
                "crashed".to_string()
            }
        };

        let pid_col = if alive {
            pid.to_string()
        } else {
            "—".to_string()
        };

        let elapsed = if alive {
            worker::elapsed_since(&pidfile.started_at)
        } else {
            "—".to_string()
        };

        rows.push(Row {
            id: pidfile.ticket_id.clone(),
            title,
            pid: pid_col,
            state,
            elapsed,
        });
    }

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

#[cfg(test)]
mod tests {
    fn make_ended_states(ids: &[&'static str]) -> std::collections::HashSet<&'static str> {
        ids.iter().cloned().collect()
    }

    fn dead_worker_state(ticket_state: &str, ended_states: &std::collections::HashSet<&str>) -> String {
        if ended_states.contains(ticket_state) {
            ticket_state.to_string()
        } else {
            "crashed".to_string()
        }
    }

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
