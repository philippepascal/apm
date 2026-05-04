use std::sync::Arc;
use axum::{extract::State, Json};
use serde::Serialize;
use crate::AppState;

#[derive(Serialize)]
pub struct StateNode {
    pub id: String,
    pub label: String,
    pub terminal: bool,
    pub actionable: Vec<String>,
}

#[derive(Serialize)]
pub struct TransitionEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub trigger: String,
}

#[derive(Serialize)]
pub struct WorkflowGraphResponse {
    pub states: Vec<StateNode>,
    pub transitions: Vec<TransitionEdge>,
}

pub async fn workflow_handler(State(state): State<Arc<AppState>>) -> Json<WorkflowGraphResponse> {
    let Some(root) = state.git_root() else {
        return Json(WorkflowGraphResponse { states: vec![], transitions: vec![] });
    };
    let Ok(cfg) = apm_core::config::Config::load(root) else {
        return Json(WorkflowGraphResponse { states: vec![], transitions: vec![] });
    };
    let states = cfg.workflow.states.iter().map(|s| StateNode {
        id: s.id.clone(),
        label: s.label.clone(),
        terminal: s.terminal,
        actionable: s.actionable.clone(),
    }).collect();
    let transitions = cfg.workflow.states.iter().flat_map(|s| {
        s.transitions.iter().map(move |tr| TransitionEdge {
            from: s.id.clone(),
            to: tr.to.clone(),
            label: if tr.label.is_empty() {
                format!("→ {}", tr.to)
            } else {
                tr.label.clone()
            },
            trigger: tr.trigger.clone(),
        })
    }).collect();
    Json(WorkflowGraphResponse { states, transitions })
}
