use crate::{config::Config, git, git_util, ticket, ticket_fmt, worktree};
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use std::path::{Path, PathBuf};

const KNOWN_TEMP_FILES: &[&str] = &[
    "pr-body.md",
    "body.md",
    "ac.txt",
    ".apm-worker.pid",
    ".apm-worker.log",
];

pub struct RemoteCandidate {
    pub branch: String,
    pub last_commit: DateTime<Utc>,
}

pub struct CleanCandidate {
    pub ticket_id: String,
    pub ticket_title: String,
    pub branch: String,
    pub worktree: Option<PathBuf>,
    pub reason: String,
    pub local_branch_exists: bool,
    pub branch_merged: bool,
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
    agent_dirs: &[String],
) -> Result<DirtyWorktree> {
    let stdout = git_util::run(path, &["status", "--porcelain"])?;

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
        let top_dir = file.split('/').next().unwrap_or("");

        if xy == "??" {
            if KNOWN_TEMP_FILES.contains(&filename.as_str())
                || agent_dirs.iter().any(|d| d.trim_end_matches('/') == top_dir)
            {
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
        if full_path.is_dir() {
            std::fs::remove_dir_all(&full_path)?;
        } else if full_path.exists() {
            std::fs::remove_file(&full_path)?;
        }
    }
    Ok(())
}

pub struct RemoveOutput {
    pub warnings: Vec<String>,
}

pub fn candidates(root: &Path, config: &Config, force: bool, untracked: bool, dry_run: bool) -> Result<(Vec<CleanCandidate>, Vec<DirtyWorktree>, Vec<String>)> {
    let mut warnings: Vec<String> = Vec::new();
    let terminal_states = config.terminal_state_ids();

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
            .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{}", t.frontmatter.id));

        let id = t.frontmatter.id.clone();
        let branch_state = &t.frontmatter.state;

        let is_merged = merged_set.contains(branch.as_str());

        let local_tip = git::branch_tip(root, &branch);
        let is_ancestor = if let Some(ref tip) = local_tip {
            git::is_ancestor(root, tip, default_branch)
        } else {
            true
        };

        // The branch state is authoritative — we already filtered to terminal
        // states above. If the branch says the ticket is done, clean proceeds
        // regardless of what main says (or whether the ticket exists on main).

        let wt_path = worktree::find_worktree_for_branch(root, &branch);

        // Check worktree cleanliness before the tip-divergence guard so that
        // a clean worktree on a closed ticket is not blocked by stale refs.
        let wt_clean = if let Some(ref path) = wt_path {
            !git_util::is_worktree_dirty(path)
        } else {
            true
        };

        if !force {
            let remote_tip = git::remote_branch_tip(root, &branch);
            if let (Some(ref lt), Some(ref rt)) = (&local_tip, &remote_tip) {
                if lt != rt && !wt_clean {
                    warnings.push(format!(
                        "warning: {branch} local tip differs from origin/{branch} — skipping"
                    ));
                    continue;
                }
            }
        }

        if let Some(ref path) = wt_path {
            if !wt_clean {
                let lbe = git_util::local_branch_exists(root, &branch);
                let diagnosis =
                    diagnose_worktree(path, &id, &t.frontmatter.title, &branch, lbe, &config.worktrees.agent_dirs)?;
                if diagnosis.modified_tracked.is_empty() {
                    if force {
                        // Force mode: git worktree remove --force handles remaining files.
                        result.push(CleanCandidate {
                            ticket_id: id,
                            ticket_title: t.frontmatter.title.clone(),
                            branch: branch.clone(),
                            worktree: wt_path,
                            reason: branch_state.clone(),
                            local_branch_exists: lbe,
                            branch_merged: is_merged && is_ancestor,
                        });
                    } else if untracked || diagnosis.other_untracked.is_empty() {
                        // Auto-remove: known_temp always; other_untracked if --untracked.
                        // Skip actual file removal in dry-run mode.
                        if !dry_run {
                            remove_untracked(path, &diagnosis.known_temp)?;
                            if untracked {
                                remove_untracked(path, &diagnosis.other_untracked)?;
                            }
                        }
                        result.push(CleanCandidate {
                            ticket_id: id,
                            ticket_title: t.frontmatter.title.clone(),
                            branch: branch.clone(),
                            worktree: wt_path,
                            reason: branch_state.clone(),
                            local_branch_exists: lbe,
                            branch_merged: is_merged && is_ancestor,
                        });
                    } else {
                        dirty_result.push(diagnosis);
                    }
                } else {
                    dirty_result.push(diagnosis);
                }
                continue;
            }
        }

        let local_branch_exists = git_util::local_branch_exists(root, &branch);

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
            branch_merged: is_merged && is_ancestor,
        });
    }

    Ok((result, dirty_result, warnings))
}

