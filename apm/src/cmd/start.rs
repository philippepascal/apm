use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::path::Path;

pub fn run(root: &Path, id: u32) -> Result<()> {
    let agent_name = std::env::var("APM_AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("APM_AGENT_NAME is not set"))?;

    let config = Config::load(root)?;
    let actionable = config.actionable_states_for("agent");

    let mut tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found");
    };

    let fm = &t.frontmatter;
    if fm.agent.is_some() {
        bail!("ticket already claimed — run `apm next`");
    }
    if !actionable.contains(&fm.state.as_str()) {
        bail!(
            "ticket #{id} is in state {:?} — not agent-actionable\n\
             Agent-actionable states: {}",
            fm.state,
            if actionable.is_empty() { "(none configured)".to_string() } else { actionable.join(", ") }
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

    // Provision permanent worktree.
    // Worktree dir name: ticket-<id>-<slug> (branch name with / replaced by -)
    let wt_name = branch.replace('/', "-");
    let worktrees_base = root.join(&config.worktrees.dir);
    std::fs::create_dir_all(&worktrees_base)?;
    let wt_path = worktrees_base.join(&wt_name);

    if git::find_worktree_for_branch(root, &branch).is_none() {
        git::add_worktree(root, &wt_path, &branch)?;
    }

    let wt_display = git::find_worktree_for_branch(root, &branch)
        .unwrap_or(wt_path);

    println!("#{id}: {old_state} → in_progress (agent: {agent_name}, branch: {branch})");
    println!("Worktree: {}", wt_display.display());
    Ok(())
}
