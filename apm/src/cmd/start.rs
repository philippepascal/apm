use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Local;
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

    let old_state = t.frontmatter.state.clone();
    t.frontmatter.agent = Some(agent_name.clone());
    t.frontmatter.state = "in_progress".into();
    t.frontmatter.updated = Some(Local::now().date_naive());

    let today = Local::now().format("%Y-%m-%d");
    let history_row = format!("| {today} | {agent_name} | {old_state} → in_progress | |");
    if t.body.contains("## History") {
        if !t.body.ends_with('\n') {
            t.body.push('\n');
        }
        t.body.push_str(&history_row);
        t.body.push('\n');
    } else {
        t.body.push_str(&format!(
            "\n## History\n\n| Date | Actor | Transition | Note |\n|------|-------|------------|------|\n{history_row}\n"
        ));
    }

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
