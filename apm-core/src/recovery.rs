use crate::config::{CompletionStrategy, WorkflowConfig};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryKind {
    RetryMerge,
    ReturnToWorker,
    Abandon,
    Other,
}

#[derive(Debug, Clone)]
pub struct RecoveryOption {
    pub to: String,
    pub label: String,
    pub kind: RecoveryKind,
}

/// Returns true iff `state_id` is the `on_failure` target of at least one
/// merging-completion transition (Pr, Merge, or PrOrEpicMerge) anywhere in the
/// workflow.  Transitions with a missing or empty `on_failure` are skipped.
pub fn is_merge_failure_state(state_id: &str, workflow: &WorkflowConfig) -> bool {
    for state in &workflow.states {
        for t in &state.transitions {
            if !matches!(
                t.completion,
                CompletionStrategy::Pr | CompletionStrategy::Merge | CompletionStrategy::PrOrEpicMerge
            ) {
                continue;
            }
            if let Some(on_failure) = &t.on_failure {
                if !on_failure.is_empty() && on_failure == state_id {
                    return true;
                }
            }
        }
    }
    false
}

/// Classify the outgoing transitions of `state_id` as recovery options.
///
/// Each transition is labelled by its kind:
/// - `RetryMerge`: the to-state is the target of at least one merging-completion
///   transition anywhere in the workflow (Pr, Merge, or PrOrEpicMerge).
/// - `ReturnToWorker`: the to-state is the target of at least one non-spec-writer
///   `command:start` transition anywhere in the workflow.
/// - `Abandon`: the to-state has `terminal: true`.
/// - `Other`: none of the above apply.
///
/// Results are in declaration order.  Returns an empty vec if `state_id` is not
/// found in the workflow.
pub fn classify_recovery_options(state_id: &str, workflow: &WorkflowConfig) -> Vec<RecoveryOption> {
    let merge_target_ids: HashSet<String> = workflow.states.iter()
        .flat_map(|s| s.transitions.iter())
        .filter(|t| matches!(
            t.completion,
            CompletionStrategy::Pr | CompletionStrategy::Merge | CompletionStrategy::PrOrEpicMerge
        ))
        .map(|t| t.to.clone())
        .collect();

    let coder_start_ids: HashSet<String> = workflow.states.iter()
        .flat_map(|s| s.transitions.iter())
        .filter(|t| {
            t.trigger == "command:start"
                && t.worker_profile
                    .as_deref()
                    .is_none_or(|p| !p.ends_with("/spec-writer"))
        })
        .map(|t| t.to.clone())
        .collect();

    let terminal_ids: HashSet<&str> = workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();

    let Some(state) = workflow.states.iter().find(|s| s.id == state_id) else {
        return Vec::new();
    };

    state.transitions.iter().map(|t| {
        let kind = if merge_target_ids.contains(&t.to) {
            RecoveryKind::RetryMerge
        } else if coder_start_ids.contains(&t.to) {
            RecoveryKind::ReturnToWorker
        } else if terminal_ids.contains(t.to.as_str()) {
            RecoveryKind::Abandon
        } else {
            RecoveryKind::Other
        };
        let label = if t.label.is_empty() { t.to.clone() } else { t.label.clone() };
        RecoveryOption { to: t.to.clone(), label, kind }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_workflow(toml: &str) -> WorkflowConfig {
        #[derive(serde::Deserialize)]
        struct W { states: Vec<crate::config::StateConfig> }
        let w: W = toml::from_str(toml).unwrap();
        WorkflowConfig { states: w.states, ..Default::default() }
    }

    const DEFAULT_WF: &str = r#"[[states]]
id    = "ready"
label = "Ready"

  [[states.transitions]]
  to             = "in_progress"
  trigger        = "command:start"
  worker_profile = "claude/coder"

[[states]]
id    = "in_progress"
label = "In Progress"

  [[states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  completion = "pr_or_epic_merge"
  on_failure = "merge_failed"

[[states]]
id    = "implemented"
label = "Implemented"

[[states]]
id    = "merge_failed"
label = "Merge failed"

  [[states.transitions]]
  to      = "implemented"
  trigger = "manual"

  [[states.transitions]]
  to      = "in_progress"
  trigger = "manual"

[[states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;

    #[test]
    fn test_default_workflow_merge_failed() {
        let wf = parse_workflow(DEFAULT_WF);
        let opts = classify_recovery_options("merge_failed", &wf);
        assert_eq!(opts.len(), 2);
        assert_eq!(opts[0].to, "implemented");
        assert_eq!(opts[0].kind, RecoveryKind::RetryMerge);
        assert_eq!(opts[1].to, "in_progress");
        assert_eq!(opts[1].kind, RecoveryKind::ReturnToWorker);
    }

    #[test]
    fn test_shuffled_order_same_classification() {
        let shuffled = r#"[[states]]
id       = "closed"
label    = "Closed"
terminal = true

[[states]]
id         = "merge_failed"
label      = "Merge failed"

  [[states.transitions]]
  to      = "implemented"
  trigger = "manual"

  [[states.transitions]]
  to      = "in_progress"
  trigger = "manual"

[[states]]
id    = "implemented"
label = "Implemented"

[[states]]
id    = "in_progress"
label = "In Progress"

  [[states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  completion = "pr_or_epic_merge"
  on_failure = "merge_failed"

[[states]]
id    = "ready"
label = "Ready"

  [[states.transitions]]
  to             = "in_progress"
  trigger        = "command:start"
  worker_profile = "claude/coder"
"#;
        let wf = parse_workflow(shuffled);
        let opts = classify_recovery_options("merge_failed", &wf);
        assert_eq!(opts.len(), 2);
        assert_eq!(opts[0].to, "implemented");
        assert_eq!(opts[0].kind, RecoveryKind::RetryMerge);
        assert_eq!(opts[1].to, "in_progress");
        assert_eq!(opts[1].kind, RecoveryKind::ReturnToWorker);
    }

    #[test]
    fn test_renamed_merge_target() {
        let renamed = r#"[[states]]
id    = "ready"
label = "Ready"

  [[states.transitions]]
  to             = "in_progress"
  trigger        = "command:start"
  worker_profile = "claude/coder"

[[states]]
id    = "in_progress"
label = "In Progress"

  [[states.transitions]]
  to         = "shipped"
  trigger    = "manual"
  completion = "pr_or_epic_merge"
  on_failure = "merge_failed"

[[states]]
id    = "shipped"
label = "Shipped"

[[states]]
id         = "merge_failed"
label      = "Merge failed"

  [[states.transitions]]
  to      = "shipped"
  trigger = "manual"

  [[states.transitions]]
  to      = "in_progress"
  trigger = "manual"
"#;
        let wf = parse_workflow(renamed);
        let opts = classify_recovery_options("merge_failed", &wf);
        assert_eq!(opts.len(), 2);
        assert_eq!(opts[0].to, "shipped");
        assert_eq!(opts[0].kind, RecoveryKind::RetryMerge);
        assert_eq!(opts[1].to, "in_progress");
        assert_eq!(opts[1].kind, RecoveryKind::ReturnToWorker);
    }

    #[test]
    fn test_no_merge_transitions() {
        let no_merge = r#"[[states]]
id    = "some_state"
label = "Some State"

  [[states.transitions]]
  to      = "other"
  trigger = "manual"

[[states]]
id    = "other"
label = "Other"
"#;
        let wf = parse_workflow(no_merge);
        let opts = classify_recovery_options("some_state", &wf);
        assert!(!opts.iter().any(|o| o.kind == RecoveryKind::RetryMerge));
    }

    #[test]
    fn test_is_merge_failure_state_default_workflow() {
        let wf = parse_workflow(DEFAULT_WF);
        assert!(is_merge_failure_state("merge_failed", &wf));
        for state in &["new", "groomed", "specd", "ready", "in_progress", "implemented", "closed"] {
            assert!(
                !is_merge_failure_state(state, &wf),
                "expected false for state: {state}"
            );
        }
    }

    #[test]
    fn test_is_merge_failure_state_renamed() {
        let renamed = r#"[[states]]
id    = "in_progress"
label = "In Progress"

  [[states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  completion = "merge"
  on_failure = "pr_failed"

[[states]]
id    = "implemented"
label = "Implemented"

[[states]]
id    = "pr_failed"
label = "Pr Failed"
"#;
        let wf = parse_workflow(renamed);
        assert!(is_merge_failure_state("pr_failed", &wf));
        assert!(!is_merge_failure_state("merge_failed", &wf));
    }
}
