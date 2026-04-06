use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch, post, put},
    Json, Router,
};
use include_dir::{include_dir, Dir};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

static UI_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../apm-ui/dist");

mod agents;
mod tls;
mod auth;
mod credential_store;
mod log;
mod queue;
mod webauthn_state;
mod work;
mod workers;

enum TicketSource {
    Git(PathBuf, PathBuf),
    InMemory(Vec<apm_core::ticket::Ticket>),
}

struct AppState {
    source: TicketSource,
    work_engine: work::WorkEngineState,
    log_file: Option<std::path::PathBuf>,
    max_concurrent_override: Arc<tokio::sync::Mutex<Option<usize>>>,
    otp_store: auth::OtpStore,
    session_store: auth::SessionStore,
    webauthn_state: Arc<webauthn_state::WebauthnState>,
    credential_store: credential_store::CredentialStore,
}

fn is_localhost(connect_info: Option<ConnectInfo<SocketAddr>>) -> bool {
    connect_info
        .map(|ConnectInfo(addr)| addr.ip().is_loopback())
        .unwrap_or(false)
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
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
}

#[derive(serde::Serialize)]
struct TicketResponse {
    #[serde(flatten)]
    frontmatter: apm_core::ticket::Frontmatter,
    body: String,
    has_open_questions: bool,
    has_pending_amendments: bool,
    blocking_deps: Vec<BlockingDep>,
}

fn extract_section<'a>(body: &'a str, heading: &str) -> &'a str {
    let marker = format!("### {heading}");
    let Some(start) = body.find(&marker) else {
        return "";
    };
    let after = &body[start + marker.len()..];
    match after.find("\n###") {
        Some(end) => &after[..end],
        None => after,
    }
}

#[derive(serde::Serialize)]
struct BlockingDep {
    id: String,
    state: String,
}

#[derive(serde::Serialize)]
struct TicketDetailResponse {
    #[serde(flatten)]
    frontmatter: apm_core::ticket::Frontmatter,
    body: String,
    raw: String,
    valid_transitions: Vec<TransitionOption>,
    blocking_deps: Vec<BlockingDep>,
}

#[derive(serde::Deserialize)]
struct TransitionRequest {
    to: String,
}

#[derive(serde::Deserialize)]
struct BatchTransitionRequest {
    ids: Vec<String>,
    to: String,
}

#[derive(serde::Deserialize)]
struct BatchPriorityRequest {
    ids: Vec<String>,
    priority: u8,
}

#[derive(serde::Serialize)]
struct BatchFailure {
    id: String,
    error: String,
}

#[derive(serde::Serialize)]
struct BatchResult {
    succeeded: Vec<String>,
    failed: Vec<BatchFailure>,
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
    sections: Option<std::collections::HashMap<String, String>>,
    epic: Option<String>,
    depends_on: Option<Vec<String>>,
}

fn find_epic_branch(root: &std::path::Path, short_id: &str) -> Option<String> {
    apm_core::git::find_epic_branch(root, short_id)
}

#[derive(serde::Serialize)]
struct EpicSummary {
    id: String,
    title: String,
    branch: String,
    state: String,
    ticket_counts: std::collections::HashMap<String, usize>,
}

#[derive(serde::Serialize)]
struct EpicDetailResponse {
    #[serde(flatten)]
    summary: EpicSummary,
    tickets: Vec<TicketResponse>,
}

#[derive(serde::Deserialize)]
struct CreateEpicRequest {
    title: Option<String>,
}

fn parse_epic_branch(branch: &str) -> Option<(String, String)> {
    let rest = branch.strip_prefix("epic/")?;
    let dash = rest.find('-')?;
    let id = rest[..dash].to_string();
    let slug = &rest[dash + 1..];
    let title = slug
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Some((id, title))
}

fn derive_epic_state(
    tickets: &[&apm_core::ticket::Ticket],
    states: &[apm_core::config::StateConfig],
) -> String {
    if tickets.is_empty() {
        return "empty".to_string();
    }
    let state_map: std::collections::HashMap<&str, &apm_core::config::StateConfig> =
        states.iter().map(|s| (s.id.as_str(), s)).collect();
    if tickets.iter().any(|t| {
        state_map
            .get(t.frontmatter.state.as_str())
            .map(|s| s.actionable.iter().any(|a| a == "agent"))
            .unwrap_or(false)
    }) {
        return "active".to_string();
    }
    let all_satisfies_or_terminal = tickets.iter().all(|t| {
        state_map
            .get(t.frontmatter.state.as_str())
            .map(|s| matches!(s.satisfies_deps, apm_core::config::SatisfiesDeps::Bool(true)) || s.terminal)
            .unwrap_or(false)
    });
    if all_satisfies_or_terminal {
        let any_satisfies = tickets.iter().any(|t| {
            state_map
                .get(t.frontmatter.state.as_str())
                .map(|s| matches!(s.satisfies_deps, apm_core::config::SatisfiesDeps::Bool(true)))
                .unwrap_or(false)
        });
        let all_terminal = tickets.iter().all(|t| {
            state_map
                .get(t.frontmatter.state.as_str())
                .map(|s| s.terminal)
                .unwrap_or(false)
        });
        if all_terminal {
            return "done".to_string();
        }
        if any_satisfies {
            return "complete".to_string();
        }
    }
    "active".to_string()
}

fn build_epic_summary(
    branch: &str,
    all_tickets: &[apm_core::ticket::Ticket],
    states: &[apm_core::config::StateConfig],
) -> Option<EpicSummary> {
    let (id, title) = parse_epic_branch(branch)?;
    let epic_tickets: Vec<&apm_core::ticket::Ticket> = all_tickets
        .iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(id.as_str()))
        .collect();
    let mut ticket_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for t in &epic_tickets {
        *ticket_counts.entry(t.frontmatter.state.clone()).or_insert(0) += 1;
    }
    let state = derive_epic_state(&epic_tickets, states);
    Some(EpicSummary {
        id,
        title,
        branch: branch.to_string(),
        state,
        ticket_counts,
    })
}

async fn list_epics(
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let tickets = load_tickets(&state).await?;
    let config = tokio::task::spawn_blocking({
        let root = root.clone();
        move || apm_core::config::Config::load(&root)
    })
    .await??;
    let branches = tokio::task::spawn_blocking(move || apm_core::git::epic_branches(&root))
        .await??;
    let summaries: Vec<EpicSummary> = branches
        .iter()
        .filter_map(|b| build_epic_summary(b, &tickets, &config.workflow.states))
        .collect();
    Ok(Json(summaries).into_response())
}

async fn create_epic(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateEpicRequest>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let title = match req.title {
        Some(t) if !t.trim().is_empty() => t,
        _ => return Ok((StatusCode::BAD_REQUEST, "title is required").into_response()),
    };
    let title_clone = title.clone();
    let (id, branch) = tokio::task::spawn_blocking(move || {
        apm_core::git::create_epic_branch(&root, &title_clone)
    })
    .await??;
    Ok((
        StatusCode::CREATED,
        Json(EpicSummary {
            id,
            title,
            branch,
            state: "empty".to_string(),
            ticket_counts: std::collections::HashMap::new(),
        }),
    )
        .into_response())
}

