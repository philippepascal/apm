use anyhow::{bail, Result};
use apm_core::{config::Config, ticket, ticket_fmt, worker, worktree};
use std::path::{Path, PathBuf};

pub fn run(root: &Path, log_id: Option<&str>, kill_id: Option<&str>) -> Result<()> {
    if let Some(id_arg) = kill_id {
        return kill(root, id_arg);
    }
    if let Some(id_arg) = log_id {
        return tail_log(root, id_arg);
    }
    list(root)
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

fn worktree_for_ticket(root: &Path, id_arg: &str) -> Result<(PathBuf, String)> {
    let config = Config::load(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let id = ticket::resolve_id_in_slice(&tickets, id_arg)?;
    let t = tickets
        .iter()
        .find(|t| t.frontmatter.id == id)
        .ok_or_else(|| anyhow::anyhow!("ticket {id:?} not found"))?;
    let branch = t
        .frontmatter
        .branch
        .clone()
        .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt = worktree::find_worktree_for_branch(root, &branch)
        .ok_or_else(|| anyhow::anyhow!("no worktree for ticket {id:?}"))?;
    Ok((wt, id))
}
