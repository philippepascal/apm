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
    // Prefer remote branches; fall back to local when no remote is configured.
    let remote = run(root, &["branch", "-r", "--list", "origin/ticket/*"]).unwrap_or_default();
    if !remote.trim().is_empty() {
        return Ok(remote
            .lines()
            .map(|l| l.trim().trim_start_matches("origin/").to_string())
            .filter(|l| !l.is_empty())
            .collect());
    }
    let local = run(root, &["branch", "--list", "ticket/*"]).unwrap_or_default();
    Ok(local
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// ticket/* branches that are merged into main (remote or local).
pub fn merged_into_main(root: &Path) -> Result<Vec<String>> {
    // Try remote main first.
    if run(root, &["rev-parse", "--verify", "refs/remotes/origin/main"]).is_ok() {
        let out = run(
            root,
            &["branch", "-r", "--merged", "origin/main", "--list", "origin/ticket/*"],
        )
        .unwrap_or_default();
        return Ok(out
            .lines()
            .map(|l| l.trim().trim_start_matches("origin/").to_string())
            .filter(|l| !l.is_empty())
            .collect());
    }
    // Fall back to local main.
    if run(root, &["rev-parse", "--verify", "refs/heads/main"]).is_err() {
        return Ok(vec![]);
    }
    let out = run(
        root,
        &["branch", "--merged", "main", "--list", "ticket/*"],
    )
    .unwrap_or_default();
    Ok(out
        .lines()
        .map(|l| l.trim().to_string())
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
    // If the repo has no commits, write directly to the working tree (no worktree support yet).
    if !has_commits(root) {
        let local_path = root.join(rel_path);
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&local_path, content)?;
        return Ok(());
    }

    // If already on the target branch, write to working tree and commit directly.
    if current_branch(root).ok().as_deref() == Some(branch) {
        let local_path = root.join(rel_path);
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&local_path, content)?;
        let _ = run(root, &["add", rel_path]);
        let _ = run(root, &["commit", "-m", message]);
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

    result
}

/// Allocate the next ticket ID from the apm/meta branch using an optimistic-lock
/// protocol. Retries up to 5 times on push rejection (concurrent allocation).
/// Falls back to local NEXT_ID file if the repo has no commits.
pub fn next_ticket_id(root: &Path, tickets_dir: &Path) -> Result<u32> {
    if !has_commits(root) {
        return crate::ticket::next_id(tickets_dir);
    }

    const MAX_ATTEMPTS: u32 = 5;
    let meta_branch = "apm/meta";

    for attempt in 0..MAX_ATTEMPTS {
        let _ = run(root, &["fetch", "origin", meta_branch]);

        let id: u32 = read_from_branch(root, meta_branch, "NEXT_ID")
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(1);

        match write_meta(root, meta_branch, id, id + 1) {
            Ok(()) => return Ok(id),
            Err(_) if attempt + 1 < MAX_ATTEMPTS => continue,
            Err(e) => anyhow::bail!(
                "could not allocate ticket ID after {MAX_ATTEMPTS} attempts: {e:#}"
            ),
        }
    }

    unreachable!()
}

/// Initialise apm/meta with NEXT_ID = 1. Called by `apm init`. Non-fatal.
pub fn init_meta_branch(root: &Path) {
    if has_commits(root) {
        let _ = write_meta(root, "apm/meta", 0, 1);
    }
}

/// Commit new_next to NEXT_ID on the meta branch and push.
/// Returns Err if the push is rejected — the caller should retry.
fn write_meta(root: &Path, branch: &str, claimed_id: u32, new_next: u32) -> Result<()> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let wt_path = std::env::temp_dir().join(format!(
        "apm-{}-{}-meta",
        std::process::id(),
        unique,
    ));

    let has_remote = run(root, &["rev-parse", "--verify", &format!("refs/remotes/origin/{branch}")]).is_ok();
    let has_local  = run(root, &["rev-parse", "--verify", &format!("refs/heads/{branch}")]).is_ok();
    let is_new = !has_remote && !has_local;

    // When both local and remote exist, prefer whichever is ahead.
    // Local may be ahead if it has unpushed commits (e.g. a manually fixed NEXT_ID).
    let local_ahead = has_remote && has_local && run(root, &[
        "merge-base", "--is-ancestor",
        &format!("refs/remotes/origin/{branch}"),
        &format!("refs/heads/{branch}"),
    ]).is_ok();

    if has_remote && !local_ahead {
        run(root, &["worktree", "add", "--detach", &wt_path.to_string_lossy(), &format!("origin/{branch}")])?;
        run(&wt_path, &["checkout", "-B", branch])?;
    } else if has_local {
        let sha = run(root, &["rev-parse", &format!("refs/heads/{branch}")])?;
        run(root, &["worktree", "add", "--detach", &wt_path.to_string_lossy(), &sha])?;
        run(&wt_path, &["checkout", "-B", branch])?;
    } else {
        run(root, &["worktree", "add", "-b", branch, &wt_path.to_string_lossy(), "HEAD"])?;
    }

    let commit_result = (|| -> Result<()> {
        if is_new {
            // Remove files inherited from the parent commit so apm/meta
            // contains only NEXT_ID.
            let _ = run(&wt_path, &["rm", "-rf", "--ignore-unmatch", "."]);
        }
        std::fs::write(wt_path.join("NEXT_ID"), format!("{new_next}\n"))?;
        run(&wt_path, &["add", "NEXT_ID"])?;
        let msg = if claimed_id > 0 {
            format!("meta: allocate ticket #{claimed_id}")
        } else {
            "meta: initialize".to_string()
        };
        run(&wt_path, &["commit", "-m", &msg])?;
        Ok(())
    })();

    let _ = run(root, &["worktree", "remove", "--force", &wt_path.to_string_lossy()]);
    let _ = std::fs::remove_dir_all(&wt_path);
    commit_result?;

    // Push — this is the step that fails on concurrent allocation.
    // In pure-git mode (no remote), skip push; local commit is sufficient.
    let has_origin = run(root, &["remote", "get-url", "origin"]).is_ok();
    if has_origin {
        run(root, &["push", "origin", branch])?;
    }
    Ok(())
}

/// Push all local ticket/* branches that have commits not yet on origin.
/// Non-fatal: logs warnings on push failure. No-op when no origin is configured.
pub fn push_ticket_branches(root: &Path) {
    if run(root, &["remote", "get-url", "origin"]).is_err() {
        return;
    }
    let out = match run(root, &["branch", "--list", "ticket/*"]) {
        Ok(o) => o,
        Err(_) => return,
    };
    for branch in out.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        let range = format!("origin/{branch}..{branch}");
        let count = run(root, &["rev-list", "--count", &range])
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);
        if count > 0 {
            if let Err(e) = run(root, &["push", "origin", branch]) {
                eprintln!("warning: push {branch} failed: {e:#}");
            }
        }
    }
}

/// Derive the ticket branch name from the ticket file path.
/// e.g. tickets/0001-my-ticket.md → ticket/0001-my-ticket
pub fn branch_name_from_path(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    Some(format!("ticket/{stem}"))
}