async fn get_epic(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let tickets = load_tickets(&state).await?;
    let config = tokio::task::spawn_blocking({
        let root = root.clone();
        move || apm_core::config::Config::load(&root)
    })
    .await??;
    let branches = tokio::task::spawn_blocking(move || apm_core::git::epic_branches(&root))
        .await??;
    let branch = match branches.iter().find(|b| {
        b.strip_prefix("epic/")
            .and_then(|s| s.split('-').next())
            .map(|seg| seg == id)
            .unwrap_or(false)
    }) {
        Some(b) => b.clone(),
        None => return Ok((StatusCode::NOT_FOUND, "epic not found").into_response()),
    };
    let summary = match build_epic_summary(&branch, &tickets, &config.workflow.states) {
        Some(s) => s,
        None => return Ok((StatusCode::NOT_FOUND, "epic not found").into_response()),
    };
    let epic_id = summary.id.clone();
    let epic_tickets: Vec<TicketResponse> = tickets
        .into_iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id.as_str()))
        .map(|t| {
            let has_open_questions = !extract_section(&t.body, "Open questions").trim().is_empty();
            let has_pending_amendments =
                extract_section(&t.body, "Amendment requests").contains("- [ ]");
            TicketResponse {
                frontmatter: t.frontmatter,
                body: t.body,
                has_open_questions,
                has_pending_amendments,
                blocking_deps: vec![],
            }
        })
        .collect();
    Ok(Json(EpicDetailResponse {
        summary,
        tickets: epic_tickets,
    })
    .into_response())
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

fn compute_blocking_deps(
    ticket: &apm_core::ticket::Ticket,
    all_tickets: &[apm_core::ticket::Ticket],
    root: &PathBuf,
) -> Vec<BlockingDep> {
    let deps = match &ticket.frontmatter.depends_on {
        Some(d) if !d.is_empty() => d,
        _ => return vec![],
    };
    let Ok(config) = apm_core::config::Config::load(root) else {
        return vec![];
    };
    let state_map: std::collections::HashMap<&str, &str> = all_tickets
        .iter()
        .map(|t| (t.frontmatter.id.as_str(), t.frontmatter.state.as_str()))
        .collect();
    deps.iter()
        .filter_map(|dep_id| {
            state_map.get(dep_id.as_str()).and_then(|&s| {
                if apm_core::ticket::dep_satisfied(s, None, &config) {
                    None
                } else {
                    Some(BlockingDep { id: dep_id.clone(), state: s.to_string() })
                }
            })
        })
        .collect()
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
                    warning: tr.warning.clone(),
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
    let (fetch_error, branches, closed) = tokio::task::spawn_blocking(move || {
        let fetch_error = apm_core::git::fetch_all(&root).err().map(|e| e.to_string());
        apm_core::git::sync_local_ticket_refs(&root);
        let branches = apm_core::git::ticket_branches(&root)
            .map(|b| b.len())
            .unwrap_or(0);
        let closed = match apm_core::config::Config::load(&root) {
            Ok(config) => {
                match apm_core::sync::detect(&root, &config) {
                    Ok(candidates) => {
                        let n = candidates.close.len();
                        if n > 0 {
                            let aggressive = config.sync.aggressive;
                            let author = apm_core::config::resolve_identity(&root);
                            let _ = apm_core::sync::apply(&root, &config, &candidates, &author, aggressive);
                        }
                        n
                    }
                    Err(_) => 0,
                }
            }
            Err(_) => 0,
        };
        (fetch_error, branches, closed)
    })
    .await?;
    let mut resp = serde_json::json!({ "branches": branches, "closed": closed });
    if let Some(err) = fetch_error {
        resp["fetch_error"] = serde_json::Value::String(err);
    }
    Ok(Json(resp).into_response())
}

async fn clean_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let removed = tokio::task::spawn_blocking(move || -> anyhow::Result<usize> {
        let config = apm_core::config::Config::load(&root)?;
        let (candidates, _dirty) = apm_core::clean::candidates(&root, &config, false, false, false)?;
        let mut count = 0;
        for candidate in &candidates {
            if candidate.worktree.is_some() {
                apm_core::clean::remove(&root, candidate, false, false)?;
                count += 1;
            }
        }
        Ok(count)
    })
    .await??;
    Ok(Json(serde_json::json!({ "removed": removed })).into_response())
}

#[derive(serde::Deserialize, Default)]
struct ListTicketsQuery {
    include_closed: Option<bool>,
    author: Option<String>,
    owner: Option<String>,
}