pub fn remove(root: &Path, candidate: &CleanCandidate, force: bool, remove_branches: bool) -> Result<RemoveOutput> {
    let mut warnings: Vec<String> = Vec::new();

    if let Some(ref path) = candidate.worktree {
        worktree::remove_worktree(root, path, force)?;
    }

    if remove_branches && candidate.local_branch_exists && (candidate.branch_merged || force) {
        git_util::delete_local_branch(root, &candidate.branch, &mut warnings);
        // Prune the remote tracking ref so sync_local_ticket_refs does not
        // recreate the local branch on the next apm sync.
        git_util::prune_remote_tracking(root, &candidate.branch);
    }

    Ok(RemoveOutput { warnings })
}

/// Parse an --older-than threshold into a UTC DateTime.
/// Accepts "Nd" (N days ago) or "YYYY-MM-DD" (ISO date).
pub fn parse_older_than(s: &str) -> anyhow::Result<DateTime<Utc>> {
    if let Some(days_str) = s.strip_suffix('d') {
        let days: i64 = days_str
            .parse()
            .map_err(|_| anyhow::anyhow!("--older-than: invalid days value {:?}", s))?;
        return Ok(Utc::now() - chrono::Duration::days(days));
    }
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }
    anyhow::bail!(
        "--older-than: unrecognised format {:?}; use \"30d\" or \"YYYY-MM-DD\"",
        s
    )
}

/// Return remote ticket/* branches in terminal states older than `older_than`.
pub fn remote_candidates(
    root: &Path,
    config: &Config,
    older_than: DateTime<Utc>,
) -> Result<Vec<RemoteCandidate>> {
    let terminal_states = config.terminal_state_ids();
    let default_branch = &config.project.default_branch;
    let branches = git::remote_ticket_branches_with_dates(root)?;
    let mut result = Vec::new();
    for (branch, last_commit) in branches {
        if last_commit >= older_than {
            continue;
        }
        let suffix = branch.trim_start_matches("ticket/");
        let rel_path = format!("{}/{suffix}.md", config.tickets.dir.to_string_lossy());
        if let Some(state) = ticket::state_from_branch(root, default_branch, &rel_path) {
            if terminal_states.contains(&state) {
                result.push(RemoteCandidate { branch, last_commit });
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_older_than_days() {
        let threshold = parse_older_than("30d").unwrap();
        let expected = Utc::now() - chrono::Duration::days(30);
        // Allow a few seconds of skew between the two Utc::now() calls.
        assert!((threshold - expected).num_seconds().abs() < 5);
    }

    #[test]
    fn parse_older_than_iso_date() {
        let threshold = parse_older_than("2026-01-01").unwrap();
        assert_eq!(threshold.format("%Y-%m-%d").to_string(), "2026-01-01");
    }

    #[test]
    fn parse_older_than_invalid_rejects() {
        assert!(parse_older_than("notadate").is_err());
        assert!(parse_older_than("30").is_err());
        assert!(parse_older_than("").is_err());
    }

    #[test]
    fn parse_older_than_zero_days() {
        let threshold = parse_older_than("0d").unwrap();
        let now = Utc::now();
        assert!((threshold - now).num_seconds().abs() < 5);
    }
}
