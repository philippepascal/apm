use anyhow::{bail, Result};
use apm_core::{config::{CompletionStrategy, Config}, git, ticket};
use chrono::Utc;
use std::path::Path;

pub fn run(root: &Path, id: u32, new_state: String, no_aggressive: bool) -> Result<()> {
    let config = Config::load(root)?;
    let valid_states: std::collections::HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();
    if !valid_states.is_empty() && !valid_states.contains(new_state.as_str()) {
        let list: Vec<&str> = config.workflow.states.iter().map(|s| s.id.as_str()).collect();
        bail!("unknown state {:?} — valid states: {}", new_state, list.join(", "));
    }
    let aggressive = config.sync.aggressive && !no_aggressive;
    if aggressive {
        let prefix = format!("ticket/{id:04}-");
        if let Ok(branches) = git::ticket_branches(root) {
            if let Some(b) = branches.iter().find(|b| b.starts_with(&prefix)) {
                if let Err(e) = git::fetch_branch(root, b) {
                    eprintln!("warning: fetch failed: {e:#}");
                }
            }
        }
    }
    let mut tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found");
    };
    let old_state = t.frontmatter.state.clone();

    // Enforce transition rules if the current state defines any.
    // Terminal states (e.g. "closed") are always reachable regardless of rules.
    let target_is_terminal = config.workflow.states.iter()
        .find(|s| s.id == new_state)
        .map(|s| s.terminal)
        .unwrap_or(false);
    let completion = if !target_is_terminal {
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
                tr.map(|t| t.completion.clone()).unwrap_or_default()
            } else {
                CompletionStrategy::None
            }
        } else {
            CompletionStrategy::None
        }
    } else {
        CompletionStrategy::None
    };
    // Validate document for state-specific constraints.
    match new_state.as_str() {
        "specd" => {
            if let Ok(doc) = t.document() {
                let errors = doc.validate();
                if !errors.is_empty() {
                    let msgs: Vec<String> = errors.iter().map(|e| format!("  - {e}")).collect();
                    bail!("spec validation failed:\n{}", msgs.join("\n"));
                }
                if old_state == "ammend" {
                    let unchecked = doc.unchecked_amendments();
                    if !unchecked.is_empty() {
                        bail!("not all amendment requests are checked — mark them [x] before resubmitting");
                    }
                }
            }
        }
        "implemented" => {
            if let Ok(doc) = t.document() {
                let unchecked = doc.unchecked_criteria();
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
        ensure_amendment_section(&mut t.body);
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
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id:04}"));

    git::commit_to_branch(
        root,
        &branch,
        &rel_path,
        &content,
        &format!("ticket({id}): {old_state} → {new_state}"),
    )?;
    apm_core::logger::log("state_transition", &format!("#{id} {old_state} -> {new_state}"));


    match completion {
        CompletionStrategy::Pr => {
            git::push_branch(root, &branch)?;
            gh_pr_create_or_update(root, &branch, &config.project.default_branch, id, &t.frontmatter.title)?;
        }
        CompletionStrategy::Merge => {
            git::push_branch(root, &branch)?;
            merge_into_default(root, &branch, &config.project.default_branch)?;
        }
        CompletionStrategy::None => {
            if aggressive {
                if let Err(e) = git::push_branch(root, &branch) {
                    eprintln!("warning: push failed: {e:#}");
                }
            }
        }
    }

    println!("#{id}: {old_state} → {new_state}");
    Ok(())
}

fn gh_pr_create_or_update(root: &Path, branch: &str, default_branch: &str, id: u32, title: &str) -> Result<()> {
    // Check for an existing open PR on this branch.
    let existing = std::process::Command::new("gh")
        .args(["pr", "list", "--head", branch, "--state", "open", "--json", "number", "--jq", ".[0].number"])
        .current_dir(root)
        .output()?;

    let pr_num = String::from_utf8_lossy(&existing.stdout).trim().to_string();
    if !pr_num.is_empty() && pr_num != "null" {
        // PR already exists — nothing to update.
        println!("PR #{pr_num} already open for {branch}");
        return Ok(());
    }

    let body = format!("Closes #{id}");
    let out = std::process::Command::new("gh")
        .args(["pr", "create", "--base", default_branch, "--head", branch,
               "--title", title, "--body", &body])
        .current_dir(root)
        .output()?;

    if out.status.success() {
        let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
        println!("PR created: {url}");
    } else {
        bail!("gh pr create failed: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}

fn merge_into_default(root: &Path, branch: &str, default_branch: &str) -> Result<()> {
    // Fetch the default branch first.
    let _ = std::process::Command::new("git")
        .args(["fetch", "origin", default_branch])
        .current_dir(root)
        .status();

    // Check out a worktree on the default branch if we're not on it.
    let current = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()?;
    let current_branch = String::from_utf8_lossy(&current.stdout).trim().to_string();

    let merge_dir = if current_branch == default_branch {
        root.to_path_buf()
    } else {
        // Find an existing worktree on the default branch or use root.
        git::find_worktree_for_branch(root, default_branch)
            .unwrap_or_else(|| root.to_path_buf())
    };

    let out = std::process::Command::new("git")
        .args(["merge", "--no-ff", branch, "--no-edit"])
        .current_dir(&merge_dir)
        .output()?;

    if !out.status.success() {
        // Abort and report.
        let _ = std::process::Command::new("git")
            .args(["merge", "--abort"])
            .current_dir(&merge_dir)
            .status();
        bail!(
            "merge conflict — resolve manually and push: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }

    // Push the default branch.
    let push = std::process::Command::new("git")
        .args(["push", "origin", default_branch])
        .current_dir(&merge_dir)
        .output()?;

    if !push.status.success() {
        bail!("push failed: {}", String::from_utf8_lossy(&push.stderr).trim());
    }

    println!("Merged {branch} into {default_branch} and pushed.");
    Ok(())
}

pub fn ensure_amendment_section(body: &mut String) {
    if body.contains("### Amendment requests") {
        return;
    }
    let placeholder = "\n### Amendment requests\n\n<!-- Add amendment requests below -->\n";
    if let Some(pos) = body.find("### Out of scope") {
        let after = &body[pos..];
        let block_end = after[1..]
            .find("\n##")
            .map(|p| pos + 1 + p)
            .unwrap_or(body.len());
        body.insert_str(block_end, placeholder);
    } else if let Some(pos) = body.find("## History") {
        body.insert_str(pos, &format!("{}\n", placeholder));
    } else {
        body.push_str(placeholder);
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