async fn list_tickets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListTicketsQuery>,
) -> Result<Json<Vec<TicketResponse>>, AppError> {
    let mut tickets = load_tickets(&state).await?;
    let (resolved_ids, terminal_ids): (Vec<String>, Vec<String>) = match state.git_root() {
        Some(root) => match apm_core::config::Config::load(root) {
            Ok(cfg) => {
                let resolved = cfg.workflow.states.iter()
                    .filter(|s| matches!(s.satisfies_deps, apm_core::config::SatisfiesDeps::Bool(true)) || s.terminal)
                    .map(|s| s.id.clone())
                    .collect();
                let terminal = cfg.workflow.states.into_iter()
                    .filter(|s| s.terminal)
                    .map(|s| s.id)
                    .collect();
                (resolved, terminal)
            }
            Err(_) => (vec![], vec!["closed".to_string()]),
        },
        None => (vec![], vec!["closed".to_string()]),
    };
    if !params.include_closed.unwrap_or(false) {
        let terminal_set: std::collections::HashSet<&str> =
            terminal_ids.iter().map(|s| s.as_str()).collect();
        tickets.retain(|t| !terminal_set.contains(t.frontmatter.state.as_str()));
    }
    if let Some(ref author) = params.author {
        tickets.retain(|t| {
            let a = t.frontmatter.author.as_deref().unwrap_or("unassigned");
            a == author.as_str()
        });
    }
    if let Some(ref owner) = params.owner {
        if owner == "unassigned" {
            tickets.retain(|t| t.frontmatter.owner.is_none());
        } else {
            tickets.retain(|t| t.frontmatter.owner.as_deref() == Some(owner.as_str()));
        }
    }
    let resolved: std::collections::HashSet<&str> =
        resolved_ids.iter().map(|s| s.as_str()).collect();
    let state_map: std::collections::HashMap<String, String> = tickets
        .iter()
        .map(|t| (t.frontmatter.id.clone(), t.frontmatter.state.clone()))
        .collect();
    let response = tickets
        .into_iter()
        .map(|t| {
            let has_open_questions = !extract_section(&t.body, "Open questions").trim().is_empty();
            let has_pending_amendments = extract_section(&t.body, "Amendment requests").contains("- [ ]");
            let blocking_deps = t.frontmatter.depends_on
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter_map(|dep_id| {
                    state_map.get(dep_id.as_str()).and_then(|s| {
                        if resolved.contains(s.as_str()) { None }
                        else { Some(BlockingDep { id: dep_id.clone(), state: s.clone() }) }
                    })
                })
                .collect();
            let mut fm = t.frontmatter;
            if fm.author.is_none() {
                fm.author = Some("unassigned".to_string());
            }
            TicketResponse {
                frontmatter: fm,
                body: t.body,
                has_open_questions,
                has_pending_amendments,
                blocking_deps,
            }
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
            let (blocking_deps, valid_transitions) = match state.git_root() {
                None => (vec![], vec![]),
                Some(root) => {
                    let root = root.clone();
                    let ticket_ref = tickets.iter().find(|t| t.frontmatter.id == full_id).unwrap();
                    let deps = compute_blocking_deps(ticket_ref, &tickets, &root);
                    let state_str = ticket_ref.frontmatter.state.clone();
                    let transitions = tokio::task::spawn_blocking(move || compute_valid_transitions(&root, &state_str)).await?;
                    (deps, transitions)
                }
            };
            let mut ticket = tickets.into_iter().find(|t| t.frontmatter.id == full_id).unwrap();
            let raw = ticket.serialize().unwrap_or_default();
            if ticket.frontmatter.author.is_none() {
                ticket.frontmatter.author = Some("unassigned".to_string());
            }
            Ok(Json(TicketDetailResponse {
                frontmatter: ticket.frontmatter,
                body: ticket.body,
                raw,
                valid_transitions,
                blocking_deps,
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
                    let blocking_deps = compute_blocking_deps(
                        tickets.iter().find(|t| t.frontmatter.id == full_id).unwrap(),
                        &tickets,
                        &root,
                    );
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
                        blocking_deps,
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
    let blocking_deps = compute_blocking_deps(
        tickets.iter().find(|t| t.frontmatter.id == full_id).unwrap(),
        &tickets,
        &root,
    );
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
        blocking_deps,
    })
    .into_response())
}

async fn batch_transition(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchTransitionRequest>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();
    for id in req.ids {
        let root_clone = root.clone();
        let id_clone = id.clone();
        let to_clone = req.to.clone();
        let result = tokio::task::spawn_blocking(move || {
            apm_core::state::transition(&root_clone, &id_clone, to_clone, false, false)
        })
        .await?;
        match result {
            Ok(_) => succeeded.push(id),
            Err(e) => failed.push(BatchFailure { id, error: e.to_string() }),
        }
    }
    Ok(Json(BatchResult { succeeded, failed }).into_response())
}

async fn batch_priority(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchPriorityRequest>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let tickets = load_tickets(&state).await?;
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();
    for id in req.ids {
        let full_id = match apm_core::ticket::resolve_id_in_slice(&tickets, &id) {
            Ok(fid) => fid,
            Err(e) => {
                failed.push(BatchFailure { id, error: e.to_string() });
                continue;
            }
        };
        let ticket = match tickets.iter().find(|t| t.frontmatter.id == full_id) {
            Some(t) => t.clone(),
            None => {
                failed.push(BatchFailure { id, error: "not found".to_string() });
                continue;
            }
        };
        let branch = match ticket.frontmatter.branch.clone() {
            Some(b) => b,
            None => {
                failed.push(BatchFailure { id, error: "ticket has no branch".to_string() });
                continue;
            }
        };
        let rel_path = match ticket.path.strip_prefix(&root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => {
                failed.push(BatchFailure { id, error: "cannot compute relative path".to_string() });
                continue;
            }
        };
        let mut fm = ticket.frontmatter.clone();
        let body = ticket.body.clone();
        if let Err(e) = apm_core::ticket::set_field(&mut fm, "priority", &req.priority.to_string()) {
            failed.push(BatchFailure { id, error: e.to_string() });
            continue;
        }
        let updated = apm_core::ticket::Ticket { frontmatter: fm, body, path: ticket.path.clone() };
        let content = match updated.serialize() {
            Ok(c) => c,
            Err(e) => {
                failed.push(BatchFailure { id, error: e.to_string() });
                continue;
            }
        };
        let root_clone = root.clone();
        let result = tokio::task::spawn_blocking(move || {
            apm_core::git::commit_to_branch(&root_clone, &branch, &rel_path, &content, "ui: batch update priority")
        })
        .await?;
        match result {
            Ok(_) => succeeded.push(full_id),
            Err(e) => failed.push(BatchFailure { id: updated.frontmatter.id, error: e.to_string() }),
        }
    }
    Ok(Json(BatchResult { succeeded, failed }).into_response())
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
    let section_sets: Vec<(String, String)> = req.sections
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, v)| !v.trim().is_empty())
        .collect();
    let depends_on = req.depends_on;
    let (epic, target_branch) = match req.epic {
        None => (None, None),
        Some(ref short_id) => {
            match find_epic_branch(&root, short_id) {
                Some(branch) => (Some(short_id.clone()), Some(branch)),
                None => {
                    return Ok((
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": format!("no epic branch found for id {short_id}")})),
                    )
                        .into_response());
                }
            }
        }
    };
    let result = tokio::task::spawn_blocking(move || {
        let config = apm_core::config::Config::load(&root)?;
        let author = apm_core::config::resolve_identity(&root);
        apm_core::ticket::create(
            &root,
            &config,
            title,
            author,
            None,
            None,
            false,
            section_sets,
            epic,
            target_branch,
            depends_on,
            None,
        )
    })
    .await?;
    match result {
        Ok(ticket) => {
            let has_open_questions = !extract_section(&ticket.body, "Open questions").trim().is_empty();
            let has_pending_amendments = extract_section(&ticket.body, "Amendment requests").contains("- [ ]");
            let response = TicketResponse {
                frontmatter: ticket.frontmatter,
                body: ticket.body,
                has_open_questions,
                has_pending_amendments,
                blocking_deps: vec![],
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

async fn me_handler(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    if is_localhost(connect_info) {
        let username = match state.git_root() {
            Some(root) => apm_core::config::resolve_identity(root),
            None => "unassigned".to_string(),
        };
        return Json(serde_json::json!({"username": username}));
    }
    let username = find_session_username(&headers, &state.session_store)
        .unwrap_or_else(|| "unassigned".to_string());
    Json(serde_json::json!({"username": username}))
}

fn find_session_username(
    headers: &axum::http::HeaderMap,
    session_store: &auth::SessionStore,
) -> Option<String> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for part in cookie_header.split(';') {
        if let Ok(c) = cookie::Cookie::parse(part.trim().to_owned()) {
            if c.name() == "__Host-apm-session" {
                return session_store.lookup(c.value());
            }
        }
    }
    None
}

async fn otp_handler(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    body: axum::body::Bytes,
) -> Response {
    if !is_localhost(connect_info) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let parsed: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let username = match parsed.get("username").and_then(|v| v.as_str()) {
        Some(u) if !u.is_empty() => u.to_string(),
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    let otp = auth::generate_otp();
    state.otp_store.insert(&username, otp.clone());
    Json(serde_json::json!({"otp": otp})).into_response()
}

async fn register_page_handler() -> Response {
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("register.html"),
    )
        .into_response()
}

#[derive(serde::Deserialize)]
struct RegisterChallengeRequest {
    username: String,
    otp: String,
}

#[derive(serde::Serialize)]
struct RegisterChallengeResponse {
    reg_id: String,
    #[serde(rename = "publicKey")]
    public_key: serde_json::Value,
}

async fn register_challenge_handler(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Response {
    let req: RegisterChallengeRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_REQUEST, "missing or invalid fields").into_response(),
    };
    if req.username.is_empty() || req.otp.is_empty() {
        return (StatusCode::BAD_REQUEST, "missing or invalid fields").into_response();
    }
    match state.otp_store.validate(&req.username, &req.otp) {
        Ok(()) => {}
        Err(auth::OtpError::NotFound) => {
            return (StatusCode::BAD_REQUEST, "invalid OTP").into_response()
        }
        Err(auth::OtpError::Expired) => {
            return (StatusCode::BAD_REQUEST, "OTP expired").into_response()
        }
        Err(auth::OtpError::Invalid) => {
            return (StatusCode::BAD_REQUEST, "invalid OTP").into_response()
        }
    }
    let user_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, req.username.as_bytes());
    let (ccr, passkey_reg) = match state
        .webauthn_state
        .webauthn
        .start_passkey_registration(user_id, &req.username, &req.username, None)
    {
        Ok(pair) => pair,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("webauthn error: {e:?}"),
            )
                .into_response()
        }
    };
    let reg_id = {
        let bytes: [u8; 16] = rand::Rng::gen(&mut rand::thread_rng());
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    };
    {
        let mut pending = state.webauthn_state.pending.lock().unwrap();
        pending.insert(
            reg_id.clone(),
            webauthn_state::RegistrationSession {
                username: req.username,
                passkey_reg,
            },
        );
    }
    let public_key = match serde_json::to_value(&ccr.public_key) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("serialization error: {e}"),
            )
                .into_response()
        }
    };
    Json(RegisterChallengeResponse { reg_id, public_key }).into_response()
}

#[derive(serde::Deserialize)]
struct RegisterCompleteRequest {
    reg_id: String,
    response: webauthn_rs::prelude::RegisterPublicKeyCredential,
}

