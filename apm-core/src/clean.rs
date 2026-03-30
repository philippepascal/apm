use crate::{config::Config, git, ticket};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct CleanCandidate {
    pub ticket_id: String,
    pub ticket_title: String,
    pub branch: String,
    pub worktree: Option<PathBuf>,
    pub reason: String,
    pub local_branch_exists: bool,
}

pub fn candidates(root: &Path, config: &Config) -> Result<Vec<CleanCandidate>> {
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
                Ok(o) => !o.stdout.is_empty(),
                Err(_) => false,
            };
            if dirty {
                eprintln!(
                    "warning: {} has uncommitted changes — skipping",
                    path.display()
                );
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

    Ok(result)
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
