use anyhow::Result;
use apm_core::{config::Config, worktree};
use std::path::Path;

pub fn run(root: &Path, remove_id: Option<&str>) -> Result<()> {
    let config = Config::load(root)?;

    if let Some(id_arg) = remove_id {
        return remove(root, &config, id_arg);
    }

    list(root, &config)
}

fn list(root: &Path, config: &Config) -> Result<()> {
    let wt_tickets = worktree::list_worktrees_with_tickets(root, &config.tickets.dir)?;
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

fn remove(root: &Path, _config: &Config, id_arg: &str) -> Result<()> {
    let (wt_path, _id) = crate::util::worktree_for_ticket(root, id_arg)?;
    worktree::remove_worktree(root, &wt_path, false)?;
    println!("Removed worktree: {}", wt_path.display());
    Ok(())
}