async fn register_complete_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterCompleteRequest>,
) -> Response {
    let session = {
        let mut pending = state.webauthn_state.pending.lock().unwrap();
        pending.remove(&req.reg_id)
    };
    let session = match session {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, "unknown reg_id").into_response(),
    };
    let passkey = match state
        .webauthn_state
        .webauthn
        .finish_passkey_registration(&req.response, &session.passkey_reg)
    {
        Ok(p) => p,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid WebAuthn response").into_response(),
    };
    state.credential_store.insert(&session.username, passkey);
    let token = auth::generate_token();
    state.session_store.insert(token.clone(), session.username, None);
    let cookie = format!(
        "__Host-apm-session={token}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=604800"
    );
    (
        StatusCode::OK,
        [(axum::http::header::SET_COOKIE, cookie)],
        Json(serde_json::json!({"status": "ok"})),
    )
        .into_response()
}

async fn login_page_handler() -> Response {
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("login.html"),
    )
        .into_response()
}

#[derive(serde::Deserialize)]
struct LoginChallengeRequest {
    username: String,
}

#[derive(serde::Serialize)]
struct LoginChallengeResponse {
    login_id: String,
    #[serde(rename = "publicKey")]
    public_key: serde_json::Value,
}

async fn login_challenge_handler(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Response {
    let req: LoginChallengeRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_REQUEST, "missing or invalid fields").into_response(),
    };
    if req.username.is_empty() {
        return (StatusCode::BAD_REQUEST, "missing or invalid fields").into_response();
    }
    let credentials = match state.credential_store.get(&req.username) {
        Some(c) => c,
        None => return (StatusCode::BAD_REQUEST, "no credentials for user").into_response(),
    };
    let (rcr, passkey_auth) = match state
        .webauthn_state
        .webauthn
        .start_passkey_authentication(&credentials)
    {
        Ok(pair) => pair,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("webauthn error: {e:?}"),
            )
                .into_response()
        }
    };
    let login_id = auth::generate_token();
    {
        let mut pending = state.webauthn_state.pending_auth.lock().unwrap();
        pending.insert(
            login_id.clone(),
            webauthn_state::AuthenticationSession {
                username: req.username,
                passkey_auth,
                created_at: std::time::Instant::now(),
            },
        );
    }
    let public_key = match serde_json::to_value(&rcr.public_key) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("serialization error: {e}"),
            )
                .into_response()
        }
    };
    Json(LoginChallengeResponse { login_id, public_key }).into_response()
}

#[derive(serde::Deserialize)]
struct LoginCompleteRequest {
    login_id: String,
    response: webauthn_rs::prelude::PublicKeyCredential,
}

async fn login_complete_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginCompleteRequest>,
) -> Response {
    let session = {
        let mut pending = state.webauthn_state.pending_auth.lock().unwrap();
        pending.remove(&req.login_id)
    };
    let session = match session {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, "unknown login_id").into_response(),
    };
    if session.created_at.elapsed() >= std::time::Duration::from_secs(300) {
        return (StatusCode::BAD_REQUEST, "login session expired").into_response();
    }
    let credentials = match state.credential_store.get(&session.username) {
        Some(c) => c,
        None => return (StatusCode::BAD_REQUEST, "no credentials for user").into_response(),
    };
    let _ = credentials;
    let auth_result = match state
        .webauthn_state
        .webauthn
        .finish_passkey_authentication(&req.response, &session.passkey_auth)
    {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid WebAuthn response").into_response(),
    };
    state.credential_store.update_credential(&session.username, &auth_result);
    let token = auth::generate_token();
    state.session_store.insert(token.clone(), session.username, None);
    let cookie = format!(
        "__Host-apm-session={token}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=604800"
    );
    (
        StatusCode::OK,
        [(axum::http::header::SET_COOKIE, cookie)],
        Json(serde_json::json!({"status": "ok"})),
    )
        .into_response()
}

async fn list_sessions_handler(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
) -> Response {
    if !is_localhost(connect_info) {
        return StatusCode::FORBIDDEN.into_response();
    }
    Json(state.session_store.list_active()).into_response()
}

async fn revoke_sessions_handler(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<auth::RevokeRequest>,
) -> Response {
    if !is_localhost(connect_info) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let revoked = state.session_store.revoke(&req);
    Json(auth::RevokeResponse { revoked }).into_response()
}

async fn serve_ui(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if let Some(file) = UI_DIR.get_file(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        (
            [(header::CONTENT_TYPE, mime.as_ref())],
            file.contents(),
        )
            .into_response()
    } else {
        let index = UI_DIR.get_file("index.html").expect("index.html missing from embedded UI");
        (
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            index.contents(),
        )
            .into_response()
    }
}

fn build_app(root: PathBuf) -> Router {
    let config = apm_core::config::Config::load(&root).expect("cannot load apm config");
    let tickets_dir = config.tickets.dir;
    let log_file = config.logging.file.map(|p| {
        if p.is_absolute() { p } else { root.join(&p) }
    });
    let sessions_path = root.join(".apm/sessions.json");
    let credentials_path = root.join(".apm/credentials.json");
    let wa_state = webauthn_state::WebauthnState::new(&config.server.origin)
        .expect("cannot initialise WebAuthn state");
    let state = Arc::new(AppState {
        source: TicketSource::Git(root, tickets_dir),
        work_engine: work::new_engine_state(),
        log_file,
        max_concurrent_override: Arc::new(tokio::sync::Mutex::new(None)),
        otp_store: auth::OtpStore::new(),
        session_store: auth::SessionStore::load(sessions_path),
        webauthn_state: Arc::new(wa_state),
        credential_store: credential_store::CredentialStore::load(credentials_path),
    });
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/sync", post(sync_handler))
        .route("/api/clean", post(clean_handler))
        .route("/api/tickets", get(list_tickets).post(create_ticket))
        .route("/api/tickets/:id", get(get_ticket).patch(patch_ticket))
        .route("/api/tickets/:id/body", put(put_body))
        .route("/api/tickets/:id/transition", post(transition_ticket))
        .route("/api/tickets/batch/transition", post(batch_transition))
        .route("/api/tickets/batch/priority", post(batch_priority))
        .route("/api/queue", get(queue::queue_handler))
        .route("/api/workers", get(workers::workers_handler))
        .route("/api/workers/:pid", axum::routing::delete(workers::delete_worker))
        .route("/api/work/status", get(work::get_work_status))
        .route("/api/work/start", post(work::post_work_start))
        .route("/api/work/stop", post(work::post_work_stop))
        .route("/api/work/dry-run", get(work::get_work_dry_run))
        .route("/api/agents/config", get(agents::get_agents_config).patch(agents::patch_agents_config))
        .route("/api/log/stream", get(log::stream_handler))
        .route("/api/epics", get(list_epics).post(create_epic))
        .route("/api/epics/:id", get(get_epic))
        .route("/api/me", get(me_handler))
        .route("/api/auth/otp", post(otp_handler))
        .route("/api/auth/sessions", get(list_sessions_handler).delete(revoke_sessions_handler))
        .route("/register", get(register_page_handler))
        .route("/api/auth/register/challenge", post(register_challenge_handler))
        .route("/api/auth/register/complete", post(register_complete_handler))
        .route("/login", get(login_page_handler))
        .route("/api/auth/login/challenge", post(login_challenge_handler))
        .route("/api/auth/login/complete", post(login_complete_handler))
        .fallback(serve_ui)
        .with_state(state)
}

#[cfg(test)]
fn default_webauthn_state() -> Arc<webauthn_state::WebauthnState> {
    Arc::new(
        webauthn_state::WebauthnState::new("http://localhost:3000")
            .expect("test webauthn state"),
    )
}

