use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, no_aggressive: bool) -> Result<()> {
    let new_agent = apm_core::start::resolve_agent_name();

    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    if aggressive {
        let branches = git::ticket_branches(root).unwrap_or_default();
        if let Ok(b) = git::resolve_ticket_branch(&branches, id_arg) {
            if let Err(e) = git::fetch_branch(root, &b) {
                eprintln!("warning: fetch failed: {e:#}");
            }
        }
    }

    let mut tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let id = ticket::resolve_id_in_slice(&tickets, id_arg)?;

    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket {id:?} not found");
    };

    let now = Utc::now();
    let result = ticket::handoff(t, &new_agent, now)?;

    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));

    let wt_path = ensure_worktree(root, &config, &branch)?;

    if let Some(old_agent) = result {
        let content = t.serialize()?;
        let rel_path = format!(
            "{}/{}",
            config.tickets.dir.to_string_lossy(),
            t.path.file_name().unwrap().to_string_lossy()
        );

        git::commit_to_branch(root, &branch, &rel_path, &content,
            &format!("ticket({id}): agent handoff {old_agent} → {new_agent}"))?;

        if aggressive {
            if let Err(e) = git::push_branch(root, &branch) {
                eprintln!("warning: push failed: {e:#}");
            }
        }

        println!("{id}: agent handoff {old_agent} → {new_agent} (branch: {branch})");
    } else {
        println!("{id}: already assigned to {new_agent}");
    }

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
