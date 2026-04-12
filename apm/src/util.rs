use anyhow::Result;
use apm_core::{config::Config, git, ticket, ticket_fmt, worktree};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// Run `git fetch --all` when `aggressive` is true; emit a warning on failure.
pub fn fetch_if_aggressive(root: &Path, aggressive: bool) {
    if aggressive {
        if let Err(e) = git::fetch_all(root) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }
}

/// Run `git fetch <branch>` when `aggressive` is true; emit a warning on failure.
pub fn fetch_branch_if_aggressive(root: &Path, branch: &str, aggressive: bool) {
    if aggressive {
        if let Err(e) = git::fetch_branch(root, branch) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }
}

/// Print `prompt`, flush stdout, read one line, return true iff the answer is "y".
pub fn prompt_yes_no(prompt: &str) -> io::Result<bool> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
}

/// Resolve a ticket ID argument to its worktree path and canonical ticket ID.
/// Loads config and tickets from git internally.
pub fn worktree_for_ticket(root: &Path, id_arg: &str) -> Result<(PathBuf, String)> {
    let config = Config::load(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let id = ticket::resolve_id_in_slice(&tickets, id_arg)?;
    let t = tickets
        .iter()
        .find(|t| t.frontmatter.id == id)
        .ok_or_else(|| anyhow::anyhow!("ticket {id:?} not found"))?;
    let branch = t
        .frontmatter
        .branch
        .clone()
        .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt = worktree::find_worktree_for_branch(root, &branch)
        .ok_or_else(|| anyhow::anyhow!("no worktree for ticket {id:?}"))?;
    Ok((wt, id))
}
