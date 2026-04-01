use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post, put},
    Json, Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

mod queue;
mod work;
mod workers;

enum TicketSource {
    Git(PathBuf, PathBuf),
    InMemory(Vec<apm_core::ticket::Ticket>),
}

struct AppState {
    source: TicketSource,
    work_engine: work::WorkEngineState,
}

impl AppState {
    fn git_root(&self) -> Option<&PathBuf> {
        match &self.source {
            TicketSource::Git(root, _) => Some(root),
            TicketSource::InMemory(_) => None,
        }
    }
}

#[derive(serde::Serialize)]
struct TransitionOption {
    to: String,
    label: String,
}

#[derive(serde::Serialize)]
struct TicketResponse {
    #[serde(flatten)]
    frontmatter: apm_core::ticket::Frontmatter,
    body: String,
}

#[derive(serde::Serialize)]
struct TicketDetailResponse {
    #[serde(flatten)]
    frontmatter: apm_core::ticket::Frontmatter,
    body: String,
    raw: String,
    valid_transitions: Vec<TransitionOption>,
}

#[derive(serde::Deserialize)]
struct TransitionRequest {
    to: String,
}

#[derive(serde::Deserialize)]
struct PutBodyRequest {
    content: String,
}

#[derive(serde::Deserialize)]
struct PatchTicketRequest {
    effort: Option<u8>,
    risk: Option<u8>,
    priority: Option<u8>,
}

#[derive(serde::Deserialize)]
struct CreateTicketRequest {
    title: Option<String>,
    problem: Option<String>,
    acceptance_criteria: Option<String>,
    out_of_scope: Option<String>,
    approach: Option<String>,
}

fn extract_frontmatter_raw(content: &str) -> Option<&str> {
    let rest = content.strip_prefix("+++\n")?;
    let end = rest.find("\n+++")?;
    Some(&rest[..end])
}

fn extract_history_raw(content: &str) -> &str {
    match content.find("\n## History") {
        Some(idx) => &content[idx..],
        None => "",
    }
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError(e)
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(e: tokio::task::JoinError) -> Self {
        AppError(anyhow::anyhow!(e))
    }
}

fn compute_valid_transitions(root: &PathBuf, state: &str) -> Vec<TransitionOption> {
    let Ok(config) = apm_core::config::Config::load(root) else {
        return vec![];
    };
    config
        .workflow
        .states
        .iter()
        .find(|s| s.id == state)
        .map(|s| {
            s.transitions
                .iter()
                .map(|tr| TransitionOption {
                    to: tr.to.clone(),
                    label: if tr.label.is_empty() {
                        format!("-> {}", tr.to)
                    } else {
                        tr.label.clone()
                    },
                })
                .collect()
        })
        .unwrap_or_default()
}

async fn load_tickets(state: &AppState) -> Result<Vec<apm_core::ticket::Ticket>, AppError> {
    match &state.source {
        TicketSource::Git(root, tickets_dir) => {
            let root = root.clone();
            let tickets_dir = tickets_dir.clone();
            Ok(tokio::task::spawn_blocking(move || {
                apm_core::ticket::load_all_from_git(&root, &tickets_dir)
            })
            .await??)
        }
        TicketSource::InMemory(tickets) => Ok(tickets.clone()),
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

async fn sync_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let (fetch_error, branches) = tokio::task::spawn_blocking(move || {
        let fetch_error = apm_core::git::fetch_all(&root).err().map(|e| e.to_string());
        apm_core::git::sync_local_ticket_refs(&root);
        let branches = apm_core::git::ticket_branches(&root)
            .map(|b| b.len())
            .unwrap_or(0);
        (fetch_error, branches)
    })
    .await?;
    let mut resp = serde_json::json!({ "branches": branches });
    if let Some(err) = fetch_error {
        resp["fetch_error"] = serde_json::Value::String(err);
    }
    Ok(Json(resp).into_response())
}

async fn list_tickets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TicketResponse>>, AppError> {
    let tickets = load_tickets(&state).await?;
    let response = tickets
        .into_iter()
        .map(|t| TicketResponse {
            frontmatter: t.frontmatter,
            body: t.body,
        })
        .collect();
    Ok(Json(response))
}

async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let tickets = load_tickets(&state).await?;
    match apm_core::ticket::resolve_id_in_slice(&tickets, &id) {
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("no ticket matches") {
                Ok((StatusCode::NOT_FOUND, msg).into_response())
            } else if msg.contains("invalid ticket ID") {
                Ok((StatusCode::BAD_REQUEST, msg).into_response())
            } else {
                Err(AppError(e))
            }
        }
        Ok(full_id) => {
            let ticket = tickets.into_iter().find(|t| t.frontmatter.id == full_id).unwrap();
            let valid_transitions = match state.git_root() {
                None => vec![],
                Some(root) => {
                    let root = root.clone();
                    let state_str = ticket.frontmatter.state.clone();
                    tokio::task::spawn_blocking(move || compute_valid_transitions(&root, &state_str)).await?
                }
            };
            let raw = ticket.serialize().unwrap_or_default();
            Ok(Json(TicketDetailResponse {
                frontmatter: ticket.frontmatter,
                body: ticket.body,
                raw,
                valid_transitions,
            })
            .into_response())
        }
    }
}

