use axum::{
    extract::{ConnectInfo, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use include_dir::{include_dir, Dir};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

static UI_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../apm-ui/dist");

mod agents;
mod handlers;
mod tls;
mod auth;
mod credential_store;
mod log;
mod models;
mod queue;
mod util;
mod webauthn_state;
mod work;
mod workers;

use models::*;
use handlers::tickets::{
    batch_priority, batch_transition, create_ticket, get_ticket, list_tickets,
    patch_ticket, put_body, transition_ticket,
};
#[cfg(test)]
use handlers::tickets::extract_section;

#[allow(dead_code)] // InMemory is constructed in tests but matched in shared code
pub(crate) enum TicketSource {
    Git(PathBuf, PathBuf),
    InMemory(Vec<apm_core::ticket::Ticket>),
}

pub(crate) struct AppState {
    pub(crate) source: TicketSource,
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
        let mut _sync_warnings: Vec<String> = Vec::new();
        apm_core::git::sync_local_ticket_refs(&root, &mut _sync_warnings);
        let branches = apm_core::git::ticket_branches(&root)
            .map(|b| b.len())
            .unwrap_or(0);
        let closed = match apm_core::config::Config::load(&root) {
            Ok(config) => {
                let _ = apm_core::git::push_default_branch(&root, &config.project.default_branch);
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
    body: Option<Json<CleanRequest>>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let dry_run   = req.dry_run.unwrap_or(false);
    let force     = req.force.unwrap_or(false);
    let branches  = req.branches.unwrap_or(false);
    let remote    = req.remote.unwrap_or(false);
    let untracked = req.untracked.unwrap_or(false);
    let epics     = req.epics.unwrap_or(false);
    let older_than = req.older_than;

    if remote && older_than.is_none() {
        return Ok((StatusCode::BAD_REQUEST, "remote requires older_than").into_response());
    }

    let (log, removed) = util::blocking(move || -> anyhow::Result<(Vec<String>, usize)> {
        let mut log: Vec<String> = Vec::new();
        let mut count = 0usize;

        let config = apm_core::config::Config::load(&root)?;
        let (candidates, dirty, candidate_warnings) =
            apm_core::clean::candidates(&root, &config, force, untracked, dry_run)?;
        for w in &candidate_warnings {
            log.push(w.clone());
        }

        for dw in &dirty {
            if !dw.modified_tracked.is_empty() {
                for f in &dw.modified_tracked {
                    log.push(format!("  M {}", f.display()));
                }
                log.push(format!(
                    "warning: {} has modified tracked files — manual cleanup required — skipping",
                    dw.branch
                ));
            } else {
                for f in &dw.other_untracked {
                    log.push(format!("  ? {}", f.display()));
                }
                log.push(format!(
                    "warning: {} has untracked files — re-run with --untracked to remove — skipping",
                    dw.branch
                ));
            }
        }

        for candidate in &candidates {
            if dry_run {
                if let Some(ref path) = candidate.worktree {
                    log.push(format!(
                        "would remove worktree {} (ticket #{}, state: {})",
                        path.display(),
                        candidate.ticket_id,
                        candidate.reason
                    ));
                }
                if branches && candidate.local_branch_exists && (candidate.branch_merged || force) {
                    log.push(format!(
                        "would remove branch {} (state: {})",
                        candidate.branch, candidate.reason
                    ));
                } else if branches && candidate.local_branch_exists && !candidate.branch_merged {
                    log.push(format!(
                        "would keep branch {} (not merged into main)",
                        candidate.branch
                    ));
                }
            } else if force {
                log.push(format!(
                    "warning: force-removing {} — branch may not be merged",
                    candidate.branch
                ));
                let remove_out = apm_core::clean::remove(&root, candidate, true, branches)?;
                if let Some(ref path) = candidate.worktree {
                    log.push(format!("removed worktree {}", path.display()));
                    count += 1;
                }
                if branches && candidate.local_branch_exists {
                    log.push(format!("removed branch {}", candidate.branch));
                }
                for w in &remove_out.warnings {
                    log.push(w.clone());
                }
            } else {
                let remove_out = apm_core::clean::remove(&root, candidate, false, branches)?;
                if let Some(ref path) = candidate.worktree {
                    log.push(format!("removed worktree {}", path.display()));
                    count += 1;
                }
                if branches && candidate.local_branch_exists && candidate.branch_merged {
                    log.push(format!("removed branch {}", candidate.branch));
                } else if branches && candidate.local_branch_exists && !candidate.branch_merged {
                    log.push(format!("kept branch {} (not merged into main)", candidate.branch));
                }
                for w in &remove_out.warnings {
                    log.push(w.clone());
                }
            }
        }

        if remote {
            let threshold_str = older_than.as_deref().unwrap();
            let threshold = apm_core::clean::parse_older_than(threshold_str)?;
            let remote_candidates = apm_core::clean::remote_candidates(&root, &config, threshold)?;
            if remote_candidates.is_empty() {
                log.push("No remote branches to clean.".to_string());
            }
            for rc in &remote_candidates {
                if dry_run {
                    log.push(format!(
                        "would delete remote branch {} (last commit: {})",
                        rc.branch,
                        rc.last_commit.format("%Y-%m-%d")
                    ));
                } else {
                    apm_core::git::delete_remote_branch(&root, &rc.branch)?;
                    log.push(format!("deleted remote branch {}", rc.branch));
                }
            }
        }

        if epics {
            let local_output = std::process::Command::new("git")
                .current_dir(&root)
                .args(["branch", "--list", "epic/*"])
                .output()?;
            let local_branches: Vec<String> = String::from_utf8_lossy(&local_output.stdout)
                .lines()
                .map(|l| l.trim().trim_start_matches(['*', '+']).trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();

            let tickets = apm_core::ticket::load_all_from_git(&root, &config.tickets.dir)?;

            let mut epic_candidates: Vec<String> = Vec::new();
            for branch in &local_branches {
                let after_prefix = branch.trim_start_matches("epic/");
                let id_end = after_prefix.find('-').unwrap_or(after_prefix.len()).min(8);
                let id = &after_prefix[..id_end];
                let epic_tickets: Vec<_> = tickets
                    .iter()
                    .filter(|t| t.frontmatter.epic.as_deref() == Some(id))
                    .collect();
                let state_configs: Vec<&apm_core::config::StateConfig> = epic_tickets
                    .iter()
                    .filter_map(|t| config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
                    .collect();
                if apm_core::epic::derive_epic_state(&state_configs) == "done" {
                    epic_candidates.push(branch.clone());
                }
            }

            if epic_candidates.is_empty() {
                log.push("No done epics to clean.".to_string());
            }

            for branch in &epic_candidates {
                let after_prefix = branch.trim_start_matches("epic/");
                let id_end = after_prefix.find('-').unwrap_or(after_prefix.len()).min(8);
                let id = after_prefix[..id_end].to_string();

                if dry_run {
                    log.push(format!("would delete epic branch {branch}"));
                    continue;
                }

                let del_local = std::process::Command::new("git")
                    .current_dir(&root)
                    .args(["branch", "-d", branch])
                    .output()?;
                if !del_local.status.success() {
                    log.push(format!(
                        "error: failed to delete local branch {branch}: {}",
                        String::from_utf8_lossy(&del_local.stderr).trim()
                    ));
                    continue;
                }

                let del_remote = std::process::Command::new("git")
                    .current_dir(&root)
                    .args(["push", "origin", "--delete", branch])
                    .output()?;
                if !del_remote.status.success() {
                    let stderr = String::from_utf8_lossy(&del_remote.stderr);
                    if !stderr.contains("remote ref does not exist")
                        && !stderr.contains("error: unable to delete")
                    {
                        log.push(format!("warning: failed to delete remote {branch}: {}", stderr.trim()));
                    }
                }

                log.push(format!("deleted epic {branch}"));

                let epics_path = root.join(".apm").join("epics.toml");
                if epics_path.exists() {
                    let raw = std::fs::read_to_string(&epics_path)?;
                    let mut table: toml::value::Table = toml::from_str(&raw)?;
                    if table.remove(&id).is_some() {
                        std::fs::write(&epics_path, toml::to_string(&table)?)?;
                    }
                }
            }
        }

        Ok((log, count))
    }).await?;

    Ok(Json(serde_json::json!({ "log": log.join("\n"), "removed": removed })).into_response())
}

async fn collaborators_handler(
    State(state): State<Arc<AppState>>,
) -> Response {
    let root = match state.git_root() {
        Some(r) => r,
        None => return Json(Vec::<String>::new()).into_response(),
    };
    let Ok(config) = apm_core::config::Config::load(root) else {
        return Json(Vec::<String>::new()).into_response();
    };
    let local = apm_core::config::LocalConfig::load(root);
    let (collaborators, _warnings) = apm_core::config::resolve_collaborators(&config, &local);
    Json(collaborators).into_response()
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

async fn require_auth(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let connect_info = req.extensions().get::<ConnectInfo<SocketAddr>>().copied();
    if connect_info.is_none() || is_localhost(connect_info) {
        return next.run(req).await;
    }
    if find_session_username(req.headers(), &state.session_store).is_some() {
        return next.run(req).await;
    }
    if req.uri().path().starts_with("/api/") {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "unauthorized"})),
        )
            .into_response()
    } else {
        axum::response::Redirect::temporary("/login").into_response()
    }
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

fn build_app(root: PathBuf, origin_override: Option<&str>) -> Router {
    let config = apm_core::config::Config::load(&root).expect("cannot load apm config");
    let tickets_dir = config.tickets.dir;
    let log_file = config.logging.file.map(|p| {
        if p.is_absolute() { p } else { root.join(&p) }
    });
    let sessions_path = root.join(".apm/sessions.json");
    let credentials_path = root.join(".apm/credentials.json");
    let origin = origin_override
        .map(String::from)
        .unwrap_or(config.server.origin);
    let wa_state = webauthn_state::WebauthnState::new(&origin)
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
    let protected = Router::new()
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
        .route("/api/epics", get(handlers::epics::list_epics).post(handlers::epics::create_epic))
        .route("/api/epics/:id", get(handlers::epics::get_epic))
        .route("/api/me", get(me_handler))
        .route("/api/collaborators", get(collaborators_handler))
        .route("/api/auth/otp", post(otp_handler))
        .route("/api/auth/sessions", get(list_sessions_handler).delete(revoke_sessions_handler))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), require_auth));
    let open = Router::new()
        .route("/health", get(health_handler))
        .route("/register", get(register_page_handler))
        .route("/api/auth/register/challenge", post(register_challenge_handler))
        .route("/api/auth/register/complete", post(register_complete_handler))
        .route("/login", get(login_page_handler))
        .route("/api/auth/login/challenge", post(login_challenge_handler))
        .route("/api/auth/login/complete", post(login_complete_handler));
    let fallback_state = state.clone();
    Router::new()
        .merge(protected)
        .merge(open)
        .fallback(move |req: axum::extract::Request| {
            let state = fallback_state.clone();
            async move {
                let connect_info = req.extensions().get::<ConnectInfo<SocketAddr>>().copied();
                if connect_info.is_none()
                    || is_localhost(connect_info)
                    || find_session_username(req.headers(), &state.session_store).is_some()
                {
                    serve_ui(req.uri().clone()).await
                } else {
                    axum::response::Redirect::temporary("/login").into_response()
                }
            }
        })
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
        .route("/api/epics", get(handlers::epics::list_epics).post(handlers::epics::create_epic))
        .route("/api/epics/:id", get(handlers::epics::get_epic))
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

/// Spawn a plain HTTP listener on 127.0.0.1:3000 for localhost CLI access
/// (e.g. `apm register`) when the main server is running TLS.
fn spawn_localhost_http(app: Router, _tls_port: u16) {
    tokio::spawn(async move {
        let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
        let listener = tokio::net::TcpListener::bind(addr).await.expect("bind localhost HTTP listener");
        println!("Listening on http://{addr} (localhost CLI access)");
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });
}

#[tokio::main]
async fn main() {
    use clap::Parser;
    let cli = Cli::parse();

    let root = std::env::current_dir().unwrap();
    let origin_override = cli.tls_domain.as_deref().map(|d| format!("https://{d}"));
    let app = build_app(root, origin_override.as_deref());

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
            spawn_localhost_http(app.clone(), port);
            tls::serve_tls(addr, add_hsts(app), config).await;
        }
        (Some(TlsMode::SelfSigned), _) => {
            let domain = cli.tls_domain.as_deref().unwrap_or("localhost");
            eprintln!("Warning: self-signed certificate for '{domain}' — not trusted by browsers");
            let config = tls::self_signed_config(domain)
                .expect("failed to generate self-signed certificate");
            spawn_localhost_http(app.clone(), port);
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
            spawn_localhost_http(app.clone(), port);
            tls::serve_acme(addr, add_hsts(app), domain, email, cache_dir).await;
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::handlers::tickets::{extract_frontmatter_raw, extract_history_raw};
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

    pub(crate) fn test_tickets() -> Vec<Ticket> {
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
        assert!(json["tickets"].is_array());
        assert_eq!(json["tickets"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn list_tickets_envelope_has_supervisor_states() {
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
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let supervisor_states = json["supervisor_states"].as_array().unwrap();
        assert!(supervisor_states.iter().any(|s| s == "new"), "new must be in supervisor_states");
        assert!(!supervisor_states.iter().any(|s| s == "closed"), "closed must not be in supervisor_states");
        assert!(!supervisor_states.iter().any(|s| s == "ammend"), "ammend must not be in supervisor_states (agent-only)");
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
        let arr = json["tickets"].as_array().unwrap();
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
        assert_eq!(json["tickets"].as_array().unwrap().len(), 2);
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
        let arr = json["tickets"].as_array().unwrap();

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

        let app = build_app(p.clone(), None);
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
        let arr = json["tickets"].as_array().unwrap();

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

    pub(crate) fn git_setup(p: &std::path::Path) {
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
        let mut _warnings = Vec::new();
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
            &mut _warnings,
        )
        .unwrap();
        let ticket_id = ticket.frontmatter.id.clone();

        let app = build_app(p.clone(), None);
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
    async fn patch_ticket_owner_persists_to_git() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);

        let config = apm_core::config::Config::load(&p).unwrap();
        let mut _warnings = Vec::new();
        let ticket = apm_core::ticket::create(
            &p, &config, "test ticket".to_string(), "test".to_string(),
            None, None, false, vec![], None, None, None, None, &mut _warnings,
        ).unwrap();
        let ticket_id = ticket.frontmatter.id.clone();

        let app = build_app(p.clone(), None);
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/tickets/{}", &ticket_id[..8]))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"owner":"alice"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["owner"], "alice");

        let branch = ticket.frontmatter.branch.unwrap();
        let rel_path = ticket.path.strip_prefix(&p).unwrap().to_string_lossy().to_string();
        let file_content = apm_core::git::read_from_branch(&p, &branch, &rel_path).unwrap();
        assert!(file_content.contains(r#"owner = "alice""#), "expected owner in frontmatter: {file_content}");
    }

    #[tokio::test]
    async fn patch_ticket_owner_empty_clears() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);

        let config = apm_core::config::Config::load(&p).unwrap();
        let mut _warnings = Vec::new();
        let ticket = apm_core::ticket::create(
            &p, &config, "test ticket".to_string(), "test".to_string(),
            None, None, false, vec![], None, None, None, None, &mut _warnings,
        ).unwrap();
        let ticket_id = ticket.frontmatter.id.clone();

        let app1 = build_app(p.clone(), None);
        app1.oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/tickets/{}", &ticket_id[..8]))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"owner":"alice"}"#))
                .unwrap(),
        ).await.unwrap();

        let app2 = build_app(p.clone(), None);
        let response = app2
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/tickets/{}", &ticket_id[..8]))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"owner":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["owner"].is_null(), "owner should be absent after clear");

        let branch = ticket.frontmatter.branch.unwrap();
        let rel_path = ticket.path.strip_prefix(&p).unwrap().to_string_lossy().to_string();
        let file_content = apm_core::git::read_from_branch(&p, &branch, &rel_path).unwrap();
        assert!(!file_content.contains("owner ="), "owner should be absent in frontmatter: {file_content}");
    }

    #[tokio::test]
    async fn patch_ticket_owner_omitted_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_path_buf();
        git_setup(&p);

        let config = apm_core::config::Config::load(&p).unwrap();
        let mut _warnings = Vec::new();
        let ticket = apm_core::ticket::create(
            &p, &config, "test ticket".to_string(), "test".to_string(),
            None, None, false, vec![], None, None, None, None, &mut _warnings,
        ).unwrap();
        let ticket_id = ticket.frontmatter.id.clone();

        let app1 = build_app(p.clone(), None);
        app1.oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/tickets/{}", &ticket_id[..8]))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"owner":"alice"}"#))
                .unwrap(),
        ).await.unwrap();

        let app2 = build_app(p.clone(), None);
        let response = app2
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/tickets/{}", &ticket_id[..8]))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"priority":5}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["owner"], "alice", "owner should be unchanged when field omitted");
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

        let app = build_app(p.clone(), None);
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

        let app = build_app(p.clone(), None);
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

        let app = build_app(p.clone(), None);
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
        let arr = json["tickets"].as_array().unwrap();
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
        let arr = json["tickets"].as_array().unwrap();
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
        let arr = json["tickets"].as_array().unwrap();
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
        let arr = json["tickets"].as_array().unwrap();
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
        let arr = json["tickets"].as_array().unwrap();
        assert!(arr[0]["owner"].is_null());
    }

    #[tokio::test]
    async fn get_ticket_owner_field_absent() {
        let ticket = fake_ticket("cccc1111-owner-absent-detail", "Owner absent detail");
        let app = build_app_with_tickets(vec![ticket]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets/cccc1111")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["owner"].is_null());
    }

    #[tokio::test]
    async fn get_ticket_owner_field_present() {
        let mut ticket = fake_ticket("dddd2222-owner-present-detail", "Owner present detail");
        ticket.frontmatter.owner = Some("alice".to_string());
        let app = build_app_with_tickets(vec![ticket]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/tickets/dddd2222")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["owner"], "alice");
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
        let arr = json["tickets"].as_array().unwrap();
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
        let arr = json["tickets"].as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], "ffff0000-owner-none");
    }

    fn build_app_for_middleware_tests(session_store: auth::SessionStore) -> Router {
        let state = Arc::new(AppState {
            source: TicketSource::InMemory(vec![]),
            work_engine: work::new_engine_state(),
            log_file: None,
            max_concurrent_override: Arc::new(tokio::sync::Mutex::new(None)),
            otp_store: auth::OtpStore::new(),
            session_store,
            webauthn_state: default_webauthn_state(),
            credential_store: credential_store::CredentialStore::load(PathBuf::new()),
        });
        let protected = Router::new()
            .route("/api/tickets", get(list_tickets).post(create_ticket))
            .route("/api/auth/otp", post(otp_handler))
            .route("/api/auth/sessions", get(list_sessions_handler).delete(revoke_sessions_handler))
            .route_layer(axum::middleware::from_fn_with_state(state.clone(), require_auth));
        let open = Router::new()
            .route("/health", get(health_handler))
            .route("/api/auth/register/challenge", post(register_challenge_handler))
            .route("/api/auth/login/challenge", post(login_challenge_handler));
        Router::new()
            .merge(protected)
            .merge(open)
            .with_state(state)
    }

    #[tokio::test]
    async fn require_auth_external_no_session_returns_401() {
        let app = build_app_for_middleware_tests(auth::SessionStore::load(PathBuf::new()));
        let mut req = Request::builder().uri("/api/tickets").body(Body::empty()).unwrap();
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from(([203, 0, 113, 1], 8080))));
        let response = app
            .oneshot(req)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "unauthorized");
    }

    #[tokio::test]
    async fn require_auth_external_valid_session_returns_200() {
        let session_store = auth::SessionStore::load(PathBuf::new());
        session_store.insert("validtoken".to_string(), "alice".to_string(), None);
        let app = build_app_for_middleware_tests(session_store);
        let mut req = Request::builder()
            .uri("/api/tickets")
            .header("cookie", "__Host-apm-session=validtoken")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from(([203, 0, 113, 1], 8080))));
        let response = app
            .oneshot(req)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn require_auth_external_invalid_session_returns_401() {
        let app = build_app_for_middleware_tests(auth::SessionStore::load(PathBuf::new()));
        let mut req = Request::builder()
            .uri("/api/tickets")
            .header("cookie", "__Host-apm-session=bogustoken")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from(([203, 0, 113, 1], 8080))));
        let response = app
            .oneshot(req)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn require_auth_external_expired_session_returns_401() {
        let session_store = auth::SessionStore::load(PathBuf::new());
        let expired = chrono::Utc::now() - chrono::Duration::days(10);
        session_store.insert_expiring_at("exptoken".to_string(), "eve".to_string(), expired);
        let app = build_app_for_middleware_tests(session_store);
        let mut req = Request::builder()
            .uri("/api/tickets")
            .header("cookie", "__Host-apm-session=exptoken")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from(([203, 0, 113, 1], 8080))));
        let response = app
            .oneshot(req)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn require_auth_loopback_no_session_returns_200() {
        let app = build_app_for_middleware_tests(auth::SessionStore::load(PathBuf::new()));
        let mut req = Request::builder().uri("/api/tickets").body(Body::empty()).unwrap();
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 9999))));
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn require_auth_health_no_session_returns_200() {
        let app = build_app_for_middleware_tests(auth::SessionStore::load(PathBuf::new()));
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn require_auth_login_challenge_no_session_not_401() {
        let app = build_app_for_middleware_tests(auth::SessionStore::load(PathBuf::new()));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(br#"{"username":"alice"}"#.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn require_auth_register_challenge_no_session_not_401() {
        let app = build_app_for_middleware_tests(auth::SessionStore::load(PathBuf::new()));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/register/challenge")
                    .header("content-type", "application/json")
                    .body(Body::from(br#"{"username":"alice","otp":"ABCD1234"}"#.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn require_auth_otp_external_no_session_returns_401() {
        let app = build_app_for_middleware_tests(auth::SessionStore::load(PathBuf::new()));
        let mut req = Request::builder()
            .method("POST")
            .uri("/api/auth/otp")
            .header("content-type", "application/json")
            .body(Body::from(br#"{"username":"alice"}"#.to_vec()))
            .unwrap();
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from(([203, 0, 113, 1], 8080))));
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn require_auth_sessions_external_no_session_returns_401() {
        let app = build_app_for_middleware_tests(auth::SessionStore::load(PathBuf::new()));
        let mut req = Request::builder().uri("/api/auth/sessions").body(Body::empty()).unwrap();
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from(([203, 0, 113, 1], 8080))));
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
