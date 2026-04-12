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
    let actor = std::env::var("APM_AGENT_NAME").unwrap_or_else(|_| "apm".into());
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
            let merge_target = t.frontmatter.target_branch.as_deref()
                .unwrap_or(&config.project.default_branch);
            let is_main = merge_target == config.project.default_branch;
            if let Err(e) = git::push_branch_tracking(root, &branch) {
                warnings.push(format!("warning: could not push {branch}: {e}"));
            }
            git::merge_into_default(root, &config, &branch, merge_target, is_main, &mut messages, &mut warnings)?;
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

