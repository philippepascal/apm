use anyhow::{bail, Context, Result};
use crate::config::Config;
use crate::worktree::{find_worktree_for_branch, ensure_worktree};
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn run(dir: &Path, args: &[&str]) -> Result<String> {
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

/// Push the default branch to origin if it is ahead.
/// Silently succeeds if origin is unreachable or already up-to-date.
pub fn push_default_branch(root: &Path, default_branch: &str) -> Result<()> {
    let local = match run(root, &["rev-parse", default_branch]) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return Ok(()),
    };
    let remote = run(root, &["rev-parse", &format!("origin/{default_branch}")])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if local == remote {
        return Ok(());
    }
    run(root, &["push", "origin", default_branch, "--quiet"]).map(|_| ())
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
        let merged_set: std::collections::HashSet<String> = merged.iter().cloned().collect();

        // Squash-merge detection for remote branches not caught by --merged.
        // Pass full origin/ refs so merge-base resolution works even without a local branch.
        let all_remote = run(root, &["branch", "-r", "--list", "origin/ticket/*"])
            .unwrap_or_default();
        let remote_candidates: Vec<String> = all_remote
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| {
                let stripped = l.strip_prefix("origin/").unwrap_or(l.as_str());
                !l.is_empty() && !merged_set.contains(stripped)
            })
            .collect();
        let remote_squashed = squash_merged(root, &remote_merged, remote_candidates)?;
        // Strip origin/ prefix before adding to merged.
        merged.extend(remote_squashed.into_iter().map(|b| {
            b.strip_prefix("origin/").unwrap_or(&b).to_string()
        }));

        // Also check local-only ticket branches whose remote tracking ref was deleted
        // (e.g. GitHub auto-deletes the branch after squash merge).
        let remote_stripped: std::collections::HashSet<String> = all_remote
            .lines()
            .map(|l| l.trim().trim_start_matches("origin/").to_string())
            .filter(|l| !l.is_empty())
            .collect();
        let merged_now: std::collections::HashSet<String> = merged.iter().cloned().collect();
        let all_local = run(root, &["branch", "--list", "ticket/*"]).unwrap_or_default();
        let local_only: Vec<String> = all_local
            .lines()
            .map(|l| l.trim().trim_start_matches(['*', '+']).trim().to_string())
            .filter(|l| {
                !l.is_empty()
                    && !remote_stripped.contains(l)
                    && !merged_now.contains(l)
            })
            .collect();
        merged.extend(squash_merged(root, &remote_merged, local_only)?);
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

/// Detect branches squash-merged into `main_ref` using the commit-tree + cherry algorithm.
///
/// For each candidate ref, we create a virtual squash commit whose tree equals
/// the branch tip's tree and whose parent is the merge-base with main. Then
/// `git cherry` compares that squash commit's patch-id against commits already
/// in main. A `-` prefix means main has a commit with the same aggregate diff.
fn squash_merged(root: &Path, main_ref: &str, candidates: Vec<String>) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for branch in candidates {
        let merge_base = match run(root, &["merge-base", main_ref, &branch]) {
            Ok(mb) => mb,
            Err(_) => continue,
        };
        let branch_tip = match run(root, &["rev-parse", &format!("{branch}^{{commit}}")]) {
            Ok(t) => t,
            Err(_) => continue,
        };
        // Already an ancestor — caught by --merged.
        if branch_tip == merge_base {
            continue;
        }
        // Virtual squash commit: aggregate diff from merge_base to branch tip.
        let squash_commit = match run(root, &[
            "commit-tree", &format!("{branch}^{{tree}}"),
            "-p", &merge_base,
            "-m", "squash",
        ]) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // `git cherry main squash_commit`: prints `- sha` when main already has that patch.
        let cherry_out = match run(root, &["cherry", main_ref, &squash_commit]) {
            Ok(o) => o,
            Err(_) => continue,
        };
        if cherry_out.trim().starts_with('-') {
            result.push(branch);
        }
    }
    Ok(result)
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
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let wt_path = std::env::temp_dir().join(format!(
        "apm-{}-{}-{}",
        std::process::id(),
        seq,
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


/// Push all local ticket/* branches that have commits not yet on origin.
/// Non-fatal: logs warnings on push failure. No-op when no origin is configured.
pub fn push_ticket_branches(root: &Path, warnings: &mut Vec<String>) {
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
                warnings.push(format!("warning: push {branch} failed: {e:#}"));
            }
        }
    }
}

