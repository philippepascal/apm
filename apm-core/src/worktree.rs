use anyhow::Result;
use std::path::{Path, PathBuf};
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
    crate::git_util::is_file_tracked(root, path)
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
    let main_root = crate::git_util::main_worktree_root(root).unwrap_or_else(|| root.to_path_buf());
    let worktrees_base = main_root.join(&config.worktrees.dir);
    let wt = ensure_worktree(root, &worktrees_base, branch)?;
    sync_agent_dirs(root, &wt, &config.worktrees.agent_dirs, warnings);
    Ok(wt)
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use tempfile::TempDir;

    fn git_init(dir: &std::path::Path) {
        Command::new("git").args(["init", "-b", "main"]).current_dir(dir).output().unwrap();
        Command::new("git").args(["config", "user.email", "t@t.com"]).current_dir(dir).output().unwrap();
        Command::new("git").args(["config", "user.name", "test"]).current_dir(dir).output().unwrap();
    }

    #[test]
    fn provision_worktree_creates_dir_inside_repo() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        git_init(repo);
        std::fs::write(repo.join("README"), "x").unwrap();
        Command::new("git").args(["-c", "commit.gpgsign=false", "add", "README"]).current_dir(repo).output().unwrap();
        Command::new("git").args(["-c", "commit.gpgsign=false", "commit", "-m", "init"]).current_dir(repo).output().unwrap();
        Command::new("git").args(["branch", "ticket/test-branch"]).current_dir(repo).output().unwrap();

        let toml = r#"[project]
name = "test"

[tickets]
dir = "tickets"

[worktrees]
dir = "worktrees"
"#;
        let config: crate::config::Config = toml::from_str(toml).unwrap();

        let mut warnings: Vec<String> = Vec::new();
        let wt = super::provision_worktree(repo, &config, "ticket/test-branch", &mut warnings).unwrap();

        let main_root = crate::git_util::main_worktree_root(repo)
            .unwrap_or_else(|| repo.to_path_buf());
        let expected = main_root.join("worktrees").join("ticket-test-branch");
        assert_eq!(wt, expected, "provisioned path must be <repo>/worktrees/<branch-slug>");
        assert!(wt.is_dir(), "provisioned worktree dir must exist on disk: {}", wt.display());
        assert!(
            wt.starts_with(&main_root),
            "worktree path must be inside repo: wt={} repo={}",
            wt.display(),
            main_root.display()
        );
    }

    #[test]
    fn provision_worktree_honours_external_layout() {
        // Existing repos with `dir = "../<name>--worktrees"` must keep working
        // — the external layout is still supported, just no longer the default.
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        git_init(&repo);
        std::fs::write(repo.join("README"), "x").unwrap();
        Command::new("git").args(["-c", "commit.gpgsign=false", "add", "README"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["-c", "commit.gpgsign=false", "commit", "-m", "init"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["branch", "ticket/ext-branch"]).current_dir(&repo).output().unwrap();

        let toml = r#"[project]
name = "test"

[tickets]
dir = "tickets"

[worktrees]
dir = "../external-worktrees"
"#;
        let config: crate::config::Config = toml::from_str(toml).unwrap();

        let mut warnings: Vec<String> = Vec::new();
        let wt = super::provision_worktree(&repo, &config, "ticket/ext-branch", &mut warnings).unwrap();

        let expected = tmp.path().join("external-worktrees").join("ticket-ext-branch");
        assert_eq!(
            wt.canonicalize().unwrap(),
            expected.canonicalize().unwrap(),
            "external layout must place worktree as a sibling of the repo"
        );
        assert!(wt.is_dir(), "external worktree dir must exist on disk: {}", wt.display());
    }
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
