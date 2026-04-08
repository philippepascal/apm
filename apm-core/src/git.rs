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

/// Find a branch named `epic/<short_id>-*` locally or remotely.
/// Returns the branch name (without `origin/` prefix) if found.
pub fn find_epic_branch(root: &Path, short_id: &str) -> Option<String> {
    let pattern = format!("epic/{short_id}-*");
    let local = run(root, &["branch", "--list", &pattern]).ok()?;
    for b in local.lines().map(|l| l.trim().trim_start_matches(['*', '+']).trim()) {
        if !b.is_empty() {
            return Some(b.to_string());
        }
    }
    let remote_pattern = format!("origin/epic/{short_id}-*");
    let remote = run(root, &["branch", "-r", "--list", &remote_pattern]).ok()?;
    for b in remote.lines().map(|l| l.trim()) {
        if !b.is_empty() {
            return Some(b.trim_start_matches("origin/").to_string());
        }
    }
    None
}

/// Return all epic branches (local + remote, deduplicated) whose 8-char ID
/// segment starts with `id_prefix`.  The returned names are short
/// (`epic/<id>-<slug>`, no `origin/` prefix).
pub fn find_epic_branches(root: &Path, id_prefix: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    let local = run(root, &["branch", "--list", "epic/*"]).unwrap_or_default();
    for b in local.lines()
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim())
        .filter(|l| !l.is_empty())
    {
        // branch format: epic/<8-char-id>-<slug>
        let id_part = b.trim_start_matches("epic/").split('-').next().unwrap_or("");
        if id_part.starts_with(id_prefix) && seen.insert(b.to_string()) {
            result.push(b.to_string());
        }
    }

    let remote = run(root, &["branch", "-r", "--list", "origin/epic/*"]).unwrap_or_default();
    for b in remote.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        let short = b.trim_start_matches("origin/");
        let id_part = short.trim_start_matches("epic/").split('-').next().unwrap_or("");
        if id_part.starts_with(id_prefix) && seen.insert(short.to_string()) {
            result.push(short.to_string());
        }
    }

    result
}

/// All epic/* branch names visible locally or remotely (deduplicated), sorted.
pub fn epic_branches(root: &Path) -> Result<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    let mut branches = Vec::new();

    let local = run(root, &["branch", "--list", "epic/*"]).unwrap_or_default();
    for b in local.lines()
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim())
        .filter(|l| !l.is_empty())
    {
        if seen.insert(b.to_string()) {
            branches.push(b.to_string());
        }
    }

    let remote = run(root, &["branch", "-r", "--list", "origin/epic/*"]).unwrap_or_default();
    for b in remote.lines()
        .map(|l| l.trim().trim_start_matches("origin/").to_string())
        .filter(|l| !l.is_empty())
    {
        if seen.insert(b.clone()) {
            branches.push(b);
        }
    }

    branches.sort();
    Ok(branches)
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
    let config = match crate::config::Config::load(root) {
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
    let prefixes = crate::ticket::id_arg_prefixes(arg)?;
    let mut seen = std::collections::HashSet::new();
    let matches: Vec<&String> = branches.iter()
        .filter(|b| {
            let id = b.strip_prefix("ticket/")
                .and_then(|s| s.split('-').next())
                .unwrap_or("");
            prefixes.iter().any(|p| id.starts_with(p.as_str())) && seen.insert(id.to_string())
        })
        .collect();
    match matches.len() {
        0 => anyhow::bail!("no ticket matches '{arg}'"),
        1 => Ok(matches[0].clone()),
        _ => {
            let mut msg = format!("error: prefix '{arg}' is ambiguous");
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

/// Create a new epic branch at origin/main (or local main as fallback),
/// seed it with a minimal EPIC.md, and push it.
/// Returns `(id, branch_name)` where branch_name is `epic/<id>-<slug>`.
pub fn create_epic_branch(root: &Path, title: &str) -> Result<(String, String)> {
    let id = gen_hex_id();
    let slug = crate::ticket::slugify(title);
    let branch = format!("epic/{id}-{slug}");
    let _ = run(root, &["fetch", "origin", "main"]);
    if run(root, &["branch", &branch, "origin/main"]).is_err() {
        run(root, &["branch", &branch, "main"])?;
    }
    commit_to_branch(root, &branch, "EPIC.md", &format!("# {title}\n"), "epic: init")?;
    let _ = push_branch(root, &branch);
    Ok((id, branch))
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
