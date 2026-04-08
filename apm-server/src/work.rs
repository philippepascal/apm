use axum::{extract::State, Json};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{AppError, AppState, TicketSource};

#[derive(serde::Serialize)]
pub struct DryRunCandidate {
    id: String,
    title: String,
    state: String,
    priority: u8,
    effort: u8,
    risk: u8,
    score: f64,
}

#[derive(serde::Serialize)]
pub struct DryRunResponse {
    candidates: Vec<DryRunCandidate>,
}

#[derive(serde::Deserialize, Default)]
pub struct StartWorkRequest {
    pub epic: Option<String>,
}

pub struct WorkEngine {
    pub cancel: Arc<AtomicBool>,
    pub handle: tokio::task::JoinHandle<()>,
    pub epic: Option<String>,
}

pub type WorkEngineState = Arc<Mutex<Option<WorkEngine>>>;

pub fn new_engine_state() -> WorkEngineState {
    Arc::new(Mutex::new(None))
}

fn engine_is_alive(engine: &WorkEngine) -> bool {
    !engine.handle.is_finished()
}

fn check_workers_alive(root: &Path) -> bool {
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(root)
        .output();
    let Ok(out) = output else { return false };
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .any(|wt| {
            let pid_path = PathBuf::from(wt).join(".apm-worker.pid");
            if !pid_path.exists() {
                return false;
            }
            match apm_core::worker::read_pid_file(&pid_path) {
                Ok((pid, _)) => apm_core::worker::is_alive(pid),
                Err(_) => false,
            }
        })
}

pub async fn get_work_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (alive, epic) = {
        let guard = state.work_engine.lock().await;
        match guard.as_ref() {
            Some(e) => (engine_is_alive(e), e.epic.clone()),
            None => (false, None),
        }
    };

    if !alive {
        return Ok(Json(serde_json::json!({"status": "stopped"})));
    }

    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok(Json(serde_json::json!({"status": "idle", "epic": epic}))),
    };

    let has_alive_worker =
        tokio::task::spawn_blocking(move || check_workers_alive(&root)).await?;

    let status = if has_alive_worker { "running" } else { "idle" };
    Ok(Json(serde_json::json!({"status": status, "epic": epic})))
}

pub async fn post_work_start(
    State(state): State<Arc<AppState>>,
    body: Option<Json<StartWorkRequest>>,
) -> Result<Json<serde_json::Value>, AppError> {
    {
        let guard = state.work_engine.lock().await;
        let already_running = guard.as_ref().map(engine_is_alive).unwrap_or(false);
        drop(guard);
        if already_running {
            return get_work_status(State(state)).await;
        }
    }

    let epic = body.and_then(|b| b.0.epic);

    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok(Json(serde_json::json!({"status": "stopped"}))),
    };

    let root_clone = root.clone();
    let config =
        tokio::task::spawn_blocking(move || apm_core::config::Config::load(&root_clone)).await??;

    let max_concurrent = {
        let ov = state.max_concurrent_override.lock().await;
        ov.unwrap_or_else(|| config.agents.max_concurrent.max(1))
    };
    let skip_permissions = config.agents.skip_permissions;

    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = cancel.clone();
    let epic_clone = epic.clone();
    let handle = tokio::task::spawn_blocking(move || {
        let _ = apm_core::work::run_engine_loop(
            &root,
            cancel_clone,
            30,
            max_concurrent,
            skip_permissions,
            epic_clone,
        );
    });

    {
        let mut guard = state.work_engine.lock().await;
        *guard = Some(WorkEngine { cancel, handle, epic });
    }

    Ok(Json(serde_json::json!({"status": "idle"})))
}

