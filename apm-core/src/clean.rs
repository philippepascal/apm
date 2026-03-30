use crate::{config::Config, git, ticket};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

const KNOWN_TEMP_FILES: &[&str] = &[
    "pr-body.md",
    "body.md",
    "ac.txt",
    ".apm-worker.pid",
    ".apm-worker.log",
];

pub struct CleanCandidate {
    pub ticket_id: String,
    pub ticket_title: String,
    pub branch: String,
    pub worktree: Option<PathBuf>,
    pub reason: String,
    pub local_branch_exists: bool,
}

pub struct DirtyWorktree {
    pub ticket_id: String,
    pub ticket_title: String,
    pub branch: String,
    pub path: PathBuf,
    pub local_branch_exists: bool,
    pub known_temp: Vec<PathBuf>,
    pub other_untracked: Vec<PathBuf>,
    pub modified_tracked: Vec<PathBuf>,
}

pub fn diagnose_worktree(
    path: &Path,
    ticket_id: &str,
    ticket_title: &str,
    branch: &str,
    local_branch_exists: bool,
) -> Result<DirtyWorktree> {
    let out = Command::new("git")
        .args(["-C", &path.to_string_lossy(), "status", "--porcelain"])
        .output()?;
    let stdout = String::from_utf8_lossy(&out.stdout);

    let mut known_temp = Vec::new();
    let mut other_untracked = Vec::new();
    let mut modified_tracked = Vec::new();

    for line in stdout.lines() {
        if line.len() < 3 {
            continue;
        }
        let xy = &line[..2];
        let file = line[3..].trim();
        let filename = std::path::Path::new(file)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        if xy == "??" {
            if KNOWN_TEMP_FILES.contains(&filename.as_str()) {
                known_temp.push(PathBuf::from(file));
            } else {
                other_untracked.push(PathBuf::from(file));
            }
        } else {
            modified_tracked.push(PathBuf::from(file));
        }
    }

    Ok(DirtyWorktree {
        ticket_id: ticket_id.to_string(),
        ticket_title: ticket_title.to_string(),
        branch: branch.to_string(),
        path: path.to_path_buf(),
        local_branch_exists,
        known_temp,
        other_untracked,
        modified_tracked,
    })
}

pub fn remove_untracked(wt_path: &Path, files: &[PathBuf]) -> Result<()> {
    for file in files {
        let full_path = wt_path.join(file);
        if full_path.exists() {
            std::fs::remove_file(&full_path)?;
        }
    }
    Ok(())
}

pub fn candidates(root: &Path, config: &Config) -> Result<(Vec<CleanCandidate>, Vec<DirtyWorktree>)> {
    let mut terminal_states: std::collections::HashSet<String> = config
        .workflow
        .states
        .iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.clone())
        .collect();
    terminal_states.insert("closed".to_string());

    let default_branch = &config.project.default_branch;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let merged = git::merged_into_main(root, default_branch)?;
    let merged_set: std::collections::HashSet<&str> = merged.iter().map(|s| s.as_str()).collect();

    let mut result = Vec::new();
    let mut dirty_result = Vec::new();

    for t in &tickets {
        if !terminal_states.contains(t.frontmatter.state.as_str()) {
            continue;
        }

        let branch = t
            .frontmatter
            .branch
            .clone()
            .or_else(|| git::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{}", t.frontmatter.id));

        let id = t.frontmatter.id.clone();
        let branch_state = &t.frontmatter.state;

        if !merged_set.contains(branch.as_str()) {
            eprintln!("warning: {branch} not merged — skipping");
            continue;
        }

        let local_tip = git::branch_tip(root, &branch);
        if let Some(ref tip) = local_tip {
            if !git::is_ancestor(root, tip, default_branch) {
                eprintln!(
                    "warning: {branch} tip is not a git ancestor of {default_branch} — skipping"
                );
                continue;
            }
        }

        let suffix = branch.trim_start_matches("ticket/");
        let rel_path = format!("{}/{suffix}.md", config.tickets.dir.to_string_lossy());
        let main_state = ticket::state_from_branch(root, default_branch, &rel_path);
        match &main_state {
            Some(ms) if ms != branch_state => {
                eprintln!(
                    "warning: {branch} state mismatch — branch={branch_state} \
                     main={ms} — run `apm close {id}` to reconcile"
                );
                continue;
            }
            None => {
                eprintln!(
                    "warning: {branch} not found on {default_branch} — \
                     run `apm close {id}` to reconcile"
                );
                continue;
            }
            _ => {}
        }

        let remote_tip = git::remote_branch_tip(root, &branch);
        if let (Some(ref lt), Some(ref rt)) = (&local_tip, &remote_tip) {
            if lt != rt {
                eprintln!(
                    "warning: {branch} local tip differs from origin/{branch} — skipping"
                );
                continue;
            }
        }

        let wt_path = git::find_worktree_for_branch(root, &branch);

        if let Some(ref path) = wt_path {
            let out = Command::new("git")
                .args(["-C", &path.to_string_lossy(), "status", "--porcelain"])
                .output();
            let dirty = match out {
                Ok(ref o) => !o.stdout.is_empty(),
                Err(_) => false,
            };
            if dirty {
                let lbe = Command::new("git")
                    .args([
                        "-C",
                        &root.to_string_lossy(),
                        "rev-parse",
                        "--verify",
                        &format!("refs/heads/{branch}"),
                    ])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                let diagnosis =
                    diagnose_worktree(path, &id, &t.frontmatter.title, &branch, lbe)?;
                dirty_result.push(diagnosis);
                continue;
            }
        }

        let local_branch_exists = Command::new("git")
            .args([
                "-C",
                &root.to_string_lossy(),
                "rev-parse",
                "--verify",
                &format!("refs/heads/{branch}"),
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if wt_path.is_none() && !local_branch_exists {
            continue;
        }

        result.push(CleanCandidate {
            ticket_id: id,
            ticket_title: t.frontmatter.title.clone(),
            branch: branch.clone(),
            worktree: wt_path,
            reason: branch_state.clone(),
            local_branch_exists,
        });
    }

    Ok((result, dirty_result))
}

pub fn remove(root: &Path, candidate: &CleanCandidate) -> Result<()> {
    if let Some(ref path) = candidate.worktree {
        git::remove_worktree(root, path)?;
    }

    if candidate.local_branch_exists {
        let result = Command::new("git")
            .args([
                "-C",
                &root.to_string_lossy(),
                "branch",
                "-D",
                &candidate.branch,
            ])
            .output();
        match result {
            Ok(o) if o.status.success() => {}
            Ok(o) => {
                let msg = String::from_utf8_lossy(&o.stderr);
                eprintln!(
                    "warning: could not delete branch {}: {}",
                    candidate.branch,
                    msg.trim()
                );
            }
            Err(e) => {
                eprintln!(
                    "warning: could not delete branch {}: {e}",
                    candidate.branch
                );
            }
        }
    }

    Ok(())
}