async fn transition_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<TransitionRequest>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => {
            return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response());
        }
    };
    let to_state = req.to.clone();
    let id_clone = id.clone();
    let root_clone = root.clone();
    let result = tokio::task::spawn_blocking(move || {
        apm_core::state::transition(&root_clone, &id_clone, to_state, false, false)
    })
    .await?;
    match result {
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("no ticket matches") {
                Ok((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": msg})),
                )
                    .into_response())
            } else {
                Ok((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({"error": msg})),
                )
                    .into_response())
            }
        }
        Ok(_output) => {
            let tickets = load_tickets(&state).await?;
            match apm_core::ticket::resolve_id_in_slice(&tickets, &id) {
                Err(e) => Err(AppError(e)),
                Ok(full_id) => {
                    let ticket =
                        tickets.into_iter().find(|t| t.frontmatter.id == full_id).unwrap();
                    let state_str = ticket.frontmatter.state.clone();
                    let root2 = root.clone();
                    let valid_transitions = tokio::task::spawn_blocking(move || {
                        compute_valid_transitions(&root2, &state_str)
                    })
                    .await?;
                    let raw = ticket.serialize().unwrap_or_default();
                    Ok(Json(TicketDetailResponse {
                        frontmatter: ticket.frontmatter,
                        body: ticket.body,
                        raw,
                        valid_transitions,
                    })
                    .into_response())
                }
            }
        }
    }
}

async fn put_body(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<PutBodyRequest>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let tickets = load_tickets(&state).await?;
    let full_id = match apm_core::ticket::resolve_id_in_slice(&tickets, &id) {
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("no ticket matches") {
                return Ok((StatusCode::NOT_FOUND, msg).into_response());
            } else if msg.contains("invalid ticket ID") {
                return Ok((StatusCode::BAD_REQUEST, msg).into_response());
            } else {
                return Err(AppError(e));
            }
        }
        Ok(id) => id,
    };
    let ticket = tickets.into_iter().find(|t| t.frontmatter.id == full_id).unwrap();
    let branch = match ticket.frontmatter.branch.clone() {
        Some(b) => b,
        None => {
            return Ok((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"error": "ticket has no branch"})),
            )
                .into_response())
        }
    };
    let rel_path = match ticket.path.strip_prefix(&root) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => {
            return Err(AppError(anyhow::anyhow!("cannot compute relative path for ticket")))
        }
    };

    let root_clone = root.clone();
    let branch_clone = branch.clone();
    let rel_path_clone = rel_path.clone();
    let current_content = tokio::task::spawn_blocking(move || {
        apm_core::git::read_from_branch(&root_clone, &branch_clone, &rel_path_clone)
    })
    .await??;

    let current_fm = match extract_frontmatter_raw(&current_content) {
        Some(fm) => fm.to_owned(),
        None => {
            return Err(AppError(anyhow::anyhow!("cannot parse frontmatter from current ticket")))
        }
    };
    let submitted_fm = match extract_frontmatter_raw(&req.content) {
        Some(fm) => fm.to_owned(),
        None => {
            return Ok((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"error": "cannot parse frontmatter from submitted content"})),
            )
                .into_response())
        }
    };

    let current_fm_val: toml::Value = toml::from_str(&current_fm)
        .map_err(|e| AppError(anyhow::anyhow!("invalid current frontmatter TOML: {e}")))?;
    let submitted_fm_val: toml::Value = match toml::from_str(&submitted_fm) {
        Ok(v) => v,
        Err(_) => {
            return Ok((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"error": "invalid frontmatter TOML in submitted content"})),
            )
                .into_response())
        }
    };
    if current_fm_val != submitted_fm_val {
        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": "frontmatter is read-only"})),
        )
            .into_response());
    }

    let current_history = extract_history_raw(&current_content).to_owned();
    let submitted_history = extract_history_raw(&req.content).to_owned();
    if current_history != submitted_history {
        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": "history section is read-only"})),
        )
            .into_response());
    }

    let content = req.content.clone();
    tokio::task::spawn_blocking(move || {
        apm_core::git::commit_to_branch(&root, &branch, &rel_path, &content, "ui: edit ticket body")
    })
    .await??;

    Ok(Json(serde_json::json!({"ok": true})).into_response())
}

