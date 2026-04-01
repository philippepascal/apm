use axum::{extract::State, Json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{AppError, AppState, TicketSource};

#[derive(serde::Serialize)]
pub struct WorkerInfo {
    pid: u32,
    ticket_id: String,
    ticket_title: String,
    state: String,
    agent: String,
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

fn collect_workers(root: &Path, tickets_dir: &Path) -> anyhow::Result<Vec<WorkerInfo>> {
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
        let status = if apm_core::worker::is_alive(pid) { "running" } else { "crashed" };
        let elapsed = apm_core::worker::elapsed_since(&pf.started_at);
        let ticket = tickets.iter().find(|t| t.frontmatter.id == pf.ticket_id);
        let (ticket_title, state, agent) = match ticket {
            Some(t) => (
                t.frontmatter.title.clone(),
                t.frontmatter.state.clone(),
                t.frontmatter.agent.clone().unwrap_or_default(),
            ),
            None => (String::new(), String::new(), String::new()),
        };
        results.push(WorkerInfo {
            pid,
            ticket_id: pf.ticket_id,
            ticket_title,
            state,
            agent,
            elapsed,
            status: status.to_string(),
        });
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

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
