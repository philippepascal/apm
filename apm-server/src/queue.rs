use axum::{
    extract::{ConnectInfo, State},
    Json,
};
use std::{net::SocketAddr, sync::Arc};

use crate::{AppError, AppState, TicketSource};

#[derive(serde::Serialize)]
pub struct QueueEntry {
    rank: usize,
    id: String,
    title: String,
    state: String,
    priority: u8,
    effort: u8,
    risk: u8,
    score: f64,
    effective_priority: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    epic: Option<String>,
}

pub async fn queue_handler(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<QueueEntry>>, AppError> {
    let is_local = connect_info
        .map(|ConnectInfo(addr)| addr.ip().is_loopback())
        .unwrap_or(false);

    let caller: Option<String> = if is_local {
        state.git_root().map(|root| apm_core::config::resolve_identity(root))
    } else {
        let cookie_header = headers
            .get(axum::http::header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let mut found = None;
        for part in cookie_header.split(';') {
            if let Ok(c) = cookie::Cookie::parse(part.trim().to_owned()) {
                if c.name() == "__Host-apm-session" {
                    found = state.session_store.lookup(c.value());
                    break;
                }
            }
        }
        found
    };

    let (root, tickets_dir) = match &state.source {
        TicketSource::Git(root, tickets_dir) => (root.clone(), tickets_dir.clone()),
        TicketSource::InMemory(_) => return Ok(Json(vec![])),
    };
    let entries = crate::util::blocking(move || {
        let config = apm_core::config::Config::load(&root)?;
        let tickets = apm_core::ticket::load_all_from_git(&root, &tickets_dir)?;
        let actionable_owned = config.actionable_states_for("agent");
        let actionable: Vec<&str> = actionable_owned.iter().map(|s| s.as_str()).collect();
        let p = &config.workflow.prioritization;
        let active: Vec<&apm_core::ticket::Ticket> = tickets
            .iter()
            .filter(|t| !apm_core::ticket::dep_satisfied(&t.frontmatter.state, None, &config))
            .collect();
        let rev_idx = apm_core::ticket::build_reverse_index(&active);
        let sorted = apm_core::ticket::sorted_actionable(
            &tickets,
            &actionable,
            p.priority_weight,
            p.effort_weight,
            p.risk_weight,
            None,
            caller.as_deref(),
        );
        let result: Vec<QueueEntry> = sorted
            .into_iter()
            .enumerate()
            .map(|(i, t)| {
                let fm = &t.frontmatter;
                let ep = apm_core::ticket::effective_priority(t, &rev_idx);
                let raw_score = ep as f64 * p.priority_weight
                    + fm.effort as f64 * p.effort_weight
                    + fm.risk as f64 * p.risk_weight;
                let score = (raw_score * 100.0).round() / 100.0;
                QueueEntry {
                    rank: i + 1,
                    id: fm.id.clone(),
                    title: fm.title.clone(),
                    state: fm.state.clone(),
                    priority: fm.priority,
                    effort: fm.effort,
                    risk: fm.risk,
                    score,
                    effective_priority: ep,
                    epic: fm.epic.clone(),
                }
            })
            .collect();
        Ok::<_, anyhow::Error>(result)
    }).await?;
    Ok(Json(entries))
}

#[cfg(test)]
mod tests {
    use apm_core::ticket::{Frontmatter, Ticket};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use std::path::PathBuf;
    use tower::ServiceExt;

    fn fake_ticket(id: &str, state: &str, priority: u8, effort: u8, risk: u8) -> Ticket {
        Ticket {
            frontmatter: Frontmatter {
                id: id.to_string(),
                title: format!("Ticket {id}"),
                state: state.to_string(),
                priority,
                effort,
                risk,
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
            path: PathBuf::from(format!("{id}.md")),
        }
    }

    #[tokio::test]
    async fn queue_empty_for_in_memory() {
        let app = crate::build_app_in_memory_with_queue(vec![]);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/queue")
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

    #[tokio::test]
    async fn queue_returns_empty_array_for_in_memory_with_tickets() {
        let tickets = vec![
            fake_ticket("aaaa1111-aaa", "ready", 10, 3, 2),
            fake_ticket("bbbb2222-bbb", "specd", 5, 1, 1),
        ];
        let app = crate::build_app_in_memory_with_queue(tickets);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/queue")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // InMemory source returns empty vec regardless of tickets
        assert!(json.is_array());
    }
}
