use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::{AppError, AppState, TicketSource};
use crate::models::*;

pub(crate) fn extract_section<'a>(body: &'a str, heading: &str) -> &'a str {
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

pub(crate) fn extract_frontmatter_raw(content: &str) -> Option<&str> {
    let rest = content.strip_prefix("+++\n")?;
    let end = rest.find("\n+++")?;
    Some(&rest[..end])
}

pub(crate) fn extract_history_raw(content: &str) -> &str {
    match content.find("\n## History") {
        Some(idx) => &content[idx..],
        None => "",
    }
}

pub async fn load_tickets(state: &AppState) -> Result<Vec<apm_core::ticket::Ticket>, AppError> {
    match &state.source {
        TicketSource::Git(root, tickets_dir) => {
            let root = root.clone();
            let tickets_dir = tickets_dir.clone();
            Ok(crate::util::blocking(move || {
                apm_core::ticket::load_all_from_git(&root, &tickets_dir)
            }).await?)
        }
        TicketSource::InMemory(tickets) => Ok(tickets.clone()),
    }
}

pub async fn list_tickets(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListTicketsQuery>,
) -> Result<Json<TicketsEnvelope>, AppError> {
    let mut tickets = load_tickets(&state).await?;
    let fallback_supervisor_states = || vec![
        "new".to_string(), "question".to_string(), "specd".to_string(),
        "blocked".to_string(), "implemented".to_string(),
    ];
    let (resolved_ids, terminal_ids, supervisor_states): (Vec<String>, Vec<String>, Vec<String>) = match state.git_root() {
        Some(root) => match apm_core::config::Config::load(root) {
            Ok(cfg) => {
                let resolved = cfg.workflow.states.iter()
                    .filter(|s| matches!(s.satisfies_deps, apm_core::config::SatisfiesDeps::Bool(true)) || s.terminal)
                    .map(|s| s.id.clone())
                    .collect();
                let mut supervisor = vec!["new".to_string()];
                supervisor.extend(
                    cfg.workflow.states.iter()
                        .filter(|s| !s.terminal && s.id != "new" && s.actionable.iter().any(|a| a == "supervisor"))
                        .map(|s| s.id.clone())
                );
                let terminal = cfg.workflow.states.into_iter()
                    .filter(|s| s.terminal)
                    .map(|s| s.id)
                    .collect();
                (resolved, terminal, supervisor)
            }
            Err(_) => (vec![], vec!["closed".to_string()], fallback_supervisor_states()),
        },
        None => (vec![], vec!["closed".to_string()], fallback_supervisor_states()),
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
    let tickets: Vec<TicketResponse> = tickets
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
            let owner = fm.owner.take();
            TicketResponse {
                frontmatter: fm,
                body: t.body,
                has_open_questions,
                has_pending_amendments,
                blocking_deps,
                owner,
            }
        })
        .collect();
    Ok(Json(TicketsEnvelope { tickets, supervisor_states }))
}

pub async fn get_ticket(
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
                    let deps = match apm_core::config::Config::load(&root) {
                        Ok(config) => apm_core::compute_blocking_deps(ticket_ref, &tickets, &config),
                        Err(_) => vec![],
                    };
                    let state_str = ticket_ref.frontmatter.state.clone();
                    let transitions = tokio::task::spawn_blocking(move || {
                        let config = match apm_core::config::Config::load(&root) {
                            Ok(c) => c,
                            Err(_) => return vec![],
                        };
                        apm_core::compute_valid_transitions(&state_str, &config)
                    }).await?;
                    (deps, transitions)
                }
            };
            let mut ticket = tickets.into_iter().find(|t| t.frontmatter.id == full_id).unwrap();
            let raw = ticket.serialize().unwrap_or_default();
            if ticket.frontmatter.author.is_none() {
                ticket.frontmatter.author = Some("unassigned".to_string());
            }
            let owner = ticket.frontmatter.owner.take();
            Ok(Json(TicketDetailResponse {
                frontmatter: ticket.frontmatter,
                body: ticket.body,
                raw,
                valid_transitions,
                blocking_deps,
                owner,
            })
            .into_response())
        }
    }
}

