use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use std::sync::Arc;

use crate::{AppError, AppState};

#[derive(serde::Serialize)]
pub struct AgentsConfigResponse {
    max_concurrent: usize,
    #[serde(rename = "override")]
    override_val: Option<usize>,
}

#[derive(serde::Deserialize)]
pub struct PatchAgentsConfigRequest {
    #[serde(rename = "override")]
    override_val: usize,
}

pub async fn get_agents_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AgentsConfigResponse>, AppError> {
    let max_concurrent = match state.git_root() {
        Some(root) => {
            let config = crate::util::load_config(root.clone()).await?;
            config.agents.max_concurrent.max(1)
        }
        None => 3,
    };
    let override_val = *state.max_concurrent_override.lock().await;
    Ok(Json(AgentsConfigResponse { max_concurrent, override_val }))
}

pub async fn patch_agents_config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PatchAgentsConfigRequest>,
) -> Result<Response, AppError> {
    if req.override_val < 1 {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, "override must be >= 1").into_response());
    }
    *state.max_concurrent_override.lock().await = Some(req.override_val);
    let max_concurrent = match state.git_root() {
        Some(root) => {
            let config = crate::util::load_config(root.clone()).await?;
            config.agents.max_concurrent.max(1)
        }
        None => 3,
    };
    Ok(Json(AgentsConfigResponse {
        max_concurrent,
        override_val: Some(req.override_val),
    }).into_response())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn get_agents_config_returns_default_when_in_memory() {
        let app = crate::build_app_in_memory_for_work();
        let response = app
            .oneshot(Request::builder().uri("/api/agents/config").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["max_concurrent"], 3);
        assert!(json["override"].is_null());
    }

    #[tokio::test]
    async fn patch_agents_config_stores_override() {
        let app = crate::build_app_in_memory_for_work();
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/agents/config")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"override":5}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["override"], 5);
        assert_eq!(json["max_concurrent"], 3);
    }

    #[tokio::test]
    async fn patch_agents_config_with_zero_returns_422() {
        let app = crate::build_app_in_memory_for_work();
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/agents/config")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"override":0}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
