use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::path::Path;
use std::process::Command;

pub fn run(root: &Path, id: u32) -> Result<()> {
    let new_agent = std::env::var("APM_AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("APM_AGENT_NAME is not set"))?;

    let config = Config::load(root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    let mut tickets = ticket::load_all(&tickets_dir)?;
    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found — run `apm sync` to refresh");
    };

    let fm = &t.frontmatter;
    let old_agent = match &fm.agent {
        None => bail!("no agent assigned — use `apm start` instead"),
        Some(a) => a.clone(),
    };

    if fm.state != "in_progress" && fm.state != "implemented" {
        bail!("ticket #{id} is in state {:?} — take requires in_progress or implemented", fm.state);
    }

    if old_agent == new_agent {
        println!("#{id}: already assigned to {new_agent} — nothing to do");
        return Ok(());
    }

    let now = Utc::now();
    t.frontmatter.agent = Some(new_agent.clone());
    t.frontmatter.updated_at = Some(now);
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
    // Record handoff: From = old agent, To = new agent (state unchanged).
    super::state::append_history(&mut t.body, &old_agent, &new_agent, &when, "handoff");

    let content = t.serialize()?;
    let rel_path = format!(
        "{}/{}",
        config.tickets.dir.to_string_lossy(),
        t.path.file_name().unwrap().to_string_lossy()
    );
    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id:04}"));

    git::commit_to_branch(root, &branch, &rel_path, &content,
        &format!("ticket({id}): agent handoff {old_agent} → {new_agent}"))?;

    let branch_local = Command::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{branch}")])
        .current_dir(root).output().map(|o| o.status.success()).unwrap_or(false);

    if !branch_local {
        let _ = Command::new("git").args(["fetch", "origin", &branch]).current_dir(root).status();
    }

    let checkout = Command::new("git").args(["checkout", &branch]).current_dir(root).status()?;
    if !checkout.success() {
        bail!("checkout of {branch} failed");
    }

    println!("#{id}: agent handoff {old_agent} → {new_agent} (branch: {branch})");
    Ok(())
}
