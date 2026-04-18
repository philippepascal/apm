use anyhow::{bail, Result};
use crate::{config::{CompletionStrategy, Config}, git, review, ticket, ticket_fmt};
use chrono::Utc;
use std::path::{Path, PathBuf};

pub struct TransitionOutput {
    pub id: String,
    pub old_state: String,
    pub new_state: String,
    pub worktree_path: Option<PathBuf>,
    pub warnings: Vec<String>,
    pub messages: Vec<String>,
}

pub fn transition(root: &Path, id_arg: &str, new_state: String, no_aggressive: bool, force: bool) -> Result<TransitionOutput> {
    let mut warnings: Vec<String> = Vec::new();
    let mut messages: Vec<String> = Vec::new();

    let config = Config::load(root)?;
    let valid_states: std::collections::HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();
    if !valid_states.is_empty() && !valid_states.contains(new_state.as_str()) {
        let list: Vec<&str> = config.workflow.states.iter().map(|s| s.id.as_str()).collect();
        bail!("unknown state {:?} — valid states: {}", new_state, list.join(", "));
    }
    let aggressive = config.sync.aggressive && !no_aggressive;

    let mut tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let id = ticket::resolve_id_in_slice(&tickets, id_arg)?;

    if aggressive {
        let branches = git::ticket_branches(root).unwrap_or_default();
        if let Some(b) = branches.iter().find(|b| {
            b.strip_prefix("ticket/")
                .and_then(|s| s.split('-').next())
                .map(|bid| bid == id.as_str())
                .unwrap_or(false)
        }) {
            if let Err(e) = git::fetch_branch(root, b) {
                warnings.push(format!("warning: fetch failed: {e:#}"));
            }
        }
    }

    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket {id:?} not found");
    };
    let old_state = t.frontmatter.state.clone();

    let target_is_terminal = config.workflow.states.iter()
        .find(|s| s.id == new_state)
        .map(|s| s.terminal)
        .unwrap_or(false);
    let completion = if force {
        CompletionStrategy::None
    } else if !target_is_terminal {
        if let Some(state_cfg) = config.workflow.states.iter().find(|s| s.id == old_state) {
            if !state_cfg.transitions.is_empty() {
                let tr = state_cfg.transitions.iter().find(|tr| tr.to == new_state);
                if tr.is_none() {
                    let allowed: Vec<&str> = state_cfg.transitions.iter().map(|tr| tr.to.as_str()).collect();
                    bail!(
                        "no transition from {:?} to {:?} — valid transitions from {:?}: {}",
                        old_state, new_state, old_state,
                        allowed.join(", ")
                    );
                }
                let found = tr.unwrap();
                if let Some(ref w) = found.warning {
                    warnings.push(format!("⚠ {w}"));
                }
                found.completion.clone()
            } else {
                CompletionStrategy::None
            }
        } else {
            CompletionStrategy::None
        }
    } else {
        CompletionStrategy::None
    };

    match new_state.as_str() {
        "specd" => {
            if let Ok(doc) = t.document() {
                let errors = doc.validate(&config.ticket.sections);
                if !errors.is_empty() {
                    let msgs: Vec<String> = errors.iter().map(|e| format!("  - {e}")).collect();
                    bail!("spec validation failed:\n{}", msgs.join("\n"));
                }
                if old_state == "ammend" {
                    let unchecked = doc.unchecked_tasks("Amendment requests");
                    if !unchecked.is_empty() {
                        bail!("not all amendment requests are checked — mark them [x] before resubmitting");
                    }
                }
            }
        }
        "implemented" => {
            if let Ok(doc) = t.document() {
                let unchecked = doc.unchecked_tasks("Acceptance criteria");
                if !unchecked.is_empty() {
                    bail!(
                        "not all acceptance criteria are checked — mark them [x] before transitioning to implemented"
                    );
                }
            }
        }
        _ => {}
    }

    let now = Utc::now();
    let actor = crate::config::resolve_caller_name();
    t.frontmatter.state = new_state.clone();
    t.frontmatter.updated_at = Some(now);
    if new_state == "ammend" {
        review::ensure_amendment_section(&mut t.body);
    }
    append_history(&mut t.body, &old_state, &new_state, &now.format("%Y-%m-%dT%H:%MZ").to_string(), &actor);

    let content = t.serialize()?;
    let rel_path = format!(
        "{}/{}",
        config.tickets.dir.to_string_lossy(),
        t.path.file_name().unwrap().to_string_lossy()
    );
    let branch = t
        .frontmatter
        .branch
        .clone()
        .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));

    git::commit_to_branch(
        root,
        &branch,
        &rel_path,
        &content,
        &format!("ticket({id}): {old_state} → {new_state}"),
    )?;
    crate::logger::log("state_transition", &format!("{id:?} {old_state} -> {new_state}"));

    match completion {
        CompletionStrategy::Pr => {
            git::push_branch_tracking(root, &branch)?;
            let pr_base = t.frontmatter.target_branch.as_deref()
                .unwrap_or(&config.project.default_branch);
            crate::github::gh_pr_create_or_update(root, &branch, pr_base, &id, &t.frontmatter.title, &format!("Closes #{id}"), &mut messages)?;
        }
        CompletionStrategy::Merge => {
            let merge_result = {
                let merge_target = t.frontmatter.target_branch.as_deref()
                    .unwrap_or(&config.project.default_branch);
                let is_main = merge_target == config.project.default_branch;
                if let Err(e) = git::push_branch_tracking(root, &branch) {
                    warnings.push(format!("warning: could not push {branch}: {e}"));
                }
                git::merge_into_default(root, &config, &branch, merge_target, is_main, &mut messages, &mut warnings)
            };
            if let Err(merge_err) = merge_result {
                let merge_err_msg = format!("{merge_err:#}");
                let fail_now = Utc::now();
                t.frontmatter.state = "merge_failed".to_string();
                t.frontmatter.updated_at = Some(fail_now);
                set_merge_notes(&mut t.body, &merge_err_msg);
                append_history(&mut t.body, &new_state, "merge_failed", &fail_now.format("%Y-%m-%dT%H:%MZ").to_string(), &actor);
                let fallback_content = match t.serialize() {
                    Ok(c) => c,
                    Err(_) => return Err(merge_err),
                };
                if git::commit_to_branch(root, &branch, &rel_path, &fallback_content, &format!("ticket({id}): {new_state} → merge_failed")).is_err() {
                    return Err(merge_err);
                }
                crate::logger::log("state_transition", &format!("{id:?} {new_state} -> merge_failed"));
                return Ok(TransitionOutput {
                    id: id.clone(),
                    old_state: old_state.clone(),
                    new_state: "merge_failed".to_string(),
                    worktree_path: None,
                    warnings,
                    messages,
                });
            }
        }
        CompletionStrategy::PrOrEpicMerge => {
            git::push_branch_tracking(root, &branch)?;
            if let Some(ref target) = t.frontmatter.target_branch {
                git::merge_into_default(root, &config, &branch, target, false, &mut messages, &mut warnings)?;
            } else {
                crate::github::gh_pr_create_or_update(root, &branch, &config.project.default_branch, &id, &t.frontmatter.title, &format!("Closes #{id}"), &mut messages)?;
            }
        }
        CompletionStrategy::Pull => {
            git::pull_default(root, &config.project.default_branch, &mut warnings)?;
        }
        CompletionStrategy::None => {
            if aggressive {
                if let Err(e) = git::push_branch_tracking(root, &branch) {
                    warnings.push(format!("warning: push failed: {e:#}"));
                }
            }
        }
    }

    let worktree_path = if new_state == "in_design" {
        Some(crate::worktree::provision_worktree(root, &config, &branch, &mut warnings)?)
    } else {
        None
    };

    Ok(TransitionOutput {
        id,
        old_state,
        new_state,
        worktree_path,
        warnings,
        messages,
    })
}


