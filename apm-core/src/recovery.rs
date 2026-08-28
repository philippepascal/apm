use crate::config::{CompletionStrategy, WorkflowConfig};
use anyhow::{anyhow, bail, Result};
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
        .flat_map(|s| s.transitions.iter().map(move |t| (s, t)))
        .filter(|(_, t)| t.trigger == "command:start")
        .filter(|(_, t)| {
            let dest_is_spec_writer = workflow.states.iter()
                .find(|s| s.id == t.to)
                .and_then(|s| s.worker_profile.as_deref())
                .map(|wp| wp.ends_with("/spec-writer"))
                .unwrap_or(false);
            !dest_is_spec_writer
        })
        .map(|(_, t)| t.to.clone())
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

/// Resolve the state a crashed worker's ticket should roll back to.
///
/// Resolution order:
/// 1. `explicit_to`, if given — must name a real, non-terminal state.
/// 2. The ticket's `## History` table in `body` — the *last* row whose `To`
///    column equals `current_state`; that row's `From` column is used. This
///    distinguishes e.g. `groomed → in_design` from `amend → in_design`.
/// 3. Fallback: every state with a `command:start` transition targeting
///    `current_state`. Used only if exactly one candidate exists.
/// 4. Otherwise, an error listing the candidate states and instructing the
///    caller to pass `--to <state>` explicitly.
pub fn resolve_recovery_target(
    body: &str,
    current_state: &str,
    workflow: &WorkflowConfig,
    explicit_to: Option<&str>,
) -> Result<String> {
    if let Some(to) = explicit_to {
        let state = workflow.states.iter().find(|s| s.id == to)
            .ok_or_else(|| anyhow!("unknown state {to:?} — not defined in workflow"))?;
        if state.terminal {
            bail!("cannot recover into terminal state {to:?}");
        }
        return Ok(to.to_string());
    }

    if let Some(from) = history_predecessor(body, current_state) {
        return Ok(from);
    }

    let candidates: Vec<&str> = workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|t| t.trigger == "command:start" && t.to == current_state))
        .map(|s| s.id.as_str())
        .collect();

    match candidates.len() {
        1 => Ok(candidates[0].to_string()),
        0 => bail!(
            "cannot determine a recovery target for {current_state:?} — no matching ## History \
             row and no command:start transition targets it; pass --to <state>"
        ),
        _ => bail!(
            "ambiguous recovery target for {current_state:?} — candidates: {} — pass --to <state>",
            candidates.join(", ")
        ),
    }
}

