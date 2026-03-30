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

/// Read a file's content from a branch ref without changing working tree.
/// Prefers the local ref (reflects recent commits before push);
/// falls back to origin when no local ref exists.
pub fn read_from_branch(root: &Path, branch: &str, rel_path: &str) -> Result<String> {
    run(root, &["show", &format!("{branch}:{rel_path}")])
        .or_else(|_| run(root, &["show", &format!("origin/{branch}:{rel_path}")]))
}

/// All ticket/* branch names visible locally or remotely (deduplicated).
/// Local branches are included even when a remote exists, so that
/// unpushed branches (e.g. just created) are visible without a push.
pub fn ticket_branches(root: &Path) -> Result<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    let mut branches = Vec::new();

    let local = run(root, &["branch", "--list", "ticket/*"]).unwrap_or_default();
    for b in local.lines()
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim())
        .filter(|l| !l.is_empty())
    {
        if seen.insert(b.to_string()) {
            branches.push(b.to_string());
        }
    }

    let remote = run(root, &["branch", "-r", "--list", "origin/ticket/*"]).unwrap_or_default();
    for b in remote.lines()
        .map(|l| l.trim().trim_start_matches("origin/").to_string())
        .filter(|l| !l.is_empty())
    {
        if seen.insert(b.clone()) {
            branches.push(b);
        }
    }

    Ok(branches)
}

/// ticket/* branches that are merged into the default branch (remote or local),
/// including branches that were squash-merged (not detected by `--merged`).
pub fn merged_into_main(root: &Path, default_branch: &str) -> Result<Vec<String>> {
    let remote_ref = format!("refs/remotes/origin/{default_branch}");
    let remote_merged = format!("origin/{default_branch}");

    if run(root, &["rev-parse", "--verify", &remote_ref]).is_ok() {
        // Regular merges via remote.
        let regular_out = run(
            root,
            &["branch", "-r", "--merged", &remote_merged, "--list", "origin/ticket/*"],
        )
        .unwrap_or_default();
        let mut merged: Vec<String> = regular_out
            .lines()
            .map(|l| l.trim().trim_start_matches("origin/").to_string())
            .filter(|l| !l.is_empty())
            .collect();
        let merged_set: std::collections::HashSet<&str> = merged.iter().map(|s| s.as_str()).collect();

        // Squash-merge detection for branches not caught by --merged.
        let all_remote = run(root, &["branch", "-r", "--list", "origin/ticket/*"])
            .unwrap_or_default();
        let candidates: Vec<String> = all_remote
            .lines()
            .map(|l| l.trim().trim_start_matches("origin/").to_string())
            .filter(|l| !l.is_empty() && !merged_set.contains(l.as_str()))
            .collect();
        merged.extend(squash_merged(root, &remote_merged, candidates)?);
        return Ok(merged);
    }

    // Fall back to local branch.
    let local_ref = format!("refs/heads/{default_branch}");
    if run(root, &["rev-parse", "--verify", &local_ref]).is_err() {
        return Ok(vec![]);
    }
    let regular_out = run(
        root,
        &["branch", "--merged", default_branch, "--list", "ticket/*"],
    )
    .unwrap_or_default();
    let mut merged: Vec<String> = regular_out
        .lines()
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let merged_set: std::collections::HashSet<&str> = merged.iter().map(|s| s.as_str()).collect();

    let all_local = run(root, &["branch", "--list", "ticket/*"]).unwrap_or_default();
    let candidates: Vec<String> = all_local
        .lines()
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim().to_string())
        .filter(|l| !l.is_empty() && !merged_set.contains(l.as_str()))
        .collect();
    merged.extend(squash_merged(root, default_branch, candidates)?);
    Ok(merged)
}

/// Detect branches squash-merged into `main_ref`: every commit on the branch
/// has an equivalent patch already in `main_ref` (via git's patch-id mechanism).
fn squash_merged(root: &Path, main_ref: &str, candidates: Vec<String>) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for branch in candidates {
        let range = format!("{main_ref}...{branch}");
        let out = run(root, &[
            "log", "--cherry-pick", "--right-only", "--no-merges", "--format=%H", &range,
        ])
        .unwrap_or_default();
        if out.trim().is_empty() {
            result.push(branch);
        }
    }
    Ok(result)
}