#[cfg(test)]
fn build_app_with_tickets(tickets: Vec<apm_core::ticket::Ticket>) -> Router {
    let state = Arc::new(AppState {
        source: TicketSource::InMemory(tickets),
        work_engine: work::new_engine_state(),
        log_file: None,
        max_concurrent_override: Arc::new(tokio::sync::Mutex::new(None)),
        otp_store: auth::OtpStore::new(),
        session_store: auth::SessionStore::load(PathBuf::new()),
        webauthn_state: default_webauthn_state(),
        credential_store: credential_store::CredentialStore::load(PathBuf::new()),
    });
    Router::new()
        .route("/api/sync", post(sync_handler))
        .route("/api/clean", post(clean_handler))
        .route("/api/tickets", get(list_tickets).post(create_ticket))
        .route("/api/tickets/:id", get(get_ticket).patch(patch_ticket))
        .route("/api/tickets/:id/body", put(put_body))
        .route("/api/tickets/:id/transition", post(transition_ticket))
        .route("/api/epics", get(list_epics).post(create_epic))
        .route("/api/epics/:id", get(get_epic))
        .route("/api/me", get(me_handler))
        .with_state(state)
}

#[cfg(test)]
pub fn build_app_in_memory_with_workers(tickets: Vec<apm_core::ticket::Ticket>) -> Router {
    let state = Arc::new(AppState {
        source: TicketSource::InMemory(tickets),
        work_engine: work::new_engine_state(),
        log_file: None,
        max_concurrent_override: Arc::new(tokio::sync::Mutex::new(None)),
        otp_store: auth::OtpStore::new(),
        session_store: auth::SessionStore::load(PathBuf::new()),
        webauthn_state: default_webauthn_state(),
        credential_store: credential_store::CredentialStore::load(PathBuf::new()),
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
        log_file: None,
        max_concurrent_override: Arc::new(tokio::sync::Mutex::new(None)),
        otp_store: auth::OtpStore::new(),
        session_store: auth::SessionStore::load(PathBuf::new()),
        webauthn_state: default_webauthn_state(),
        credential_store: credential_store::CredentialStore::load(PathBuf::new()),
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
        log_file: None,
        max_concurrent_override: Arc::new(tokio::sync::Mutex::new(None)),
        otp_store: auth::OtpStore::new(),
        session_store: auth::SessionStore::load(PathBuf::new()),
        webauthn_state: default_webauthn_state(),
        credential_store: credential_store::CredentialStore::load(PathBuf::new()),
    });
    Router::new()
        .route("/api/work/status", get(work::get_work_status))
        .route("/api/work/start", post(work::post_work_start))
        .route("/api/work/stop", post(work::post_work_stop))
        .route("/api/work/dry-run", get(work::get_work_dry_run))
        .route("/api/agents/config", get(agents::get_agents_config).patch(agents::patch_agents_config))
        .with_state(state)
}

#[cfg(test)]
fn build_app_for_auth_tests(
    root: PathBuf,
    otp_store: auth::OtpStore,
    session_store: auth::SessionStore,
) -> Router {
    let state = Arc::new(AppState {
        source: TicketSource::Git(root.clone(), root.join("tickets")),
        work_engine: work::new_engine_state(),
        log_file: None,
        max_concurrent_override: Arc::new(tokio::sync::Mutex::new(None)),
        otp_store,
        session_store,
        webauthn_state: default_webauthn_state(),
        credential_store: credential_store::CredentialStore::load(PathBuf::new()),
    });
    Router::new()
        .route("/api/me", get(me_handler))
        .route("/api/auth/otp", post(otp_handler))
        .route("/api/auth/sessions", get(list_sessions_handler).delete(revoke_sessions_handler))
        .with_state(state)
}

#[derive(clap::Parser)]
struct Cli {
    #[arg(long, value_name = "MODE", num_args = 0..=1, default_missing_value = "acme")]
    tls: Option<TlsMode>,
    #[arg(long, value_name = "DOMAIN")]
    tls_domain: Option<String>,
    #[arg(long, value_name = "EMAIL")]
    tls_email: Option<String>,
    #[arg(long, value_name = "DIR")]
    tls_cert_dir: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    tls_cert: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    tls_key: Option<PathBuf>,
    #[arg(long, value_name = "PORT")]
    port: Option<u16>,
    #[arg(long, value_name = "ADDR")]
    bind: Option<String>,
}

#[derive(clap::ValueEnum, Clone)]
enum TlsMode {
    Acme,
    #[value(name = "self-signed")]
    SelfSigned,
}

fn add_hsts(app: Router) -> Router {
    use axum::http::HeaderValue;
    use axum::http::header::STRICT_TRANSPORT_SECURITY;
    use tower_http::set_header::SetResponseHeaderLayer;
    app.layer(SetResponseHeaderLayer::if_not_present(
        STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=63072000; includeSubDomains"),
    ))
}