async fn patch_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<PatchTicketRequest>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let tickets = load_tickets(&state).await?;
    let full_id = match apm_core::ticket::resolve_id_in_slice(&tickets, &id) {
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("no ticket matches") {
                return Ok((StatusCode::NOT_FOUND, msg).into_response());
            } else if msg.contains("invalid ticket ID") {
                return Ok((StatusCode::BAD_REQUEST, msg).into_response());
            } else {
                return Err(AppError(e));
            }
        }
        Ok(id) => id,
    };
    let ticket = tickets.into_iter().find(|t| t.frontmatter.id == full_id).unwrap();
    let branch = match ticket.frontmatter.branch.clone() {
        Some(b) => b,
        None => {
            return Ok((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"error": "ticket has no branch"})),
            )
                .into_response())
        }
    };
    let rel_path = match ticket.path.strip_prefix(&root) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => {
            return Err(AppError(anyhow::anyhow!("cannot compute relative path for ticket")))
        }
    };

    let mut fm = ticket.frontmatter;
    let body = ticket.body;

    if let Some(v) = req.effort {
        apm_core::ticket::set_field(&mut fm, "effort", &v.to_string())?;
    }
    if let Some(v) = req.risk {
        apm_core::ticket::set_field(&mut fm, "risk", &v.to_string())?;
    }
    if let Some(v) = req.priority {
        apm_core::ticket::set_field(&mut fm, "priority", &v.to_string())?;
    }

    let updated = apm_core::ticket::Ticket {
        frontmatter: fm,
        body,
        path: ticket.path,
    };
    let content = updated
        .serialize()
        .map_err(|e| AppError(anyhow::anyhow!("cannot serialize ticket: {e}")))?;

    let root_clone = root.clone();
    tokio::task::spawn_blocking(move || {
        apm_core::git::commit_to_branch(
            &root_clone,
            &branch,
            &rel_path,
            &content,
            "ui: update ticket fields",
        )
    })
    .await??;

    let state_str = updated.frontmatter.state.clone();
    let valid_transitions =
        tokio::task::spawn_blocking(move || compute_valid_transitions(&root, &state_str)).await?;
    let raw = updated.serialize().unwrap_or_default();
    Ok(Json(TicketDetailResponse {
        frontmatter: updated.frontmatter,
        body: updated.body,
        raw,
        valid_transitions,
    })
    .into_response())
}

async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTicketRequest>,
) -> Result<Response, AppError> {
    let title = match req.title {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "title is required"})),
            )
                .into_response());
        }
    };
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let section_sets: Vec<(String, String)> = [
        ("Problem", req.problem),
        ("Acceptance criteria", req.acceptance_criteria),
        ("Out of scope", req.out_of_scope),
        ("Approach", req.approach),
    ]
    .into_iter()
    .filter_map(|(name, val)| val.filter(|v| !v.trim().is_empty()).map(|v| (name.to_string(), v)))
    .collect();
    let result = tokio::task::spawn_blocking(move || {
        let config = apm_core::config::Config::load(&root)?;
        apm_core::ticket::create(
            &root,
            &config,
            title,
            "apm-ui".to_string(),
            None,
            None,
            false,
            section_sets,
        )
    })
    .await?;
    match result {
        Ok(ticket) => {
            let response = TicketResponse {
                frontmatter: ticket.frontmatter,
                body: ticket.body,
            };
            Ok((StatusCode::CREATED, Json(response)).into_response())
        }
        Err(e) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response()),
    }
}