/// Find the directory of an existing permanent worktree for the given branch.
/// Returns None if no such worktree is registered.
pub fn find_worktree_for_branch(root: &Path, branch: &str) -> Option<std::path::PathBuf> {
    let out = run(root, &["worktree", "list", "--porcelain"]).ok()?;
    let mut current_path: Option<std::path::PathBuf> = None;
    for line in out.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            current_path = Some(std::path::PathBuf::from(p));
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
pub fn list_ticket_worktrees(root: &Path) -> Result<Vec<(std::path::PathBuf, String)>> {
    let out = run(root, &["worktree", "list", "--porcelain"])?;
    let main = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    let mut result = Vec::new();
    let mut current_path: Option<std::path::PathBuf> = None;
    for line in out.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            current_path = Some(std::path::PathBuf::from(p));
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
pub fn ensure_worktree(root: &Path, worktrees_base: &Path, branch: &str) -> Result<std::path::PathBuf> {
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
pub fn remove_worktree(root: &Path, wt_path: &Path) -> Result<()> {
    run(root, &["worktree", "remove", &wt_path.to_string_lossy()])
        .map(|_| ())
}

/// Commit a file to a specific branch without disturbing the current working tree.
///
/// If a permanent worktree exists for the branch, commits there directly.
/// If the caller is already on the target branch, commits directly.
/// Otherwise uses a temporary git worktree.
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

    // If a permanent worktree exists for this branch, commit there directly.
    if let Some(wt_path) = find_worktree_for_branch(root, branch) {
        // Fast-forward to remote if remote is ahead, so our commit lands on top of it.
        let remote_ref = format!("origin/{branch}");
        if run(root, &["rev-parse", "--verify", &remote_ref]).is_ok() {
            let _ = run(&wt_path, &["merge", "--ff-only", &remote_ref]);
        }
        let full_path = wt_path.join(rel_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full_path, content)?;
        let _ = run(&wt_path, &["add", rel_path]);
        let _ = run(&wt_path, &["commit", "-m", message]);
        crate::logger::log("commit_to_branch", &format!("{branch} {message}"));
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
        crate::logger::log("commit_to_branch", &format!("{branch} {message}"));
        return Ok(());
    }

    let result = try_worktree_commit(root, branch, rel_path, content, message);
    if result.is_ok() {
        crate::logger::log("commit_to_branch", &format!("{branch} {message}"));
    }
    result
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

/// Generate an 8-character hex ticket ID from local entropy (timestamp + PID).
/// No network access or shared state is required. Birthday collision probability
/// at N=1000 tickets: N²/2³² ≈ 0.023% — acceptable at this scale.
pub fn gen_hex_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs();
    let nanos = dur.subsec_nanos() as u64;
    let pid = std::process::id() as u64;
    // splitmix64-style mixing for good bit avalanche
    let a = secs.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(nanos);
    let b = (a ^ (a >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    let c = (b ^ (b >> 27)).wrapping_mul(0x94d049bb133111eb);
    let result = (c ^ (c >> 31)) ^ pid.wrapping_mul(0x6c62272e07bb0142);
    format!("{:016x}", result)[..8].to_string()
}

/// Find a ticket branch matching a user-supplied ID argument (prefix or full hex).
/// Normalizes plain integers (e.g. 35 → 0035) via `ticket::normalize_id_arg`.
pub fn resolve_ticket_branch(branches: &[String], arg: &str) -> Result<String> {
    let prefix = crate::ticket::normalize_id_arg(arg)?;
    let matches: Vec<&String> = branches.iter()
        .filter(|b| {
            b.strip_prefix("ticket/")
                .and_then(|s| s.split('-').next())
                .map(|id| id.starts_with(prefix.as_str()))
                .unwrap_or(false)
        })
        .collect();
    match matches.len() {
        0 => anyhow::bail!("no ticket matches '{prefix}'"),
        1 => Ok(matches[0].clone()),
        _ => {
            let mut msg = format!("error: prefix '{prefix}' is ambiguous");
            for b in &matches {
                let id = b.strip_prefix("ticket/")
                    .and_then(|s| s.split('-').next())
                    .unwrap_or(b.as_str());
                msg.push_str(&format!("\n  {id}  ({})", b));
            }
            anyhow::bail!("{msg}")
        }
    }
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

/// List all files in a directory on a branch (non-recursive).
pub fn list_files_on_branch(root: &Path, branch: &str, dir: &str) -> Result<Vec<String>> {
    let tree_ref = format!("{branch}:{dir}");
    let out = run(root, &["ls-tree", "--name-only", &tree_ref])
        .or_else(|_| run(root, &["ls-tree", "--name-only", &format!("origin/{branch}:{dir}")]))?;
    Ok(out.lines()
        .filter(|l| !l.is_empty())
        .map(|l| format!("{dir}/{l}"))
        .collect())
}

/// Commit multiple files to a branch in a single commit without disturbing the working tree.
pub fn commit_files_to_branch(
    root: &Path,
    branch: &str,
    files: &[(&str, String)],
    message: &str,
) -> Result<()> {
    if !has_commits(root) {
        for (rel_path, content) in files {
            let local_path = root.join(rel_path);
            if let Some(parent) = local_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&local_path, content)?;
        }
        return Ok(());
    }

    if let Some(wt_path) = find_worktree_for_branch(root, branch) {
        for (rel_path, content) in files {
            let full_path = wt_path.join(rel_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, content)?;
            let _ = run(&wt_path, &["add", rel_path]);
        }
        run(&wt_path, &["commit", "-m", message])?;
        crate::logger::log("commit_files_to_branch", &format!("{branch} {message}"));
        return Ok(());
    }

    if current_branch(root).ok().as_deref() == Some(branch) {
        for (rel_path, content) in files {
            let local_path = root.join(rel_path);
            if let Some(parent) = local_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&local_path, content)?;
            let _ = run(root, &["add", rel_path]);
        }
        run(root, &["commit", "-m", message])?;
        crate::logger::log("commit_files_to_branch", &format!("{branch} {message}"));
        return Ok(());
    }

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
        let sha = run(root, &["rev-parse", &format!("refs/heads/{branch}")])?;
        run(root, &["worktree", "add", "--detach", &wt_path.to_string_lossy(), &sha])?;
        let _ = run(&wt_path, &["checkout", "-B", branch]);
    } else {
        run(root, &["worktree", "add", &wt_path.to_string_lossy(), branch])?;
    }

    let result = (|| -> Result<()> {
        for (rel_path, content) in files {
            let full_path = wt_path.join(rel_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, content)?;
            run(&wt_path, &["add", rel_path])?;
        }
        run(&wt_path, &["commit", "-m", message])?;
        Ok(())
    })();

    let _ = run(root, &["worktree", "remove", "--force", &wt_path.to_string_lossy()]);
    let _ = std::fs::remove_dir_all(&wt_path);

    if result.is_ok() {
        crate::logger::log("commit_files_to_branch", &format!("{branch} {message}"));
    }
    result
}

/// Get the commit SHA at the tip of a local branch.
pub fn branch_tip(root: &Path, branch: &str) -> Option<String> {
    run(root, &["rev-parse", &format!("refs/heads/{branch}")]).ok()
}

/// Get the commit SHA at the tip of the remote tracking ref for a branch.
pub fn remote_branch_tip(root: &Path, branch: &str) -> Option<String> {
    run(root, &["rev-parse", &format!("refs/remotes/origin/{branch}")]).ok()
}

/// Check if `commit` is a git ancestor of `of_ref` (i.e. reachable from `of_ref`).
/// Uses `git merge-base --is-ancestor`.
pub fn is_ancestor(root: &Path, commit: &str, of_ref: &str) -> bool {
    Command::new("git")
        .current_dir(root)
        .args(["merge-base", "--is-ancestor", commit, of_ref])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn fetch_branch(root: &Path, branch: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("git")
        .args(["fetch", "origin", branch])
        .current_dir(root)
        .status()?;
    if !status.success() {
        anyhow::bail!("git fetch failed");
    }
    Ok(())
}

pub fn push_branch(root: &Path, branch: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("git")
        .args(["push", "origin", &format!("{branch}:{branch}")])
        .current_dir(root)
        .status()?;
    if !status.success() {
        anyhow::bail!("git push failed");
    }
    Ok(())
}

pub fn has_remote(root: &Path) -> bool {
    run(root, &["remote", "get-url", "origin"]).is_ok()
}

/// Merge `branch` into `default_branch` (fast-forward or merge commit).
/// Pushes `default_branch` to origin when a remote exists.
pub fn merge_branch_into_default(root: &Path, branch: &str, default_branch: &str) -> Result<()> {
    let _ = run(root, &["fetch", "origin", default_branch]);

    let merge_dir = if current_branch(root).ok().as_deref() == Some(default_branch) {
        root.to_path_buf()
    } else {
        find_worktree_for_branch(root, default_branch).unwrap_or_else(|| root.to_path_buf())
    };

    if let Err(e) = run(&merge_dir, &["merge", "--no-ff", branch, "--no-edit"]) {
        let _ = run(&merge_dir, &["merge", "--abort"]);
        anyhow::bail!("merge failed: {e:#}");
    }

    if has_remote(root) {
        if let Err(e) = run(&merge_dir, &["push", "origin", default_branch]) {
            eprintln!("warning: push {default_branch} failed: {e:#}");
        }
    }
    Ok(())
}