/// Update local refs for all ticket/* branches to match origin after a fetch.
/// Skips branches that are currently checked out in any worktree (main or permanent).
/// All failures are handled as warnings — this function never panics or returns an error.
pub fn sync_local_ticket_refs(root: &Path, warnings: &mut Vec<String>) {
    // Collect all branches currently checked out across all worktrees.
    let checked_out: std::collections::HashSet<String> = {
        let mut set = std::collections::HashSet::new();
        if let Ok(out) = run(root, &["worktree", "list", "--porcelain"]) {
            for line in out.lines() {
                if let Some(b) = line.strip_prefix("branch refs/heads/") {
                    set.insert(b.to_string());
                }
            }
        }
        set
    };

    // Enumerate all origin ticket branches.
    let remote_refs = match run(root, &["for-each-ref", "--format=%(refname:short)", "refs/remotes/origin/ticket/"]) {
        Ok(o) => o,
        Err(_) => return,
    };

    for remote_name in remote_refs.lines().filter(|l| !l.is_empty()) {
        // remote_name is like "origin/ticket/<slug>"; strip the "origin/" prefix.
        let branch = match remote_name.strip_prefix("origin/") {
            Some(b) => b.to_string(),
            None => continue,
        };

        if checked_out.contains(&branch) {
            continue;
        }

        // Resolve the origin SHA.
        let sha = match run(root, &["rev-parse", &format!("refs/remotes/{remote_name}")]) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Create or update the local ref unconditionally.
        if let Err(e) = run(root, &["update-ref", &format!("refs/heads/{branch}"), &sha]) {
            warnings.push(format!("warning: could not update local ref {branch}: {e:#}"));
        }
    }
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

/// Resolve a branch name to a commit SHA.
/// Prefers `origin/<branch>`; falls back to local `<branch>`.
pub fn resolve_branch_sha(root: &Path, branch: &str) -> Result<String> {
    run(root, &["rev-parse", &format!("origin/{branch}")])
        .or_else(|_| run(root, &["rev-parse", branch]))
        .with_context(|| format!("branch '{branch}' not found locally or on origin"))
}

/// Create a local branch pointing at a specific commit SHA.
pub fn create_branch_at(root: &Path, branch: &str, sha: &str) -> Result<()> {
    run(root, &["branch", branch, sha]).map(|_| ())
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

pub fn push_branch_tracking(root: &Path, branch: &str) -> anyhow::Result<()> {
    let out = std::process::Command::new("git")
        .args(["push", "--set-upstream", "origin", &format!("{branch}:{branch}")])
        .current_dir(root)
        .output()?;
    if !out.status.success() {
        anyhow::bail!("git push failed: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}

pub fn has_remote(root: &Path) -> bool {
    run(root, &["remote", "get-url", "origin"]).is_ok()
}

/// Merge `branch` into `default_branch` (fast-forward or merge commit).
/// Pushes `default_branch` to origin when a remote exists.
/// List remote ticket/* branches with their last commit date.
/// Returns (branch_name_without_origin_prefix, commit_date) pairs.
pub fn remote_ticket_branches_with_dates(
    root: &Path,
) -> Result<Vec<(String, chrono::DateTime<chrono::Utc>)>> {
    use chrono::{TimeZone, Utc};
    let out = Command::new("git")
        .current_dir(root)
        .args([
            "for-each-ref",
            "refs/remotes/origin/ticket/",
            "--format=%(refname:short) %(creatordate:unix)",
        ])
        .output()
        .context("git for-each-ref failed")?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut result = Vec::new();
    for line in stdout.lines() {
        let mut parts = line.splitn(2, ' ');
        let refname = parts.next().unwrap_or("").trim();
        let ts_str = parts.next().unwrap_or("").trim();
        let branch = refname.trim_start_matches("origin/");
        if branch.is_empty() {
            continue;
        }
        if let Ok(ts) = ts_str.parse::<i64>() {
            if let Some(dt) = Utc.timestamp_opt(ts, 0).single() {
                result.push((branch.to_string(), dt));
            }
        }
    }
    Ok(result)
}

/// Delete a remote branch on origin.
pub fn delete_remote_branch(root: &Path, branch: &str) -> Result<()> {
    let status = Command::new("git")
        .current_dir(root)
        .args(["push", "origin", "--delete", branch])
        .status()
        .context("git push origin --delete failed")?;
    if !status.success() {
        anyhow::bail!("git push origin --delete {branch} failed");
    }
    Ok(())
}

/// Move files on a branch in a single commit.
/// Each element of `moves` is (old_rel_path, new_rel_path, content).
/// Writes each new file, stages it, then removes each old file via `git rm`.
/// Uses the same permanent-worktree / temp-worktree pattern as commit_files_to_branch.
pub fn move_files_on_branch(
    root: &Path,
    branch: &str,
    moves: &[(&str, &str, &str)],
    message: &str,
) -> Result<()> {
    if !has_commits(root) {
        for (old, new, content) in moves {
            let new_path = root.join(new);
            if let Some(parent) = new_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&new_path, content)?;
            let old_path = root.join(old);
            let _ = std::fs::remove_file(&old_path);
        }
        return Ok(());
    }

    let do_moves = |wt: &Path| -> Result<()> {
        for (old, new, content) in moves {
            let new_path = wt.join(new);
            if let Some(parent) = new_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&new_path, content)?;
            run(wt, &["add", new])?;
            run(wt, &["rm", "--force", "--quiet", old])?;
        }
        run(wt, &["commit", "-m", message])?;
        Ok(())
    };

    if let Some(wt_path) = find_worktree_for_branch(root, branch) {
        let remote_ref = format!("origin/{branch}");
        if run(root, &["rev-parse", "--verify", &remote_ref]).is_ok() {
            let _ = run(&wt_path, &["merge", "--ff-only", &remote_ref]);
        }
        let result = do_moves(&wt_path);
        if result.is_ok() {
            crate::logger::log("move_files_on_branch", &format!("{branch} {message}"));
        }
        return result;
    }

    if current_branch(root).ok().as_deref() == Some(branch) {
        let result = do_moves(root);
        if result.is_ok() {
            crate::logger::log("move_files_on_branch", &format!("{branch} {message}"));
        }
        return result;
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

    let result = do_moves(&wt_path);
    let _ = run(root, &["worktree", "remove", "--force", &wt_path.to_string_lossy()]);
    let _ = std::fs::remove_dir_all(&wt_path);
    if result.is_ok() {
        crate::logger::log("move_files_on_branch", &format!("{branch} {message}"));
    }
    result
}

pub fn merge_branch_into_default(root: &Path, branch: &str, default_branch: &str, warnings: &mut Vec<String>) -> Result<()> {
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
            warnings.push(format!("warning: push {default_branch} failed: {e:#}"));
        }
    }
    Ok(())
}

pub fn merge_into_default(root: &Path, config: &Config, branch: &str, default_branch: &str, skip_push: bool, messages: &mut Vec<String>, _warnings: &mut Vec<String>) -> Result<()> {
    let _ = std::process::Command::new("git")
        .args(["fetch", "origin", default_branch])
        .current_dir(root)
        .status();

    let current = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()?;
    let current_branch = String::from_utf8_lossy(&current.stdout).trim().to_string();

    let merge_dir = if current_branch == default_branch {
        root.to_path_buf()
    } else {
        let worktrees_base = root.join(&config.worktrees.dir);
        ensure_worktree(root, &worktrees_base, default_branch)?
    };

    let out = std::process::Command::new("git")
        .args(["merge", "--no-ff", branch, "--no-edit"])
        .current_dir(&merge_dir)
        .output()?;

    if !out.status.success() {
        let _ = std::process::Command::new("git")
            .args(["merge", "--abort"])
            .current_dir(&merge_dir)
            .status();
        bail!(
            "merge conflict — resolve manually and push: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }

    if skip_push {
        messages.push(format!("Merged {branch} into {default_branch} (local only)."));
    } else {
        push_branch(&merge_dir, default_branch)?;
        messages.push(format!("Merged {branch} into {default_branch} and pushed to origin."));
    }
    Ok(())
}

pub fn pull_default(root: &Path, default_branch: &str, warnings: &mut Vec<String>) -> Result<()> {
    let fetch = std::process::Command::new("git")
        .args(["fetch", "origin", default_branch])
        .current_dir(root)
        .output();

    match fetch {
        Err(e) => {
            warnings.push(format!("warning: fetch failed: {e:#}"));
            return Ok(());
        }
        Ok(out) if !out.status.success() => {
            warnings.push(format!(
                "warning: fetch failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
            return Ok(());
        }
        _ => {}
    }

    let current = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()?;
    let current_branch = String::from_utf8_lossy(&current.stdout).trim().to_string();

    let merge_dir = if current_branch == default_branch {
        root.to_path_buf()
    } else {
        find_worktree_for_branch(root, default_branch)
            .unwrap_or_else(|| root.to_path_buf())
    };

    let remote_ref = format!("origin/{default_branch}");
    let out = std::process::Command::new("git")
        .args(["merge", "--ff-only", &remote_ref])
        .current_dir(&merge_dir)
        .output()?;

    if !out.status.success() {
        warnings.push(format!("warning: could not fast-forward {default_branch} — pull manually"));
    }

    Ok(())
}

pub fn is_worktree_dirty(path: &Path) -> bool {
    let Ok(out) = Command::new("git")
        .args(["-C", &path.to_string_lossy(), "status", "--porcelain"])
        .output()
    else {
        return false;
    };
    !out.stdout.is_empty()
}

pub fn local_branch_exists(root: &Path, branch: &str) -> bool {
    Command::new("git")
        .args(["-C", &root.to_string_lossy(), "rev-parse", "--verify", &format!("refs/heads/{branch}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn delete_local_branch(root: &Path, branch: &str, warnings: &mut Vec<String>) {
    let Ok(out) = Command::new("git")
        .args(["-C", &root.to_string_lossy(), "branch", "-D", branch])
        .output()
    else {
        warnings.push(format!("warning: could not delete branch {branch}: command failed"));
        return;
    };
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        warnings.push(format!("warning: could not delete branch {branch}: {stderr}"));
    }
}

pub fn prune_remote_tracking(root: &Path, branch: &str) {
    let _ = Command::new("git")
        .args(["-C", &root.to_string_lossy(), "branch", "-dr", &format!("origin/{branch}")])
        .output();
}

pub fn stage_files(root: &Path, files: &[&str]) -> Result<()> {
    let mut args = vec!["add"];
    args.extend_from_slice(files);
    run(root, &args).map(|_| ())
}

pub fn commit(root: &Path, message: &str) -> Result<()> {
    run(root, &["commit", "-m", message]).map(|_| ())
}

pub fn git_config_get(root: &Path, key: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["-C", &root.to_string_lossy(), "config", key])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

pub fn merge_ref(dir: &Path, refname: &str, warnings: &mut Vec<String>) -> Option<String> {
    let out = match Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "merge", refname, "--no-edit"])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            warnings.push(format!("warning: merge {refname} failed: {e}"));
            return None;
        }
    };
    if out.status.success() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if stdout.contains("Already up to date") {
            None
        } else {
            Some(format!("Merged {refname} into branch."))
        }
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        warnings.push(format!("warning: merge {refname} failed: {stderr}"));
        None
    }
}

pub fn is_file_tracked(root: &Path, path: &str) -> bool {
    Command::new("git")
        .args(["ls-files", "--error-unmatch", path])
        .current_dir(root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn main_worktree_root(root: &Path) -> Option<PathBuf> {
    let out = run(root, &["worktree", "list", "--porcelain"]).ok()?;
    out.lines()
        .next()
        .and_then(|line| line.strip_prefix("worktree "))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command as Cmd;
    use tempfile::TempDir;

    fn git_init() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        Cmd::new("git").args(["init", "-q"]).current_dir(p).status().unwrap();
        Cmd::new("git").args(["config", "user.email", "t@t.com"]).current_dir(p).status().unwrap();
        Cmd::new("git").args(["config", "user.name", "test"]).current_dir(p).status().unwrap();
        dir
    }

    fn git_cmd(dir: &Path, args: &[&str]) {
        Cmd::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "t@t.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "t@t.com")
            .status()
            .unwrap();
    }

    fn make_commit(dir: &Path, filename: &str, content: &str) {
        std::fs::write(dir.join(filename), content).unwrap();
        git_cmd(dir, &["add", filename]);
        git_cmd(dir, &["commit", "-m", "init"]);
    }

    #[test]
    fn is_worktree_dirty_clean() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        assert!(!is_worktree_dirty(dir.path()));
    }

    #[test]
    fn is_worktree_dirty_dirty() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        std::fs::write(dir.path().join("f.txt"), "changed").unwrap();
        assert!(is_worktree_dirty(dir.path()));
    }

    #[test]
    fn local_branch_exists_present_and_absent() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        let on_main = local_branch_exists(dir.path(), "main");
        let on_master = local_branch_exists(dir.path(), "master");
        assert!(on_main || on_master);
        assert!(!local_branch_exists(dir.path(), "no-such-branch"));
    }

    #[test]
    fn delete_local_branch_success() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        git_cmd(dir.path(), &["branch", "to-delete"]);
        let mut warnings = Vec::new();
        delete_local_branch(dir.path(), "to-delete", &mut warnings);
        assert!(warnings.is_empty());
        assert!(!local_branch_exists(dir.path(), "to-delete"));
    }

    #[test]
    fn delete_local_branch_failure_adds_warning() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        let mut warnings = Vec::new();
        delete_local_branch(dir.path(), "nonexistent", &mut warnings);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("warning:"));
    }

    #[test]
    fn prune_remote_tracking_no_panic() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        // Just verify it doesn't panic even when the remote ref doesn't exist.
        prune_remote_tracking(dir.path(), "nonexistent-branch");
    }

    #[test]
    fn stage_files_ok_and_err() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        std::fs::write(dir.path().join("new.txt"), "new").unwrap();
        assert!(stage_files(dir.path(), &["new.txt"]).is_ok());
        assert!(stage_files(dir.path(), &["missing.txt"]).is_err());
    }

    #[test]
    fn commit_ok_and_err() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        std::fs::write(dir.path().join("new.txt"), "new").unwrap();
        git_cmd(dir.path(), &["add", "new.txt"]);
        assert!(commit(dir.path(), "test commit").is_ok());
        // Nothing staged — should fail
        assert!(commit(dir.path(), "empty commit").is_err());
    }

    #[test]
    fn git_config_get_some_and_none() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        let val = git_config_get(dir.path(), "user.email");
        assert_eq!(val, Some("t@t.com".to_string()));
        let missing = git_config_get(dir.path(), "no.such.key");
        assert!(missing.is_none());
    }

    #[test]
    fn merge_ref_already_up_to_date() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        let branch = {
            let out = Cmd::new("git").args(["branch", "--show-current"]).current_dir(dir.path()).output().unwrap();
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        let mut warnings = Vec::new();
        // Merging current branch into itself is already up to date
        let result = merge_ref(dir.path(), &branch, &mut warnings);
        assert!(result.is_none());
        assert!(warnings.is_empty());
    }

    #[test]
    fn merge_ref_success() {
        let dir = git_init();
        make_commit(dir.path(), "f.txt", "hi");
        git_cmd(dir.path(), &["checkout", "-b", "feature"]);
        make_commit(dir.path(), "g.txt", "there");
        git_cmd(dir.path(), &["checkout", "main"]);
        let mut warnings = Vec::new();
        let result = merge_ref(dir.path(), "feature", &mut warnings);
        assert!(result.is_some());
        assert!(warnings.is_empty());
    }

    #[test]
    fn is_file_tracked_tracked_and_untracked() {
        let dir = git_init();
        make_commit(dir.path(), "tracked.txt", "hi");
        assert!(is_file_tracked(dir.path(), "tracked.txt"));
        std::fs::write(dir.path().join("untracked.txt"), "new").unwrap();
        assert!(!is_file_tracked(dir.path(), "untracked.txt"));
    }
}