pub async fn transition_ticket(
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
                    let blocking_deps = match apm_core::config::Config::load(&root) {
                        Ok(config) => apm_core::compute_blocking_deps(
                            tickets.iter().find(|t| t.frontmatter.id == full_id).unwrap(),
                            &tickets,
                            &config,
                        ),
                        Err(_) => vec![],
                    };
                    let ticket =
                        tickets.into_iter().find(|t| t.frontmatter.id == full_id).unwrap();
                    let state_str = ticket.frontmatter.state.clone();
                    let root2 = root.clone();
                    let valid_transitions = tokio::task::spawn_blocking(move || {
                        let config = match apm_core::config::Config::load(&root2) {
                            Ok(c) => c,
                            Err(_) => return vec![],
                        };
                        apm_core::compute_valid_transitions(&state_str, &config)
                    })
                    .await?;
                    let raw = ticket.serialize().unwrap_or_default();
                    let owner = ticket.frontmatter.owner.clone();
                    let mut fm = ticket.frontmatter;
                    fm.owner = None;
                    Ok(Json(TicketDetailResponse {
                        frontmatter: fm,
                        body: ticket.body,
                        raw,
                        valid_transitions,
                        blocking_deps,
                        owner,
                    })
                    .into_response())
                }
            }
        }
    }
}

pub async fn put_body(
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
    let current_content = crate::util::blocking(move || {
        apm_core::git::read_from_branch(&root_clone, &branch_clone, &rel_path_clone)
    }).await?;

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
    crate::util::blocking(move || {
        apm_core::git::commit_to_branch(&root, &branch, &rel_path, &content, "ui: edit ticket body")
    }).await?;

    Ok(Json(serde_json::json!({"ok": true})).into_response())
}

pub async fn patch_ticket(
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
    let blocking_deps = match apm_core::config::Config::load(&root) {
        Ok(config) => apm_core::compute_blocking_deps(
            tickets.iter().find(|t| t.frontmatter.id == full_id).unwrap(),
            &tickets,
            &config,
        ),
        Err(_) => vec![],
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
    if let Some(v) = req.owner {
        let val = if v.is_empty() { "-".to_string() } else { v };
        apm_core::ticket::set_field(&mut fm, "owner", &val)?;
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
    crate::util::blocking(move || {
        apm_core::git::commit_to_branch(
            &root_clone,
            &branch,
            &rel_path,
            &content,
            "ui: update ticket fields",
        )
    }).await?;

    let state_str = updated.frontmatter.state.clone();
    let valid_transitions = tokio::task::spawn_blocking(move || {
        let config = match apm_core::config::Config::load(&root) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        apm_core::compute_valid_transitions(&state_str, &config)
    })
    .await?;
    let raw = updated.serialize().unwrap_or_default();
    let owner = updated.frontmatter.owner.clone();
    let mut fm = updated.frontmatter;
    fm.owner = None;
    Ok(Json(TicketDetailResponse {
        frontmatter: fm,
        body: updated.body,
        raw,
        valid_transitions,
        blocking_deps,
        owner,
    })
    .into_response())
}

pub async fn batch_transition(
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

pub async fn batch_priority(
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

pub async fn create_ticket(
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
            match apm_core::epic::find_epic_branch(&root, short_id) {
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
        let mut _warnings = Vec::new();
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
            &mut _warnings,
        )
    })
    .await?;
    match result {
        Ok(ticket) => {
            let has_open_questions = !extract_section(&ticket.body, "Open questions").trim().is_empty();
            let has_pending_amendments = extract_section(&ticket.body, "Amendment requests").contains("- [ ]");
            let mut fm = ticket.frontmatter;
            let owner = fm.owner.take();
            let response = TicketResponse {
                frontmatter: fm,
                body: ticket.body,
                has_open_questions,
                has_pending_amendments,
                blocking_deps: vec![],
                owner,
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
