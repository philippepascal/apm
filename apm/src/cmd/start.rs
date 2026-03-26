use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::path::Path;
use std::process::Command;

pub fn run(root: &Path, id: u32) -> Result<()> {
    let agent_name = std::env::var("APM_AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("APM_AGENT_NAME is not set"))?;

    let config = Config::load(root)?;
    let actionable: std::collections::HashSet<&str> = config.agents.actionable_states
        .iter()
        .map(|s| s.as_str())
        .collect();

    let tickets_dir = root.join(&config.tickets.dir);
    let mut tickets = ticket::load_all(&tickets_dir)?;
    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found — run `apm sync` to refresh");
    };

    let fm = &t.frontmatter;
    if fm.agent.is_some() {
        bail!("ticket already claimed — run `apm next`");
    }
    if !actionable.contains(fm.state.as_str()) {
        bail!(
            "ticket #{id} is in state {:?} — must be one of: {}",
            fm.state,
            config.agents.actionable_states.join(", ")
        );
    }

    let now = Utc::now();
    let old_state = t.frontmatter.state.clone();
    t.frontmatter.agent = Some(agent_name.clone());
    t.frontmatter.state = "in_progress".into();
    t.frontmatter.updated_at = Some(now);
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
    super::state::append_history(&mut t.body, &old_state, "in_progress", &when, &agent_name);

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

    git::commit_to_branch(root, &branch, &rel_path, &content, &format!("ticket({id}): start — {old_state} → in_progress"))?;

    // Ensure branch is present locally before checking out.
    let branch_local = Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{branch}")])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !branch_local {
        let _ = Command::new("git")
            .args(["fetch", "origin", &branch])
            .current_dir(root)
            .status();
    }

    // The ticket file may be in the working tree with an older state (e.g. the
    // supervisor checked it out to read the spec). Reset it to the branch HEAD
    // so the full checkout below doesn't see "local changes".
    let _ = Command::new("git")
        .args(["checkout", &branch, "--", &rel_path])
        .current_dir(root)
        .status();

    let checkout = Command::new("git")
        .args(["checkout", &branch])
        .current_dir(root)
        .status()?;

    if !checkout.success() {
        bail!("checkout of {branch} failed");
    }

    println!("#{id}: {old_state} → in_progress (agent: {agent_name}, branch: {branch})");
    Ok(())
}
