use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
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
    if !target_is_terminal {
        if let Some(state_cfg) = config.workflow.states.iter().find(|s| s.id == old_state) {
            if !state_cfg.transitions.is_empty() {
                let allowed: Vec<&str> = state_cfg.transitions.iter().map(|tr| tr.to.as_str()).collect();
                if !allowed.contains(&new_state.as_str()) {
                    bail!(
                        "no transition from {:?} to {:?} — valid transitions from {:?}: {}",
                        old_state, new_state, old_state,
                        allowed.join(", ")
                    );
                }
            }
        }
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

    let aggressive = config.sync.aggressive && !no_aggressive;
    if aggressive {
        if let Err(e) = git::push_branch(root, &branch) {
            eprintln!("warning: push failed: {e:#}");
        }
    }

    println!("#{id}: {old_state} → {new_state}");
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
