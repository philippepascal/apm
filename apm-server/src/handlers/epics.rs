use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::{AppError, AppState};
use crate::handlers::tickets::{extract_section, load_tickets};
use crate::models::{CreateEpicRequest, EpicDetailResponse, EpicSummary, TicketResponse};

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
    let mut ticket_counts: HashMap<String, usize> = HashMap::new();
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

pub async fn list_epics(
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let tickets = load_tickets(&state).await?;
    let config = crate::util::load_config(root.clone()).await?;
    let branches = crate::util::blocking(move || apm_core::epic::epic_branches(&root)).await?;
    let summaries: Vec<EpicSummary> = branches
        .iter()
        .filter_map(|b| build_epic_summary(b, &tickets, &config.workflow.states))
        .collect();
    Ok(Json(summaries).into_response())
}

pub async fn create_epic(
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
    let (id, branch) = crate::util::blocking(move || {
        let config = apm_core::config::Config::load(&root)?;
        apm_core::epic::create_epic_branch(&root, &title_clone, &config)
    }).await?;
    Ok((
        StatusCode::CREATED,
        Json(EpicSummary {
            id,
            title,
            branch,
            state: "empty".to_string(),
            ticket_counts: HashMap::new(),
        }),
    )
        .into_response())
}

pub async fn get_epic(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let tickets = load_tickets(&state).await?;
    let config = crate::util::load_config(root.clone()).await?;
    let branches = crate::util::blocking(move || apm_core::epic::epic_branches(&root)).await?;
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
            let mut fm = t.frontmatter;
            let owner = fm.owner.take();
            TicketResponse {
                frontmatter: fm,
                body: t.body,
                has_open_questions,
                has_pending_amendments,
                blocking_deps: vec![],
                owner,
            }
        })
        .collect();
    Ok(Json(EpicDetailResponse {
        summary,
        tickets: epic_tickets,
    })
    .into_response())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn list_epics_in_memory_returns_501() {
        let app = crate::build_app_with_tickets(crate::tests::test_tickets());
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
        crate::tests::git_setup(&p);
        let app = crate::build_app(p.clone(), None);
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
        crate::tests::git_setup(&p);
        let app = crate::build_app(p.clone(), None);
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
        let app = crate::build_app_with_tickets(crate::tests::test_tickets());
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
        let app = crate::build_app_with_tickets(crate::tests::test_tickets());
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
        crate::tests::git_setup(&p);
        let app = crate::build_app(p.clone(), None);
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
        crate::tests::git_setup(&p);
        let app = crate::build_app(p.clone(), None);
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
        crate::tests::git_setup(&p);

        let app = crate::build_app(p.clone(), None);
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
        let app2 = crate::build_app(p.clone(), None);
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
        let app3 = crate::build_app(p.clone(), None);
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
}
