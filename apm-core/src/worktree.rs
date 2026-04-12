use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::config::Config;
use crate::git_util::run;
use crate::ticket::{Ticket, load_all_from_git};

/// Find the directory of an existing permanent worktree for the given branch.
/// Returns None if no such worktree is registered.
pub fn find_worktree_for_branch(root: &Path, branch: &str) -> Option<PathBuf> {
    let out = run(root, &["worktree", "list", "--porcelain"]).ok()?;
    let mut current_path: Option<PathBuf> = None;
    for line in out.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(p));
        } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
            if b == branch {
                return current_path;
            }
        }
    }
    None
}

/// List all permanent worktrees for ticket/* branches.
/// Returns (worktree_path, branch_name) pairs, skipping the main worktree.
pub fn list_ticket_worktrees(root: &Path) -> Result<Vec<(PathBuf, String)>> {
    let out = run(root, &["worktree", "list", "--porcelain"])?;
    let main = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    let mut result = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    for line in out.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(p));
        } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
            if b.starts_with("ticket/") {
                if let Some(p) = &current_path {
                    if p.canonicalize().unwrap_or_else(|_| p.clone()) != main {
                        result.push((p.clone(), b.to_string()));
                    }
                }
            }
        }
    }
    Ok(result)
}

/// Find the worktree for `branch` or create one under `worktrees_base`.
/// Returns the canonical worktree path. Idempotent.
pub fn ensure_worktree(root: &Path, worktrees_base: &Path, branch: &str) -> Result<PathBuf> {
    if let Some(existing) = find_worktree_for_branch(root, branch) {
        return Ok(existing);
    }
    let wt_name = branch.replace('/', "-");
    std::fs::create_dir_all(worktrees_base)?;
    let wt_path = worktrees_base.join(&wt_name);
    add_worktree(root, &wt_path, branch)?;
    Ok(find_worktree_for_branch(root, branch).unwrap_or(wt_path))
}

/// Add a permanent worktree for the given branch at wt_path.
/// Fetches the branch locally first if needed.
pub fn add_worktree(root: &Path, wt_path: &Path, branch: &str) -> Result<()> {
    let has_local = run(root, &["rev-parse", "--verify", &format!("refs/heads/{branch}")]).is_ok();
    if !has_local {
        let _ = run(root, &["fetch", "origin", branch]);
    }
    run(root, &["worktree", "add", &wt_path.to_string_lossy(), branch])?;
    crate::logger::log("add_worktree", &format!("{}", wt_path.display()));
    Ok(())
}

/// Remove a permanent worktree.
pub fn remove_worktree(root: &Path, wt_path: &Path, force: bool) -> Result<()> {
    clean_agent_dirs(root, wt_path);
    let path_str = wt_path.to_string_lossy();
    if force {
        run(root, &["worktree", "remove", "--force", &path_str]).map(|_| ())
    } else {
        run(root, &["worktree", "remove", &path_str]).map(|_| ())
    }
}

/// Copy agent config directories from the main repo into a worktree.
/// Only copies directories that are NOT tracked by git (untracked/gitignored).
pub fn sync_agent_dirs(root: &Path, wt_path: &Path, agent_dirs: &[String], warnings: &mut Vec<String>) {
    for dir_name in agent_dirs {
        let src = root.join(dir_name);
        if !src.is_dir() {
            continue;
        }
        if is_tracked(root, dir_name) {
            continue;
        }
        let dst = wt_path.join(dir_name);
        if let Err(e) = copy_dir_recursive(&src, &dst) {
            warnings.push(format!("warning: could not copy {dir_name} to worktree: {e}"));
        }
    }
}

/// Remove agent config directories from a worktree before cleanup.
/// Only removes directories that are NOT tracked by git.
fn clean_agent_dirs(root: &Path, wt_path: &Path) {
    let config = match Config::load(root) {
        Ok(c) => c,
        Err(_) => return,
    };
    for dir_name in &config.worktrees.agent_dirs {
        let dir = wt_path.join(dir_name);
        if !dir.is_dir() {
            continue;
        }
        if is_tracked(root, dir_name) {
            continue;
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

fn is_tracked(root: &Path, path: &str) -> bool {
    Command::new("git")
        .args(["ls-files", "--error-unmatch", path])
        .current_dir(root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        std::fs::remove_dir_all(dst)?;
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn provision_worktree(root: &Path, config: &Config, branch: &str, warnings: &mut Vec<String>) -> Result<PathBuf> {
    let worktrees_base = root.join(&config.worktrees.dir);
    let wt = ensure_worktree(root, &worktrees_base, branch)?;
    sync_agent_dirs(root, &wt, &config.worktrees.agent_dirs, warnings);
    Ok(wt)
}

pub fn list_worktrees_with_tickets(
    root: &Path,
    tickets_dir: &Path,
) -> Result<Vec<(PathBuf, String, Option<Ticket>)>> {
    let worktrees = list_ticket_worktrees(root)?;
    let tickets = load_all_from_git(root, tickets_dir).unwrap_or_default();
    let result = worktrees.into_iter().map(|(wt_path, branch)| {
        let ticket = tickets.iter().find(|t| {
            t.frontmatter.branch.as_deref() == Some(branch.as_str())
                || crate::ticket_fmt::branch_name_from_path(&t.path).as_deref() == Some(branch.as_str())
        }).cloned();
        (wt_path, branch, ticket)
    }).collect();
    Ok(result)
}
