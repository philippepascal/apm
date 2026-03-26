use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

fn run(dir: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .context("git not found")?;
    if !out.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

pub fn current_branch(root: &Path) -> Result<String> {
    run(root, &["branch", "--show-current"])
}

pub fn has_commits(root: &Path) -> bool {
    run(root, &["rev-parse", "HEAD"]).is_ok()
}

pub fn fetch_all(root: &Path) -> Result<()> {
    run(root, &["fetch", "--all", "--quiet"]).map(|_| ())
}

/// Read a file's content from a remote branch ref without changing working tree.
pub fn read_from_branch(root: &Path, branch: &str, rel_path: &str) -> Result<String> {
    run(root, &["show", &format!("origin/{branch}:{rel_path}")])
        .or_else(|_| run(root, &["show", &format!("{branch}:{rel_path}")]))
}

/// All remote ticket/* branch names (without the origin/ prefix).
pub fn ticket_branches(root: &Path) -> Result<Vec<String>> {
    let out = run(root, &["branch", "-r", "--list", "origin/ticket/*"]).unwrap_or_default();
    Ok(out
        .lines()
        .map(|l| l.trim().trim_start_matches("origin/").to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// Remote ticket/* branches that are merged into origin/main.
pub fn merged_into_main(root: &Path) -> Result<Vec<String>> {
    if run(root, &["rev-parse", "--verify", "origin/main"]).is_err() {
        return Ok(vec![]);
    }
    let out = run(
        root,
        &[
            "branch",
            "-r",
            "--merged",
            "origin/main",
            "--list",
            "origin/ticket/*",
        ],
    )
    .unwrap_or_default();
    Ok(out
        .lines()
        .map(|l| l.trim().trim_start_matches("origin/").to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// Commit a file to a specific branch without disturbing the current working tree.
///
/// If the caller is already on the target branch, commits directly.
/// Otherwise uses a temporary git worktree. Push is attempted but non-fatal.
pub fn commit_to_branch(
    root: &Path,
    branch: &str,
    rel_path: &str,
    content: &str,
    message: &str,
) -> Result<()> {
    // Always update the local cache first.
    let local_path = root.join(rel_path);
    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&local_path, content)?;

    // If the repo has no commits, we cannot use worktrees. Stop here.
    if !has_commits(root) {
        return Ok(());
    }

    // If already on the target branch, commit directly.
    if current_branch(root).ok().as_deref() == Some(branch) {
        let _ = run(root, &["add", rel_path]);
        let _ = run(root, &["commit", "-m", message]);
        let _ = run(root, &["push", "origin", branch]);
        return Ok(());
    }

    let _ = try_worktree_commit(root, branch, rel_path, content, message);
    Ok(())
}

fn try_worktree_commit(
    root: &Path,
    branch: &str,
    rel_path: &str,
    content: &str,
    message: &str,
) -> Result<()> {
    // Use nanosecond timestamp for uniqueness across parallel calls and sequential reuse.
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let wt_path = std::env::temp_dir().join(format!(
        "apm-{}-{}-{}",
        std::process::id(),
        unique,
        branch.replace('/', "-"),
    ));

    let has_remote = run(root, &["rev-parse", "--verify", &format!("refs/remotes/origin/{branch}")]).is_ok();
    let has_local = run(root, &["rev-parse", "--verify", &format!("refs/heads/{branch}")]).is_ok();

    if has_remote {
        run(root, &["worktree", "add", "--detach", &wt_path.to_string_lossy(), &format!("origin/{branch}")])?;
        let _ = run(&wt_path, &["checkout", "-B", branch]);
    } else if has_local {
        // Use detached approach to avoid "already checked out" errors.
        let sha = run(root, &["rev-parse", &format!("refs/heads/{branch}")])?;
        run(root, &["worktree", "add", "--detach", &wt_path.to_string_lossy(), &sha])?;
        let _ = run(&wt_path, &["checkout", "-B", branch]);
    } else {
        run(root, &["worktree", "add", "-b", branch, &wt_path.to_string_lossy(), "HEAD"])?;
    }

    let result = (|| -> Result<()> {
        let full_path = wt_path.join(rel_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full_path, content)?;
        run(&wt_path, &["add", rel_path])?;
        run(&wt_path, &["commit", "-m", message])?;
        Ok(())
    })();

    let _ = run(root, &["worktree", "remove", "--force", &wt_path.to_string_lossy()]);
    let _ = std::fs::remove_dir_all(&wt_path);

    result?;

    // Push the local branch (non-fatal).
    let _ = run(root, &["push", "origin", branch]);
    Ok(())
}

/// Allocate the next ticket ID from the apm/meta branch.
/// Falls back to a local NEXT_ID file if git ops are unavailable.
pub fn next_ticket_id(root: &Path, tickets_dir: &Path) -> Result<u32> {
    if !has_commits(root) {
        return crate::ticket::next_id(tickets_dir);
    }

    let meta_branch = "apm/meta";
    let _ = run(root, &["fetch", "origin", meta_branch]);

    let id: u32 = read_from_branch(root, meta_branch, "NEXT_ID")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1);

    // Try to commit the incremented ID back to apm/meta.
    // Non-fatal: if this fails, we still return the id and accept the risk
    // of a duplicate (acceptable for single-machine use without a remote).
    let new_id = id + 1;
    let _ = commit_to_branch(
        root,
        meta_branch,
        "NEXT_ID",
        &format!("{new_id}\n"),
        &format!("meta: allocate ticket #{id}"),
    );

    Ok(id)
}

/// Derive the ticket branch name from the ticket file path.
/// e.g. tickets/0001-my-ticket.md → ticket/0001-my-ticket
pub fn branch_name_from_path(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    Some(format!("ticket/{stem}"))
}