fn build_app(root: PathBuf) -> Router {
    let config = apm_core::config::Config::load(&root).expect("cannot load apm config");
    let tickets_dir = config.tickets.dir;
    let state = Arc::new(AppState {
        source: TicketSource::Git(root, tickets_dir),
        work_engine: work::new_engine_state(),
    });
    let serve_dir = ServeDir::new("apm-ui/dist")
        .not_found_service(ServeFile::new("apm-ui/dist/index.html"));
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/sync", post(sync_handler))
        .route("/api/tickets", get(list_tickets).post(create_ticket))
        .route("/api/tickets/:id", get(get_ticket).patch(patch_ticket))
        .route("/api/tickets/:id/body", put(put_body))
        .route("/api/tickets/:id/transition", post(transition_ticket))
        .route("/api/queue", get(queue::queue_handler))
        .route("/api/workers", get(workers::workers_handler))
        .route("/api/work/status", get(work::get_work_status))
        .route("/api/work/start", post(work::post_work_start))
        .route("/api/work/stop", post(work::post_work_stop))
        .route("/api/work/dry-run", get(work::get_work_dry_run))
        .nest_service("/", serve_dir)
        .with_state(state)
}

#[cfg(test)]
fn build_app_with_tickets(tickets: Vec<apm_core::ticket::Ticket>) -> Router {
    let state = Arc::new(AppState {
        source: TicketSource::InMemory(tickets),
        work_engine: work::new_engine_state(),
    });
    Router::new()
        .route("/api/sync", post(sync_handler))
        .route("/api/tickets", get(list_tickets).post(create_ticket))
        .route("/api/tickets/:id", get(get_ticket).patch(patch_ticket))
        .route("/api/tickets/:id/body", put(put_body))
        .route("/api/tickets/:id/transition", post(transition_ticket))
        .with_state(state)
}

#[cfg(test)]
pub fn build_app_in_memory_with_workers(tickets: Vec<apm_core::ticket::Ticket>) -> Router {
    let state = Arc::new(AppState {
        source: TicketSource::InMemory(tickets),
        work_engine: work::new_engine_state(),
    });
    Router::new()
        .route("/api/workers", get(workers::workers_handler))
        .with_state(state)
}

#[cfg(test)]
pub fn build_app_in_memory_with_queue(tickets: Vec<apm_core::ticket::Ticket>) -> Router {
    let state = Arc::new(AppState {
        source: TicketSource::InMemory(tickets),
        work_engine: work::new_engine_state(),
    });
    Router::new()
        .route("/api/queue", get(queue::queue_handler))
        .with_state(state)
}