pub async fn post_work_stop(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let engine_opt = {
        let mut guard = state.work_engine.lock().await;
        guard.take()
    };

    let engine = match engine_opt {
        None => return Ok(Json(serde_json::json!({"status": "stopped"}))),
        Some(e) => e,
    };

    if !engine_is_alive(&engine) {
        return Ok(Json(serde_json::json!({"status": "stopped"})));
    }

    engine.cancel.store(true, Ordering::Relaxed);
    let _ =
        tokio::time::timeout(std::time::Duration::from_secs(10), engine.handle).await;

    Ok(Json(serde_json::json!({"status": "stopped"})))
}

pub async fn get_work_dry_run(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DryRunResponse>, AppError> {
    let (root, tickets_dir) = match &state.source {
        TicketSource::Git(root, tickets_dir) => (root.clone(), tickets_dir.clone()),
        TicketSource::InMemory(_) => {
            return Ok(Json(DryRunResponse { candidates: vec![] }))
        }
    };
    let override_val = *state.max_concurrent_override.lock().await;
    let candidates = tokio::task::spawn_blocking(move || {
        let config = apm_core::config::Config::load(&root)?;
        let pw = config.workflow.prioritization.priority_weight;
        let ew = config.workflow.prioritization.effort_weight;
        let rw = config.workflow.prioritization.risk_weight;
        let max_concurrent = override_val.unwrap_or_else(|| config.agents.max_concurrent.max(1));

        let startable: Vec<String> = config
            .workflow
            .states
            .iter()
            .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
            .map(|s| s.id.clone())
            .collect();
        let actionable_owned = config.actionable_states_for("agent");

        let current_user = apm_core::config::resolve_identity(&root);
        let tickets = apm_core::ticket::load_all_from_git(&root, &tickets_dir)?;
        let mut filtered: Vec<&apm_core::ticket::Ticket> = tickets
            .iter()
            .filter(|t| {
                let st = t.frontmatter.state.as_str();
                actionable_owned.iter().any(|a| a == st)
                    && (startable.is_empty() || startable.iter().any(|s| s == st))
            })
            .collect();
        filtered.retain(|t| t.frontmatter.owner.as_deref() == Some(current_user.as_str()));
        filtered.sort_by(|a, b| {
            b.score(pw, ew, rw)
                .partial_cmp(&a.score(pw, ew, rw))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let result: Vec<DryRunCandidate> = filtered
            .into_iter()
            .take(max_concurrent)
            .map(|t| {
                let fm = &t.frontmatter;
                let raw_score = t.score(pw, ew, rw);
                let score = (raw_score * 100.0).round() / 100.0;
                DryRunCandidate {
                    id: fm.id.clone(),
                    title: fm.title.clone(),
                    state: fm.state.clone(),
                    priority: fm.priority,
                    effort: fm.effort,
                    risk: fm.risk,
                    score,
                }
            })
            .collect();
        Ok::<_, anyhow::Error>(result)
    })
    .await??;
    Ok(Json(DryRunResponse { candidates }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn work_status_stopped_when_no_engine() {
        let app = crate::build_app_in_memory_for_work();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/work/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "stopped");
    }

    #[tokio::test]
    async fn work_stop_when_already_stopped_returns_stopped() {
        let app = crate::build_app_in_memory_for_work();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/work/stop")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "stopped");
    }

    #[tokio::test]
    async fn work_start_without_git_root_returns_stopped() {
        let app = crate::build_app_in_memory_for_work();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/work/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "stopped");
    }

    #[tokio::test]
    async fn work_start_with_epic_field_accepted() {
        let app = crate::build_app_in_memory_for_work();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/work/start")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"epic":"abc123"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn work_status_has_no_epic_key_when_stopped() {
        let app = crate::build_app_in_memory_for_work();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/work/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "stopped");
        assert!(json.get("epic").is_none());
    }

    #[tokio::test]
    async fn dry_run_returns_empty_candidates_for_in_memory() {
        let app = crate::build_app_in_memory_for_work();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/work/dry-run")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["candidates"].is_array());
        assert_eq!(json["candidates"].as_array().unwrap().len(), 0);
    }
}