/// Find the `From` column of the last `## History` row whose `To` column
/// equals `current_state`. Returns `None` if the table has no such row.
fn history_predecessor(body: &str, current_state: &str) -> Option<String> {
    let idx = body.find("## History")?;
    let mut result = None;
    for line in body[idx..].lines() {
        let line = line.trim();
        if !line.starts_with('|') || !line.ends_with('|') {
            continue;
        }
        let cols: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cols.len() != 4 {
            continue;
        }
        if cols[0].eq_ignore_ascii_case("when") {
            continue; // header row
        }
        if !cols[0].is_empty() && cols[0].chars().all(|c| c == '-') {
            continue; // separator row
        }
        if cols[2] == current_state {
            result = Some(cols[1].to_string());
        }
    }
    result
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
  to      = "in_progress"
  trigger = "command:start"

[[states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

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
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

  [[states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  completion = "pr_or_epic_merge"
  on_failure = "merge_failed"

[[states]]
id    = "ready"
label = "Ready"

  [[states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
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
  to      = "in_progress"
  trigger = "command:start"

[[states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

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

    const RECOVER_WF: &str = r#"[[states]]
id    = "groomed"
label = "Groomed"

  [[states.transitions]]
  to      = "in_design"
  trigger = "command:start"

[[states]]
id    = "amend"
label = "Amend"

  [[states.transitions]]
  to      = "in_design"
  trigger = "command:start"

[[states]]
id    = "in_design"
label = "In Design"

[[states]]
id    = "ready"
label = "Ready"

  [[states.transitions]]
  to      = "in_progress"
  trigger = "command:start"

[[states]]
id    = "fix"
label = "Fix"

  [[states.transitions]]
  to      = "in_progress"
  trigger = "command:start"

[[states]]
id    = "in_progress"
label = "In Progress"

[[states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;

    fn history_body(rows: &[(&str, &str)]) -> String {
        let mut body = "## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|\n".to_string();
        for (from, to) in rows {
            body.push_str(&format!("| 2026-01-01T00:00Z | {from} | {to} | test |\n"));
        }
        body
    }

    #[test]
    fn resolve_recovery_target_explicit_to_wins() {
        let wf = parse_workflow(RECOVER_WF);
        let body = history_body(&[("groomed", "in_design")]);
        let target = resolve_recovery_target(&body, "in_design", &wf, Some("ready")).unwrap();
        assert_eq!(target, "ready");
    }

    #[test]
    fn resolve_recovery_target_explicit_to_rejects_unknown_state() {
        let wf = parse_workflow(RECOVER_WF);
        let err = resolve_recovery_target("", "in_design", &wf, Some("bogus")).unwrap_err();
        assert!(format!("{err}").contains("bogus"));
    }

    #[test]
    fn resolve_recovery_target_explicit_to_rejects_terminal_state() {
        let wf = parse_workflow(RECOVER_WF);
        let err = resolve_recovery_target("", "in_design", &wf, Some("closed")).unwrap_err();
        assert!(format!("{err}").contains("terminal"));
    }

    #[test]
    fn resolve_recovery_target_uses_history_amend_predecessor() {
        let wf = parse_workflow(RECOVER_WF);
        let body = history_body(&[("groomed", "amend"), ("amend", "in_design")]);
        let target = resolve_recovery_target(&body, "in_design", &wf, None).unwrap();
        assert_eq!(target, "amend");
    }

    #[test]
    fn resolve_recovery_target_uses_history_groomed_predecessor() {
        let wf = parse_workflow(RECOVER_WF);
        let body = history_body(&[("groomed", "in_design")]);
        let target = resolve_recovery_target(&body, "in_design", &wf, None).unwrap();
        assert_eq!(target, "groomed");
    }

    #[test]
    fn resolve_recovery_target_uses_last_matching_history_row() {
        let wf = parse_workflow(RECOVER_WF);
        // Ticket bounced groomed -> in_design -> question -> amend -> in_design;
        // the LAST row into in_design (from amend) must win, not the first.
        let body = history_body(&[
            ("groomed", "in_design"),
            ("in_design", "question"),
            ("question", "amend"),
            ("amend", "in_design"),
        ]);
        let target = resolve_recovery_target(&body, "in_design", &wf, None).unwrap();
        assert_eq!(target, "amend");
    }

    #[test]
    fn resolve_recovery_target_falls_back_to_config_when_no_history_match() {
        // No ## History section at all, but only "ready" has a command:start
        // transition into in_progress once "fix" is out of the picture.
        let single_candidate_wf = r#"[[states]]
id    = "ready"
label = "Ready"

  [[states.transitions]]
  to      = "in_progress"
  trigger = "command:start"

[[states]]
id    = "in_progress"
label = "In Progress"
"#;
        let wf = parse_workflow(single_candidate_wf);
        let target = resolve_recovery_target("## Spec\n\ncontent\n", "in_progress", &wf, None).unwrap();
        assert_eq!(target, "ready");
    }

    #[test]
    fn resolve_recovery_target_ambiguous_config_fallback_requires_to() {
        let wf = parse_workflow(RECOVER_WF);
        let body = "## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|\n";
        let err = resolve_recovery_target(body, "in_design", &wf, None).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("--to"), "expected --to guidance: {msg}");
        assert!(msg.contains("groomed") && msg.contains("amend"), "expected both candidates: {msg}");
    }

    #[test]
    fn resolve_recovery_target_no_candidates_requires_to() {
        let wf = parse_workflow(RECOVER_WF);
        let err = resolve_recovery_target("", "closed", &wf, None).unwrap_err();
        assert!(format!("{err}").contains("--to"));
    }
}
