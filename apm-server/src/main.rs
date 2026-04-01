use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

mod workers;

enum TicketSource {
    Git(PathBuf, PathBuf),
    InMemory(Vec<apm_core::ticket::Ticket>),
}

struct AppState {
    source: TicketSource,
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
    valid_transitions: Vec<TransitionOption>,
}

#[derive(serde::Deserialize)]
struct TransitionRequest {
    to: String,
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
            let valid_transitions = state
                .git_root()
                .map(|root| compute_valid_transitions(root, &ticket.frontmatter.state))
                .unwrap_or_default();
            Ok(Json(TicketDetailResponse {
                frontmatter: ticket.frontmatter,
                body: ticket.body,
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
                    let valid_transitions =
                        compute_valid_transitions(&root, &ticket.frontmatter.state);
                    Ok(Json(TicketDetailResponse {
                        frontmatter: ticket.frontmatter,
                        body: ticket.body,
                        valid_transitions,
                    })
                    .into_response())
                }
            }
        }
    }
}

fn build_app(root: PathBuf) -> Router {
    let config = apm_core::config::Config::load(&root).expect("cannot load apm config");
    let tickets_dir = config.tickets.dir;
    let state = Arc::new(AppState {
        source: TicketSource::Git(root, tickets_dir),
    });
    let serve_dir = ServeDir::new("apm-ui/dist")
        .not_found_service(ServeFile::new("apm-ui/dist/index.html"));
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/tickets", get(list_tickets))
        .route("/api/tickets/:id", get(get_ticket))
        .route("/api/tickets/:id/transition", post(transition_ticket))
        .route("/api/workers", get(workers::workers_handler))
        .nest_service("/", serve_dir)
        .with_state(state)
}

#[cfg(test)]
fn build_app_with_tickets(tickets: Vec<apm_core::ticket::Ticket>) -> Router {
    let state = Arc::new(AppState {
        source: TicketSource::InMemory(tickets),
    });
    Router::new()
        .route("/api/tickets", get(list_tickets))
        .route("/api/tickets/:id", get(get_ticket))
        .route("/api/tickets/:id/transition", post(transition_ticket))
        .with_state(state)
}

#[cfg(test)]
pub fn build_app_in_memory_with_workers(tickets: Vec<apm_core::ticket::Ticket>) -> Router {
    let state = Arc::new(AppState {
        source: TicketSource::InMemory(tickets),
    });
    Router::new()
        .route("/api/workers", get(workers::workers_handler))
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
