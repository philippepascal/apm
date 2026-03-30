use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WorkerPid {
    pub pid: u32,
    pub ticket_id: String,
    pub started_at: String,
}

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
    let worktrees = git::list_ticket_worktrees(root)?;
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

        let wpid = match read_pid_file(&pid_path) {
            Ok(w) => w,
            Err(_) => continue,
        };

        let alive = is_alive(wpid.pid);

        let t = tickets.iter().find(|t| {
            t.frontmatter.branch.as_deref() == Some(branch.as_str())
                || git::branch_name_from_path(&t.path).as_deref() == Some(branch.as_str())
        });

        let title = t.map(|t| t.frontmatter.title.as_str()).unwrap_or("—").to_string();
        let state = if alive {
            t.map(|t| t.frontmatter.state.as_str()).unwrap_or("—").to_string()
        } else {
            "crashed".to_string()
        };

        let pid_col = if alive {
            wpid.pid.to_string()
        } else {
            "—".to_string()
        };

        let elapsed = if alive {
            elapsed_since(&wpid.started_at)
        } else {
            "—".to_string()
        };

        rows.push(Row {
            id: wpid.ticket_id.clone(),
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
    let wpid = read_pid_file(&pid_path)?;
    if !is_alive(wpid.pid) {
        let _ = std::fs::remove_file(&pid_path);
        bail!("worker for ticket {id} is not running (stale PID {})", wpid.pid);
    }
    let status = std::process::Command::new("kill")
        .args(["-TERM", &wpid.pid.to_string()])
        .status()?;
    if !status.success() {
        bail!("failed to send SIGTERM to PID {}", wpid.pid);
    }
    let _ = std::fs::remove_file(&pid_path);
    println!("killed worker for ticket #{id} (PID {})", wpid.pid);
    Ok(())
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
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt = git::find_worktree_for_branch(root, &branch)
        .ok_or_else(|| anyhow::anyhow!("no worktree for ticket {id:?}"))?;
    Ok((wt, id))
}

pub fn read_pid_file(path: &Path) -> Result<WorkerPid> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn is_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn elapsed_since(started_at: &str) -> String {
    let Ok(started) = chrono::DateTime::parse_from_rfc3339(started_at)
        .or_else(|_| {
            // Try our truncated format: 2026-03-30T05:14Z
            chrono::DateTime::parse_from_rfc3339(&started_at.replace('Z', "+00:00"))
        })
    else {
        return "—".to_string();
    };
    let now = chrono::Utc::now();
    let secs = (now.timestamp() - started.timestamp()).max(0) as u64;
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        if m == 0 {
            format!("{h}h")
        } else {
            format!("{h}h {m}m")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_alive_returns_true_for_current_process() {
        assert!(is_alive(std::process::id()));
    }

    #[test]
    fn is_alive_returns_false_for_dead_pid() {
        // PID 0 is never a valid user process on Unix; kill -0 0 checks the
        // process group, but using a very large unlikely PID is safer.
        // PID 1 is init/launchd and kill -0 1 may fail due to permissions, but
        // a large PID like 99999999 is almost certainly not running.
        assert!(!is_alive(99999999));
    }

    #[test]
    fn read_pid_file_parses_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.pid");
        std::fs::write(&path, r#"{"pid":12345,"ticket_id":"0042","started_at":"2026-01-01T00:00Z"}"#).unwrap();
        let wp = read_pid_file(&path).unwrap();
        assert_eq!(wp.pid, 12345);
        assert_eq!(wp.ticket_id, "0042");
    }

    #[test]
    fn elapsed_since_seconds() {
        let now = chrono::Utc::now();
        let started = (now - chrono::Duration::seconds(30))
            .format("%Y-%m-%dT%H:%M:%S+00:00")
            .to_string();
        let s = elapsed_since(&started);
        assert!(s.ends_with('s'), "expected seconds, got: {s}");
    }

    #[test]
    fn elapsed_since_minutes() {
        let now = chrono::Utc::now();
        let started = (now - chrono::Duration::minutes(42))
            .format("%Y-%m-%dT%H:%M:%S+00:00")
            .to_string();
        let s = elapsed_since(&started);
        assert_eq!(s, "42m");
    }

    #[test]
    fn elapsed_since_hours() {
        let now = chrono::Utc::now();
        let started = (now - chrono::Duration::hours(2) - chrono::Duration::minutes(15))
            .format("%Y-%m-%dT%H:%M:%S+00:00")
            .to_string();
        let s = elapsed_since(&started);
        assert_eq!(s, "2h 15m");
    }

    #[test]
    fn elapsed_since_invalid_returns_dash() {
        assert_eq!(elapsed_since("not-a-date"), "—");
    }
}
