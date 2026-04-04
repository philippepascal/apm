use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use std::path::Path;

pub fn run(root: &Path, remove_id: Option<&str>) -> Result<()> {
    let config = Config::load(root)?;

    if let Some(id_arg) = remove_id {
        return remove(root, &config, id_arg);
    }

    list(root, &config)
}

fn list(root: &Path, config: &Config) -> Result<()> {
    let wt_tickets = ticket::list_worktrees_with_tickets(root, &config.tickets.dir)?;
    if wt_tickets.is_empty() {
        println!("No ticket worktrees provisioned.");
        return Ok(());
    }

    for (wt_path, branch, t) in &wt_tickets {
        let wt_name = wt_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(branch.as_str());

        match t {
            Some(t) => println!(
                "{}  {}",
                wt_name,
                t.frontmatter.state,
            ),
            None => println!("{}  (ticket not found)", wt_name),
        }
    }
    Ok(())
}

fn remove(root: &Path, config: &Config, id_arg: &str) -> Result<()> {
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let id = ticket::resolve_id_in_slice(&tickets, id_arg)?;
    let Some(t) = tickets.iter().find(|t| t.frontmatter.id == id) else {
        bail!("ticket {id:?} not found");
    };

    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));

    let Some(wt_path) = git::find_worktree_for_branch(root, &branch) else {
        bail!("no worktree found for ticket {id:?} (branch: {branch})");
    };

    git::remove_worktree(root, &wt_path, false)?;
    println!("Removed worktree: {}", wt_path.display());
    Ok(())
}