#[cfg(test)]
pub fn build_app_in_memory_for_work() -> Router {
    let state = Arc::new(AppState {
        source: TicketSource::InMemory(vec![]),
        work_engine: work::new_engine_state(),
    });
    Router::new()
        .route("/api/work/status", get(work::get_work_status))
        .route("/api/work/start", post(work::post_work_start))
        .route("/api/work/stop", post(work::post_work_stop))
        .route("/api/work/dry-run", get(work::get_work_dry_run))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let root = std::env::current_dir().unwrap();
    let app = build_app(root);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use apm_core::ticket::{Frontmatter, Ticket};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn fake_ticket(id: &str, title: &str) -> Ticket {
        Ticket {
            frontmatter: Frontmatter {
                id: id.to_string(),
                title: title.to_string(),
                state: "ready".to_string(),
                priority: 0,
                effort: 0,
                risk: 0,
                author: None,
                supervisor: None,
                agent: None,
                branch: None,
                created_at: None,
                updated_at: None,
                focus_section: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    fn test_tickets() -> Vec<Ticket> {
        vec![
            fake_ticket("aaaabbbb-fake-ticket-one", "Fake ticket one"),
            fake_ticket("ccccdddd-fake-ticket-two", "Fake ticket two"),
        ]
    }

    #[tokio::test]
    async fn list_tickets_returns_200_json_array() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets")
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
        assert_eq!(json.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn get_ticket_unknown_id_returns_404() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets/00000000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_ticket_valid_prefix_returns_200_object() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets/aaaabbbb")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json.is_object());
        assert_eq!(json["id"], "aaaabbbb-fake-ticket-one");
    }

    #[tokio::test]
    async fn get_ticket_invalid_id_format_returns_400() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets/ab")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_ticket_includes_valid_transitions_field() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets/aaaabbbb")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["valid_transitions"].is_array());
    }

    #[test]
    fn extract_frontmatter_raw_returns_toml_content() {
        let content = "+++\nid = \"abc\"\ntitle = \"t\"\n+++\n\n## Body\n";
        let fm = extract_frontmatter_raw(content).unwrap();
        // extract_frontmatter_raw returns the slice up to (but not including) the \n before +++
        assert_eq!(fm, "id = \"abc\"\ntitle = \"t\"");
    }

    #[test]
    fn extract_frontmatter_raw_returns_none_on_missing() {
        assert!(extract_frontmatter_raw("no frontmatter").is_none());
    }

    #[test]
    fn extract_history_raw_returns_from_heading() {
        let content = "## Spec\n\nBody text\n\n## History\n\n| row |";
        assert_eq!(extract_history_raw(content), "\n## History\n\n| row |");
    }

    #[test]
    fn extract_history_raw_returns_empty_when_absent() {
        assert_eq!(extract_history_raw("no history here"), "");
    }

    #[tokio::test]
    async fn put_body_in_memory_returns_501() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/tickets/aaaabbbb/body")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"content":"+++\n+++\n\nbody"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn get_ticket_includes_raw_field() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets/aaaabbbb")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["raw"].is_string());
        let raw = json["raw"].as_str().unwrap();
        assert!(raw.starts_with("+++\n"), "raw should start with +++");
    }

    #[tokio::test]
    async fn create_ticket_missing_title_returns_400() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tickets")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"problem":"something"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_ticket_empty_title_returns_400() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tickets")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"   "}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_ticket_in_memory_returns_501() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tickets")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"New ticket"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn patch_ticket_in_memory_returns_501() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/tickets/aaaabbbb")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"effort":5}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn patch_ticket_priority_out_of_range_returns_422() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/tickets/aaaabbbb")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"priority":256}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        // 256 exceeds u8::MAX → JSON deserialization fails → 422
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    fn git_setup(p: &std::path::Path) {
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@test.com"],
            vec!["config", "user.name", "test"],
        ] {
            std::process::Command::new("git")
                .args(&args)
                .current_dir(p)
                .status()
                .unwrap();
        }
        std::fs::write(
            p.join("apm.toml"),
            r#"[project]
name = "test"

[tickets]
dir = "tickets"

[agents]
max_concurrent = 3

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id         = "ready"
label      = "Ready"
actionable = ["agent"]

[[workflow.states]]
id    = "in_progress"
label = "In Progress"
"#,
        )
        .unwrap();
        for args in [
            vec!["add", "apm.toml"],
            vec!["-c", "commit.gpgsign=false", "commit", "-m", "init"],
        ] {
            std::process::Command::new("git")
                .args(&args)
                .current_dir(p)
                .env("GIT_AUTHOR_NAME", "test")
                .env("GIT_AUTHOR_EMAIL", "test@test.com")
                .env("GIT_COMMITTER_NAME", "test")
                .env("GIT_COMMITTER_EMAIL", "test@test.com")
                .status()
                .unwrap();
        }
        std::fs::create_dir_all(p.join("tickets")).unwrap();
    }

    #[tokio::test]
    async fn patch_ticket_priority_persists_to_git() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);

        let config = apm_core::config::Config::load(&p).unwrap();
        let ticket = apm_core::ticket::create(
            &p,
            &config,
            "test ticket".to_string(),
            "test".to_string(),
            None,
            None,
            false,
            vec![],
        )
        .unwrap();
        let ticket_id = ticket.frontmatter.id.clone();

        let app = build_app(p.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/tickets/{}", &ticket_id[..8]))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"priority":42}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["priority"], 42);

        // Verify the value was committed to the branch
        let branch = ticket.frontmatter.branch.unwrap();
        let rel_path = ticket.path.strip_prefix(&p).unwrap().to_string_lossy().to_string();
        let content = apm_core::git::read_from_branch(&p, &branch, &rel_path).unwrap();
        assert!(content.contains("priority = 42"), "expected priority = 42 in: {content}");
    }

    #[tokio::test]
    async fn patch_ticket_unknown_id_returns_not_implemented_for_in_memory() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/tickets/00000000")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"effort":5}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        // In-memory has no git root so returns 501 before ID resolution
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn sync_in_memory_returns_not_implemented() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/sync")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn transition_in_memory_returns_not_implemented() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tickets/aaaabbbb/transition")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"to":"in_progress"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn transition_unknown_id_returns_not_implemented_for_in_memory() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tickets/00000000/transition")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"to":"in_progress"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        // In-memory has no git root so returns 501 before ID resolution
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }
}