#[tokio::main]
async fn main() {
    use clap::Parser;
    let cli = Cli::parse();

    let root = std::env::current_dir().unwrap();
    let app = build_app(root);

    let is_tls = cli.tls.is_some() || cli.tls_cert.is_some();
    let default_port: u16 = if is_tls { 443 } else { 3000 };
    let port = cli.port.unwrap_or(default_port);
    let bind = cli.bind.as_deref().unwrap_or("0.0.0.0");
    let addr: SocketAddr = format!("{bind}:{port}").parse().expect("invalid bind address");

    match (cli.tls, &cli.tls_cert) {
        (None, None) => {
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            println!("Listening on http://{addr}");
            axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .unwrap();
        }
        (None, Some(cert)) => {
            let key = cli.tls_key.as_ref().expect("--tls-key is required when using --tls-cert");
            let config = tls::custom_cert_config(cert, key)
                .expect("failed to load TLS certificate");
            tls::serve_tls(addr, add_hsts(app), config).await;
        }
        (Some(TlsMode::SelfSigned), _) => {
            let domain = cli.tls_domain.as_deref().unwrap_or("localhost");
            eprintln!("Warning: self-signed certificate for '{domain}' — not trusted by browsers");
            let config = tls::self_signed_config(domain)
                .expect("failed to generate self-signed certificate");
            tls::serve_tls(addr, add_hsts(app), config).await;
        }
        (Some(TlsMode::Acme), _) => {
            let domain = cli.tls_domain.clone()
                .expect("--tls-domain is required with --tls (ACME mode)");
            let email = cli.tls_email.clone()
                .expect("--tls-email is required with --tls (ACME mode)");
            let cache_dir = cli.tls_cert_dir.clone().unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".apm/certs")
            });
            tls::serve_acme(addr, add_hsts(app), domain, email, cache_dir).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apm_core::ticket::{Frontmatter, Ticket};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn hsts_header_present_when_tls_enabled() {
        let app = add_hsts(build_app_with_tickets(vec![]));
        let response = app
            .oneshot(Request::builder().uri("/api/tickets").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            response.headers().get("strict-transport-security").map(|v| v.to_str().unwrap()),
            Some("max-age=63072000; includeSubDomains"),
        );
    }

    #[tokio::test]
    async fn no_hsts_header_without_tls_wrapper() {
        let app = build_app_with_tickets(vec![]);
        let response = app
            .oneshot(Request::builder().uri("/api/tickets").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(response.headers().get("strict-transport-security").is_none());
    }

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
                owner: None,
                branch: None,
                created_at: None,
                updated_at: None,
                focus_section: None,
                epic: None,
                target_branch: None,
                depends_on: None,
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
    async fn list_tickets_excludes_closed_by_default() {
        let mut closed_ticket = fake_ticket("aaaabbbb-closed-one", "Closed ticket");
        closed_ticket.frontmatter.state = "closed".to_string();
        let open_ticket = fake_ticket("ccccdddd-open-one", "Open ticket");
        let app = build_app_with_tickets(vec![closed_ticket, open_ticket]);
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
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"].as_str().unwrap(), "ccccdddd-open-one");
    }

    #[tokio::test]
    async fn list_tickets_includes_closed_when_requested() {
        let mut closed_ticket = fake_ticket("aaaabbbb-closed-two", "Closed ticket");
        closed_ticket.frontmatter.state = "closed".to_string();
        let open_ticket = fake_ticket("ccccdddd-open-two", "Open ticket");
        let app = build_app_with_tickets(vec![closed_ticket, open_ticket]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets?include_closed=true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json.as_array().unwrap().len(), 2);
    }

    #[test]
    fn extract_section_finds_content() {
        let body = "## Spec\n\n### Open questions\n\nIs this real?\n\n### Amendment requests\n\n- [ ] Fix it\n";
        assert_eq!(extract_section(body, "Open questions").trim(), "Is this real?");
        assert_eq!(extract_section(body, "Amendment requests").trim(), "- [ ] Fix it");
    }

    #[test]
    fn extract_section_missing_returns_empty() {
        let body = "## Spec\n\nNo sections here.\n";
        assert_eq!(extract_section(body, "Open questions"), "");
    }

    #[tokio::test]
    async fn list_tickets_includes_badge_fields() {
        let mut ticket_with_question = fake_ticket("11112222-badge-test-q", "Question ticket");
        ticket_with_question.body = "### Open questions\n\nWhat is this?\n".to_string();

        let mut ticket_with_amendment = fake_ticket("33334444-badge-test-a", "Amendment ticket");
        ticket_with_amendment.body = "### Amendment requests\n\n- [ ] Fix the thing\n".to_string();

        let mut ticket_clean = fake_ticket("55556666-badge-test-c", "Clean ticket");
        ticket_clean.body = "### Open questions\n\n\n### Amendment requests\n\n- [x] Done\n".to_string();

        let app = build_app_with_tickets(vec![ticket_with_question, ticket_with_amendment, ticket_clean]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();

        let q = arr.iter().find(|t| t["id"] == "11112222-badge-test-q").unwrap();
        assert_eq!(q["has_open_questions"], true);
        assert_eq!(q["has_pending_amendments"], false);

        let a = arr.iter().find(|t| t["id"] == "33334444-badge-test-a").unwrap();
        assert_eq!(a["has_open_questions"], false);
        assert_eq!(a["has_pending_amendments"], true);

        let c = arr.iter().find(|t| t["id"] == "55556666-badge-test-c").unwrap();
        assert_eq!(c["has_open_questions"], false);
        assert_eq!(c["has_pending_amendments"], false);
    }

    #[tokio::test]
    async fn list_tickets_blocking_deps() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();

        // git init
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["config", "user.email", "test@test.com"],
            vec!["config", "user.name", "test"],
        ] {
            std::process::Command::new("git")
                .args(&args)
                .current_dir(&p)
                .status()
                .unwrap();
        }

        // config with `implemented` satisfying deps
        std::fs::write(
            p.join("apm.toml"),
            r#"[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"

[[workflow.states]]
id             = "implemented"
label          = "Implemented"
satisfies_deps = true
"#,
        )
        .unwrap();

        std::fs::create_dir_all(p.join("tickets")).unwrap();

        for args in [
            vec!["add", "apm.toml"],
            vec!["-c", "commit.gpgsign=false", "commit", "-m", "init"],
        ] {
            std::process::Command::new("git")
                .args(&args)
                .current_dir(&p)
                .env("GIT_AUTHOR_NAME", "test")
                .env("GIT_AUTHOR_EMAIL", "test@test.com")
                .env("GIT_COMMITTER_NAME", "test")
                .env("GIT_COMMITTER_EMAIL", "test@test.com")
                .status()
                .unwrap();
        }

        // helper: commit a ticket file to its own branch
        let commit_ticket = |slug: &str, content: &str| {
            let branch = format!("ticket/{slug}");
            let filename = format!("tickets/{slug}.md");
            let file_path = p.join(&filename);
            std::fs::create_dir_all(p.join("tickets")).unwrap();
            std::fs::write(&file_path, content).unwrap();
            for args in [
                vec!["checkout", "-b", branch.as_str()],
                vec!["add", filename.as_str()],
                vec!["-c", "commit.gpgsign=false", "commit", "-m", "add ticket"],
            ] {
                std::process::Command::new("git")
                    .args(&args)
                    .current_dir(&p)
                    .env("GIT_AUTHOR_NAME", "test")
                    .env("GIT_AUTHOR_EMAIL", "test@test.com")
                    .env("GIT_COMMITTER_NAME", "test")
                    .env("GIT_COMMITTER_EMAIL", "test@test.com")
                    .status()
                    .unwrap();
            }
            std::process::Command::new("git")
                .args(["checkout", "main"])
                .current_dir(&p)
                .status()
                .unwrap();
        };

        // dep-satisfied: in `implemented` state (satisfies_deps=true)
        commit_ticket(
            "aabbccdd-dep-satisfied",
            "+++\nid = \"aabbccdd-dep-satisfied\"\ntitle = \"Dep Satisfied\"\nstate = \"implemented\"\n+++\n\n",
        );

        // dep-blocking: in `in_progress` state (not satisfying)
        commit_ticket(
            "11223344-dep-blocking",
            "+++\nid = \"11223344-dep-blocking\"\ntitle = \"Dep Blocking\"\nstate = \"in_progress\"\n+++\n\n",
        );

        // ticket with no depends_on
        commit_ticket(
            "aaaaaaaa-no-deps",
            "+++\nid = \"aaaaaaaa-no-deps\"\ntitle = \"No Deps\"\nstate = \"ready\"\n+++\n\n",
        );

        // ticket depending on satisfied dep → blocking_deps should be []
        commit_ticket(
            "bbbbbbbb-dep-on-satisfied",
            "+++\nid = \"bbbbbbbb-dep-on-satisfied\"\ntitle = \"Dep On Satisfied\"\nstate = \"ready\"\ndepends_on = [\"aabbccdd-dep-satisfied\"]\n+++\n\n",
        );

        // ticket depending on blocking dep → blocking_deps should be non-empty
        commit_ticket(
            "cccccccc-dep-on-blocking",
            "+++\nid = \"cccccccc-dep-on-blocking\"\ntitle = \"Dep On Blocking\"\nstate = \"ready\"\ndepends_on = [\"11223344-dep-blocking\"]\n+++\n\n",
        );

        let app = build_app(p.clone());
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
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();

        let no_deps = arr.iter().find(|t| t["id"] == "aaaaaaaa-no-deps").unwrap();
        assert_eq!(no_deps["blocking_deps"], serde_json::json!([]));

        let on_satisfied = arr.iter().find(|t| t["id"] == "bbbbbbbb-dep-on-satisfied").unwrap();
        assert_eq!(on_satisfied["blocking_deps"], serde_json::json!([]));

        let on_blocking = arr.iter().find(|t| t["id"] == "cccccccc-dep-on-blocking").unwrap();
        let blocking = on_blocking["blocking_deps"].as_array().unwrap();
        assert_eq!(blocking.len(), 1);
        assert_eq!(blocking[0]["id"], "11223344-dep-blocking");
        assert_eq!(blocking[0]["state"], "in_progress");
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
    async fn create_ticket_with_epic_and_depends_on_in_memory_returns_501() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tickets")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"title":"t","epic":"ab12cd34","depends_on":["cd56ef78"]}"#,
                    ))
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
            None,
            None,
            None,
            None,
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

    #[tokio::test]
    async fn create_ticket_with_depends_on_persists_to_git() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);

        let app = build_app(p.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tickets")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"dep ticket","depends_on":["ab12cd34"]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["depends_on"][0], "ab12cd34");

        let branch = json["branch"].as_str().unwrap().to_string();
        let id = json["id"].as_str().unwrap().to_string();
        let config = apm_core::config::Config::load(&p).unwrap();
        let rel_path = format!("{}/{}-dep-ticket.md", config.tickets.dir.to_string_lossy(), id);
        let content = apm_core::git::read_from_branch(&p, &branch, &rel_path).unwrap();
        assert!(content.contains(r#"depends_on = ["ab12cd34"]"#), "expected depends_on in: {content}");
    }

    #[tokio::test]
    async fn create_ticket_with_unknown_epic_returns_400() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);

        let app = build_app(p.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tickets")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"orphan","epic":"deadbeef"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_ticket_with_epic_resolves_target_branch() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);

        // Create an epic branch locally
        std::process::Command::new("git")
            .args(["-c", "commit.gpgsign=false", "branch", "epic/ab12cd34-foo"])
            .current_dir(&p)
            .status()
            .unwrap();

        let app = build_app(p.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tickets")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"epic child","epic":"ab12cd34"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["epic"], "ab12cd34");
        assert_eq!(json["target_branch"], "epic/ab12cd34-foo");
    }

    #[tokio::test]
    async fn list_epics_in_memory_returns_501() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/epics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn create_epic_missing_title_returns_400() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);
        let app = build_app(p.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/epics")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_epic_empty_title_returns_400() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);
        let app = build_app(p.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/epics")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_epic_in_memory_returns_501() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/epics")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"My Epic"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn get_epic_in_memory_returns_501() {
        let app = build_app_with_tickets(test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/epics/ab12cd34")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn get_epic_not_found_returns_404() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);
        let app = build_app(p.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/epics/deadbeef")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_epics_empty_returns_empty_array() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);
        let app = build_app(p.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/epics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json, serde_json::json!([]));
    }

    #[tokio::test]
    async fn create_epic_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);

        let app = build_app(p.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/epics")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"My Epic"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["state"], "empty");
        assert_eq!(json["title"], "My Epic");
        assert!(json["ticket_counts"].as_object().unwrap().is_empty());
        let epic_id = json["id"].as_str().unwrap().to_string();

        // list should include the new epic
        let app2 = build_app(p.clone());
        let response2 = app2
            .oneshot(
                Request::builder()
                    .uri("/api/epics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response2.status(), StatusCode::OK);
        let bytes2 = response2.into_body().collect().await.unwrap().to_bytes();
        let list: serde_json::Value = serde_json::from_slice(&bytes2).unwrap();
        assert_eq!(list.as_array().unwrap().len(), 1);
        assert_eq!(list[0]["id"], epic_id);

        // get by id
        let app3 = build_app(p.clone());
        let response3 = app3
            .oneshot(
                Request::builder()
                    .uri(format!("/api/epics/{epic_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response3.status(), StatusCode::OK);
        let bytes3 = response3.into_body().collect().await.unwrap().to_bytes();
        let detail: serde_json::Value = serde_json::from_slice(&bytes3).unwrap();
        assert_eq!(detail["id"], epic_id);
        assert!(detail["tickets"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_tickets_author_field_always_present() {
        let ticket = fake_ticket("aaaabbbb-no-author", "No author ticket");
        assert!(ticket.frontmatter.author.is_none());
        let app = build_app_with_tickets(vec![ticket]);
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
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["author"], "unassigned");
    }

    #[tokio::test]
    async fn list_tickets_author_filter() {
        let mut alice_ticket = fake_ticket("aaaabbbb-alice-ticket", "Alice ticket");
        alice_ticket.frontmatter.author = Some("alice".to_string());
        let mut bob_ticket = fake_ticket("ccccdddd-bob-ticket", "Bob ticket");
        bob_ticket.frontmatter.author = Some("bob".to_string());
        let app = build_app_with_tickets(vec![alice_ticket, bob_ticket]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets?author=alice")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "aaaabbbb-alice-ticket");
        assert_eq!(arr[0]["author"], "alice");
    }

    #[tokio::test]
    async fn list_tickets_author_unassigned_filter() {
        let unassigned_ticket = fake_ticket("aaaabbbb-unassigned", "Unassigned ticket");
        assert!(unassigned_ticket.frontmatter.author.is_none());
        let mut alice_ticket = fake_ticket("ccccdddd-alice", "Alice ticket");
        alice_ticket.frontmatter.author = Some("alice".to_string());
        let app = build_app_with_tickets(vec![unassigned_ticket, alice_ticket]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets?author=unassigned")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "aaaabbbb-unassigned");
        assert_eq!(arr[0]["author"], "unassigned");
    }

    #[tokio::test]
    async fn get_ticket_author_field_always_present() {
        let ticket = fake_ticket("aaaabbbb-no-author-detail", "No author ticket");
        assert!(ticket.frontmatter.author.is_none());
        let app = build_app_with_tickets(vec![ticket]);
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
        assert_eq!(json["author"], "unassigned");
    }

    #[tokio::test]
    async fn me_handler_returns_unassigned_when_no_local_toml() {
        let app = build_app_with_tickets(vec![]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["username"], "unassigned");
    }

    #[tokio::test]
    async fn post_otp_from_localhost_returns_otp() {
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let dir = tempfile::tempdir().unwrap();
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let mut req = Request::builder()
            .uri("/api/auth/otp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"username":"alice"}"#))
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let otp = json["otp"].as_str().unwrap();
        assert_eq!(otp.len(), 8);
        assert!(otp.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[tokio::test]
    async fn post_otp_from_remote_returns_403() {
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let dir = tempfile::tempdir().unwrap();
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/otp")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"alice"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn post_otp_with_malformed_body_returns_400() {
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let dir = tempfile::tempdir().unwrap();
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let mut req = Request::builder()
            .uri("/api/auth/otp")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from("not-json"))
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_me_localhost_with_local_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".apm")).unwrap();
        std::fs::write(dir.path().join(".apm/local.toml"), "username = \"alice\"\n").unwrap();
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let mut req = Request::builder()
            .uri("/api/me")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["username"], "alice");
    }

    #[tokio::test]
    async fn get_me_localhost_without_local_toml() {
        let dir = tempfile::tempdir().unwrap();
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let mut req = Request::builder()
            .uri("/api/me")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["username"], "unassigned");
    }

    #[tokio::test]
    async fn get_me_remote_with_valid_session() {
        let dir = tempfile::tempdir().unwrap();
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        session_store.insert("testtoken".to_string(), "bob".to_string(), None);
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/me")
                    .header("cookie", "__Host-apm-session=testtoken")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["username"], "bob");
    }

    #[tokio::test]
    async fn get_me_remote_with_expired_session() {
        let dir = tempfile::tempdir().unwrap();
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let expired = chrono::Utc::now() - chrono::Duration::days(10);
        session_store.insert_expiring_at("exptoken".to_string(), "eve".to_string(), expired);
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/me")
                    .header("cookie", "__Host-apm-session=exptoken")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["username"], "unassigned");
    }

    #[tokio::test]
    async fn get_me_remote_with_no_cookie() {
        let dir = tempfile::tempdir().unwrap();
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["username"], "unassigned");
    }

    fn build_app_for_webauthn_tests(
        otp_store: auth::OtpStore,
        session_store: auth::SessionStore,
    ) -> Router {
        let dir = tempfile::TempDir::new().unwrap();
        let state = Arc::new(AppState {
            source: TicketSource::InMemory(vec![]),
            work_engine: work::new_engine_state(),
            log_file: None,
            max_concurrent_override: Arc::new(tokio::sync::Mutex::new(None)),
            otp_store,
            session_store,
            webauthn_state: default_webauthn_state(),
            credential_store: credential_store::CredentialStore::load(
                dir.path().join("credentials.json"),
            ),
        });
        // keep dir alive by leaking (tests are short-lived)
        std::mem::forget(dir);
        Router::new()
            .route("/register", get(register_page_handler))
            .route("/api/auth/register/challenge", post(register_challenge_handler))
            .route("/api/auth/register/complete", post(register_complete_handler))
            .route("/api/me", get(me_handler))
            .with_state(state)
    }

    #[tokio::test]
    async fn register_page_returns_200_html() {
        let app = build_app_for_webauthn_tests(
            auth::OtpStore::new(),
            auth::SessionStore::load(PathBuf::new()),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/register")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let ct = response.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("text/html"), "unexpected content-type: {ct}");
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let html = std::str::from_utf8(&bytes).unwrap();
        assert!(html.contains("username"), "expected username field in HTML");
        assert!(html.contains("otp") || html.contains("one-time"), "expected OTP field in HTML");
    }

    #[tokio::test]
    async fn challenge_missing_fields_returns_400() {
        let app = build_app_for_webauthn_tests(
            auth::OtpStore::new(),
            auth::SessionStore::load(PathBuf::new()),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/register/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn challenge_invalid_otp_returns_400() {
        let otp_store = auth::OtpStore::new();
        otp_store.insert("alice", "VALIDOTP".to_string());
        let app = build_app_for_webauthn_tests(
            otp_store,
            auth::SessionStore::load(PathBuf::new()),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/register/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"alice","otp":"WRONGOTP"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn challenge_no_otp_for_user_returns_400() {
        let app = build_app_for_webauthn_tests(
            auth::OtpStore::new(),
            auth::SessionStore::load(PathBuf::new()),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/register/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"alice","otp":"ANYTHING"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn challenge_valid_otp_returns_200_with_reg_id_and_public_key() {
        let otp_store = auth::OtpStore::new();
        otp_store.insert("alice", "VALIDOTP".to_string());
        let app = build_app_for_webauthn_tests(
            otp_store,
            auth::SessionStore::load(PathBuf::new()),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/register/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"alice","otp":"VALIDOTP"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["reg_id"].is_string(), "reg_id missing");
        assert!(json["publicKey"].is_object(), "publicKey missing");
        assert!(json["publicKey"]["challenge"].is_string(), "challenge missing");
        assert!(json["publicKey"]["rp"].is_object(), "rp missing");
        assert!(json["publicKey"]["user"].is_object(), "user missing");
    }

    #[tokio::test]
    async fn challenge_otp_consumed_second_use_returns_400() {
        let otp_store = auth::OtpStore::new();
        otp_store.insert("alice", "ONCEONLY".to_string());
        let app = build_app_for_webauthn_tests(
            otp_store,
            auth::SessionStore::load(PathBuf::new()),
        );
        // First call consumes the OTP
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/register/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"alice","otp":"ONCEONLY"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        // Second call with same OTP should fail
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/register/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"alice","otp":"ONCEONLY"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn complete_unknown_reg_id_returns_400() {
        let app = build_app_for_webauthn_tests(
            auth::OtpStore::new(),
            auth::SessionStore::load(PathBuf::new()),
        );
        let body = serde_json::json!({
            "reg_id": "nonexistent",
            "response": {
                "id": "dGVzdA",
                "rawId": "dGVzdA",
                "type": "public-key",
                "response": {
                    "clientDataJSON": "dGVzdA",
                    "attestationObject": "dGVzdA"
                }
            }
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/register/complete")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    fn build_app_for_login_tests(
        session_store: auth::SessionStore,
        credential_store: credential_store::CredentialStore,
    ) -> Router {
        let state = Arc::new(AppState {
            source: TicketSource::InMemory(vec![]),
            work_engine: work::new_engine_state(),
            log_file: None,
            max_concurrent_override: Arc::new(tokio::sync::Mutex::new(None)),
            otp_store: auth::OtpStore::new(),
            session_store,
            webauthn_state: default_webauthn_state(),
            credential_store,
        });
        Router::new()
            .route("/login", get(login_page_handler))
            .route("/api/auth/login/challenge", post(login_challenge_handler))
            .route("/api/auth/login/complete", post(login_complete_handler))
            .route("/api/me", get(me_handler))
            .with_state(state)
    }

    #[tokio::test]
    async fn login_page_returns_200_html() {
        let app = build_app_for_login_tests(
            auth::SessionStore::load(PathBuf::new()),
            credential_store::CredentialStore::load(PathBuf::new()),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let ct = response.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("text/html"), "unexpected content-type: {ct}");
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let html = std::str::from_utf8(&bytes).unwrap();
        assert!(html.contains("username"), "expected username field in HTML");
        assert!(html.contains("Sign in"), "expected sign-in button in HTML");
    }

    #[tokio::test]
    async fn login_challenge_malformed_body_returns_400() {
        let app = build_app_for_login_tests(
            auth::SessionStore::load(PathBuf::new()),
            credential_store::CredentialStore::load(PathBuf::new()),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(b"not json".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn login_challenge_unknown_user_returns_400() {
        let app = build_app_for_login_tests(
            auth::SessionStore::load(PathBuf::new()),
            credential_store::CredentialStore::load(PathBuf::new()),
        );
        let body = serde_json::json!({"username": "nobody"});
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn login_complete_unknown_login_id_returns_400() {
        let app = build_app_for_login_tests(
            auth::SessionStore::load(PathBuf::new()),
            credential_store::CredentialStore::load(PathBuf::new()),
        );
        let body = serde_json::json!({
            "login_id": "nonexistent",
            "response": {
                "id": "dGVzdA",
                "rawId": "dGVzdA",
                "type": "public-key",
                "response": {
                    "clientDataJSON": "dGVzdA",
                    "authenticatorData": "dGVzdA",
                    "signature": "dGVzdA",
                    "userHandle": null
                }
            }
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login/complete")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_sessions_from_remote_returns_403() {
        let dir = tempfile::tempdir().unwrap();
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/auth/sessions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn get_sessions_from_localhost_returns_active_only() {
        let dir = tempfile::tempdir().unwrap();
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let expired = chrono::Utc::now() - chrono::Duration::days(10);
        session_store.insert_expiring_at("exp".to_string(), "eve".to_string(), expired);
        session_store.insert("active".to_string(), "alice".to_string(), None);
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let mut req = Request::builder()
            .uri("/api/auth/sessions")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["username"], "alice");
    }

    #[tokio::test]
    async fn delete_sessions_from_remote_returns_403() {
        let dir = tempfile::tempdir().unwrap();
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/auth/sessions")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"all":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn delete_sessions_all_from_localhost() {
        let dir = tempfile::tempdir().unwrap();
        let otp_store = auth::OtpStore::new();
        let session_store = auth::SessionStore::load(PathBuf::new());
        session_store.insert("t1".to_string(), "alice".to_string(), None);
        session_store.insert("t2".to_string(), "bob".to_string(), None);
        let app = build_app_for_auth_tests(dir.path().to_path_buf(), otp_store, session_store);
        let mut req = Request::builder()
            .method("DELETE")
            .uri("/api/auth/sessions")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"all":true}"#))
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["revoked"], 2);
    }

    #[tokio::test]
    async fn list_tickets_owner_field_present() {
        let mut ticket = fake_ticket("aaaabbbb-owner-present", "Owner present");
        ticket.frontmatter.owner = Some("alice".to_string());
        let app = build_app_with_tickets(vec![ticket]);
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
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr[0]["owner"], "alice");
    }

    #[tokio::test]
    async fn list_tickets_owner_field_absent() {
        let ticket = fake_ticket("bbbbcccc-owner-absent", "Owner absent");
        let app = build_app_with_tickets(vec![ticket]);
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
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert!(arr[0].get("owner").is_none() || arr[0]["owner"].is_null());
    }

    #[tokio::test]
    async fn list_tickets_owner_filter() {
        let mut alice_ticket = fake_ticket("ccccdddd-owner-alice", "Alice ticket");
        alice_ticket.frontmatter.owner = Some("alice".to_string());
        let mut bob_ticket = fake_ticket("ddddeee0-owner-bob", "Bob ticket");
        bob_ticket.frontmatter.owner = Some("bob".to_string());
        let app = build_app_with_tickets(vec![alice_ticket, bob_ticket]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets?owner=alice")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "ccccdddd-owner-alice");
    }

    #[tokio::test]
    async fn list_tickets_owner_unassigned_filter() {
        let mut owned_ticket = fake_ticket("eeeeffff-owner-set", "Owned ticket");
        owned_ticket.frontmatter.owner = Some("alice".to_string());
        let unowned_ticket = fake_ticket("ffff0000-owner-none", "Unowned ticket");
        let app = build_app_with_tickets(vec![owned_ticket, unowned_ticket]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets?owner=unassigned")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "ffff0000-owner-none");
    }
}
