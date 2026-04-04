use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;

use crate::{AppError, AppState, TicketSource};

#[derive(serde::Serialize)]
pub struct WorkerInfo {
    pid: u32,
    ticket_id: String,
    ticket_title: String,
    branch: String,
    state: String,
    elapsed: String,
    status: String,
}

pub async fn workers_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WorkerInfo>>, AppError> {
    let (root, tickets_dir) = match &state.source {
        TicketSource::Git(root, tickets_dir) => (root.clone(), tickets_dir.clone()),
        TicketSource::InMemory(_) => return Ok(Json(vec![])),
    };
    let results = tokio::task::spawn_blocking(move || collect_workers(&root, &tickets_dir)).await??;
    Ok(Json(results))
}

fn determine_status(alive: bool, state: &str, terminal_states: &std::collections::HashSet<&str>) -> &'static str {
    if alive {
        "running"
    } else if terminal_states.contains(state) {
        "ended"
    } else {
        "crashed"
    }
}

fn collect_workers(root: &FsPath, tickets_dir: &FsPath) -> anyhow::Result<Vec<WorkerInfo>> {
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(root)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let worktree_paths: Vec<PathBuf> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .map(PathBuf::from)
        .collect();

    let tickets = apm_core::ticket::load_all_from_git(root, tickets_dir).unwrap_or_default();
    let config = apm_core::config::Config::load(root)?;
    let terminal_states: std::collections::HashSet<&str> = config
        .workflow
        .states
        .iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();

    let mut results = Vec::new();
    for wt_path in worktree_paths {
        let pid_path = wt_path.join(".apm-worker.pid");
        if !pid_path.exists() {
            continue;
        }
        let (pid, pf) = match apm_core::worker::read_pid_file(&pid_path) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let alive = apm_core::worker::is_alive(pid);
        let elapsed = apm_core::worker::elapsed_since(&pf.started_at);
        let ticket = tickets.iter().find(|t| t.frontmatter.id == pf.ticket_id);
        let (ticket_title, branch, state) = match ticket {
            Some(t) => (
                t.frontmatter.title.clone(),
                t.frontmatter.branch.clone().unwrap_or_default(),
                t.frontmatter.state.clone(),
            ),
            None => (String::new(), String::new(), String::new()),
        };
        let status = determine_status(alive, &state, &terminal_states);
        results.push(WorkerInfo {
            pid,
            ticket_id: pf.ticket_id,
            ticket_title,
            branch,
            state,
            elapsed,
            status: status.to_string(),
        });
    }
    Ok(results)
}

enum StopError {
    NotFound,
    NotAlive,
    Other(String),
}

fn stop_worker_by_pid(root: &FsPath, target_pid: u32) -> Result<(), StopError> {
    let worktrees = apm_core::git::list_ticket_worktrees(root)
        .map_err(|e| StopError::Other(e.to_string()))?;
    for (wt_path, _branch) in &worktrees {
        let pid_path = wt_path.join(".apm-worker.pid");
        if !pid_path.exists() {
            continue;
        }
        let Ok((pid, _)) = apm_core::worker::read_pid_file(&pid_path) else {
            continue;
        };
        if pid != target_pid {
            continue;
        }
        if !apm_core::worker::is_alive(pid) {
            let _ = std::fs::remove_file(&pid_path);
            return Err(StopError::NotAlive);
        }
        std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|e| StopError::Other(e.to_string()))?;
        return Ok(());
    }
    Err(StopError::NotFound)
}

pub async fn delete_worker(
    State(state): State<Arc<AppState>>,
    Path(pid_str): Path<String>,
) -> impl IntoResponse {
    let pid: u32 = match pid_str.parse() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid pid"})),
            )
                .into_response()
        }
    };
    let root = match &state.source {
        TicketSource::Git(root, _) => root.clone(),
        TicketSource::InMemory(_) => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(serde_json::json!({"error": "no git root"})),
            )
                .into_response()
        }
    };
    let result = tokio::task::spawn_blocking(move || stop_worker_by_pid(&root, pid))
        .await
        .unwrap();
    match result {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(StopError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "pid not found"})),
        )
            .into_response(),
        Err(StopError::NotAlive) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "process not alive (stale pid file)"})),
        )
            .into_response(),
        Err(StopError::Other(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[test]
    fn determine_status_dead_terminal_shows_ended() {
        let mut terminal: std::collections::HashSet<&str> = std::collections::HashSet::new();
        terminal.insert("implemented");
        terminal.insert("closed");

        assert_eq!(determine_status(false, "implemented", &terminal), "ended");
        assert_eq!(determine_status(false, "closed", &terminal), "ended");
        assert_eq!(determine_status(false, "in_progress", &terminal), "crashed");
        assert_eq!(determine_status(true, "implemented", &terminal), "running");
        assert_eq!(determine_status(true, "in_progress", &terminal), "running");
    }

    #[tokio::test]
    async fn workers_empty_when_no_pid_files() {
        let app = crate::build_app_in_memory_with_workers(vec![]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/workers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("application/json"));
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 0);
    }
}