pub fn available_transitions(config: &crate::config::Config, current_state: &str) -> Vec<(String, String, String)> {
    let terminal_ids: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();

    let state_cfg = config.workflow.states.iter().find(|s| s.id == current_state);

    if let Some(sc) = state_cfg {
        if !sc.transitions.is_empty() {
            return sc.transitions.iter()
                .filter(|tr| !tr.trigger.starts_with("event:"))
                .map(|tr| (tr.to.clone(), tr.label.clone(), tr.hint.clone()))
                .collect();
        }
    }

    // No explicit transitions: all non-terminal, non-current states are valid.
    config.workflow.states.iter()
        .filter(|s| s.id != current_state && !terminal_ids.contains(&s.id.as_str()))
        .map(|s| (s.id.clone(), s.label.clone(), String::new()))
        .collect()
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct TransitionOption {
    pub to: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

pub fn compute_valid_transitions(state: &str, config: &crate::config::Config) -> Vec<TransitionOption> {
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

fn set_merge_notes(body: &mut String, notes: &str) {
    const SECTION: &str = "### Merge notes";

    // Remove existing section if present.
    if let Some(start) = body.find(SECTION) {
        let actual_start = if start > 0 && body.as_bytes().get(start - 1) == Some(&b'\n') {
            start - 1
        } else {
            start
        };
        let after_header = start + SECTION.len();
        let end = body[after_header..]
            .find("\n##")
            .map(|i| after_header + i)
            .unwrap_or(body.len());
        body.replace_range(actual_start..end, "");
    }

    // Insert before ## History or append.
    let block = format!("\n{SECTION}\n\n{notes}\n");
    if let Some(pos) = body.find("\n## History") {
        body.insert_str(pos, &block);
    } else {
        body.push_str(&block);
    }
}

pub fn append_history(body: &mut String, from: &str, to: &str, when: &str, by: &str) {
    let row = format!("| {when} | {from} | {to} | {by} |");
    if body.contains("## History") {
        if !body.ends_with('\n') {
            body.push('\n');
        }
        body.push_str(&row);
        body.push('\n');
    } else {
        body.push_str(&format!(
            "\n## History\n\n| When | From | To | By |\n|------|------|----|----|\n{row}\n"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_transitions() -> crate::config::Config {
        let toml = concat!(
            "[project]\nname = \"test\"\n",
            "[tickets]\ndir = \"tickets\"\n",
            "[[workflow.states]]\n",
            "id = \"new\"\nlabel = \"New\"\n",
            "[[workflow.states.transitions]]\n",
            "to = \"ready\"\nlabel = \"Mark ready\"\n",
            "[[workflow.states.transitions]]\n",
            "to = \"closed\"\nlabel = \"\"\n",
            "warning = \"This will close the ticket\"\n",
            "[[workflow.states]]\n",
            "id = \"ready\"\nlabel = \"Ready\"\n",
            "[[workflow.states]]\n",
            "id = \"closed\"\nlabel = \"Closed\"\nterminal = true\n",
        );
        toml::from_str(toml).unwrap()
    }

    #[test]
    fn set_merge_notes_inserts_before_history() {
        let mut body = "## Spec\n\ncontent\n\n## History\n\n| row |".to_string();
        set_merge_notes(&mut body, "conflict error");
        assert!(body.contains("### Merge notes\n\nconflict error\n"));
        let notes_pos = body.find("### Merge notes").unwrap();
        let hist_pos = body.find("## History").unwrap();
        assert!(notes_pos < hist_pos);
    }

    #[test]
    fn set_merge_notes_appends_when_no_history() {
        let mut body = "## Spec\n\ncontent".to_string();
        set_merge_notes(&mut body, "error msg");
        assert!(body.contains("### Merge notes\n\nerror msg\n"));
    }

    #[test]
    fn set_merge_notes_overwrites_existing_section() {
        let mut body = "## Spec\n\n### Merge notes\n\nold error\n\n## History\n\n| row |".to_string();
        set_merge_notes(&mut body, "new error");
        assert!(body.contains("### Merge notes\n\nnew error\n"));
        assert!(!body.contains("old error"));
        let notes_pos = body.find("### Merge notes").unwrap();
        let hist_pos = body.find("## History").unwrap();
        assert!(notes_pos < hist_pos);
    }

    #[test]
    fn compute_valid_transitions_returns_expected_options() {
        let config = config_with_transitions();
        let opts = compute_valid_transitions("new", &config);
        assert_eq!(opts.len(), 2);
        assert_eq!(opts[0].to, "ready");
        assert_eq!(opts[0].label, "Mark ready");
        assert!(opts[0].warning.is_none());
        assert_eq!(opts[1].to, "closed");
        assert_eq!(opts[1].label, "-> closed");
        assert_eq!(opts[1].warning.as_deref(), Some("This will close the ticket"));
    }

    #[test]
    fn compute_valid_transitions_unknown_state_returns_empty() {
        let config = config_with_transitions();
        let opts = compute_valid_transitions("nonexistent", &config);
        assert!(opts.is_empty());
    }
}
