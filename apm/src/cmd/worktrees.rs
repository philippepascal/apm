use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use std::path::Path;

pub fn run(root: &Path, remove_id: Option<u32>) -> Result<()> {
    let config = Config::load(root)?;

    if let Some(id) = remove_id {
        return remove(root, &config, id);
    }

    list(root, &config)
}

fn list(root: &Path, config: &Config) -> Result<()> {
    let worktrees = git::list_ticket_worktrees(root)?;
    if worktrees.is_empty() {
        println!("No ticket worktrees provisioned.");
        return Ok(());
    }

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir).unwrap_or_default();

    for (wt_path, branch) in &worktrees {
        let ticket = tickets.iter().find(|t| {
            t.frontmatter.branch.as_deref() == Some(branch.as_str())
                || git::branch_name_from_path(&t.path).as_deref() == Some(branch.as_str())
        });

        let wt_name = wt_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(branch.as_str());

        match ticket {
            Some(t) => println!(
                "{}  {}  agent={}",
                wt_name,
                t.frontmatter.state,
                t.frontmatter.agent.as_deref().unwrap_or("—")
            ),
            None => println!("{}  (ticket not found)", wt_name),
        }
    }
    Ok(())
}

fn remove(root: &Path, config: &Config, id: u32) -> Result<()> {
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(t) = tickets.iter().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found");
    };

    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id:04}"));

    let Some(wt_path) = git::find_worktree_for_branch(root, &branch) else {
        bail!("no worktree found for ticket #{id} (branch: {branch})");
    };

    git::remove_worktree(root, &wt_path)?;
    println!("Removed worktree: {}", wt_path.display());
    Ok(())
}
