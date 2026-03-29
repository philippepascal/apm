use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::path::Path;

pub fn run(root: &Path, id: u32, no_aggressive: bool) -> Result<()> {
    let new_agent = std::env::var("APM_AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("APM_AGENT_NAME is not set"))?;

    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;
    let mut tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found");
    };

    let fm = &t.frontmatter;
    let old_agent = match &fm.agent {
        None => bail!("no agent assigned — use `apm start` instead"),
        Some(a) => a.clone(),
    };

    if old_agent == new_agent {
        // Still ensure worktree exists.
        let branch = t.frontmatter.branch.clone()
            .or_else(|| git::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{id:04}"));
        let wt_path = ensure_worktree(root, &config, &branch)?;
        println!("#{id}: already assigned to {new_agent}");
        println!("Worktree: {}", wt_path.display());
        return Ok(());
    }

    let now = Utc::now();
    t.frontmatter.agent = Some(new_agent.clone());
    t.frontmatter.updated_at = Some(now);
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
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

    let wt_path = ensure_worktree(root, &config, &branch)?;

    if aggressive {
        if let Err(e) = git::push_branch(root, &branch) {
            eprintln!("warning: push failed: {e:#}");
        }
    }

    println!("#{id}: agent handoff {old_agent} → {new_agent} (branch: {branch})");
    println!("Worktree: {}", wt_path.display());
    Ok(())
}

fn ensure_worktree(root: &Path, config: &Config, branch: &str) -> anyhow::Result<std::path::PathBuf> {
    if let Some(existing) = git::find_worktree_for_branch(root, branch) {
        return Ok(existing);
    }
    let wt_name = branch.replace('/', "-");
    let worktrees_base = root.join(&config.worktrees.dir);
    std::fs::create_dir_all(&worktrees_base)?;
    let wt_path = worktrees_base.join(&wt_name);
    git::add_worktree(root, &wt_path, branch)?;
    Ok(wt_path)
}
