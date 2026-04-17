use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::{AppError, AppState};
use crate::models::CleanRequest;

pub async fn sync_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let (fetch_error, branches, closed) = tokio::task::spawn_blocking(move || {
        let fetch_error = apm_core::git::fetch_all(&root).err().map(|e| e.to_string());
        let mut _sync_warnings: Vec<String> = Vec::new();
        apm_core::git::sync_non_checked_out_refs(&root, &mut _sync_warnings);
        let branches = apm_core::git::ticket_branches(&root)
            .map(|b| b.len())
            .unwrap_or(0);
        let closed = match apm_core::config::Config::load(&root) {
            Ok(config) => {
                let mut sync_warnings: Vec<String> = Vec::new();
                apm_core::git::sync_default_branch(&root, &config.project.default_branch, &mut sync_warnings);
                for w in &sync_warnings {
                    eprintln!("warning: {w}");
                }
                match apm_core::sync::detect(&root, &config) {
                    Ok(candidates) => {
                        let n = candidates.close.len();
                        if n > 0 {
                            let aggressive = config.sync.aggressive;
                            let author = apm_core::config::resolve_identity(&root);
                            let _ = apm_core::sync::apply(&root, &config, &candidates, &author, aggressive);
                        }
                        n
                    }
                    Err(_) => 0,
                }
            }
            Err(_) => 0,
        };
        (fetch_error, branches, closed)
    })
    .await?;
    let mut resp = serde_json::json!({ "branches": branches, "closed": closed });
    if let Some(err) = fetch_error {
        resp["fetch_error"] = serde_json::Value::String(err);
    }
    Ok(Json(resp).into_response())
}

pub async fn clean_handler(
    State(state): State<Arc<AppState>>,
    body: Option<Json<CleanRequest>>,
) -> Result<Response, AppError> {
    let root = match state.git_root() {
        Some(r) => r.clone(),
        None => return Ok((StatusCode::NOT_IMPLEMENTED, "no git root").into_response()),
    };
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let dry_run   = req.dry_run.unwrap_or(false);
    let force     = req.force.unwrap_or(false);
    let branches  = req.branches.unwrap_or(false);
    let remote    = req.remote.unwrap_or(false);
    let untracked = req.untracked.unwrap_or(false);
    let epics     = req.epics.unwrap_or(false);
    let older_than = req.older_than;

    if remote && older_than.is_none() {
        return Ok((StatusCode::BAD_REQUEST, "remote requires older_than").into_response());
    }

    let (log, removed) = crate::util::blocking(move || -> anyhow::Result<(Vec<String>, usize)> {
        let mut log: Vec<String> = Vec::new();
        let mut count = 0usize;

        let config = apm_core::config::Config::load(&root)?;
        let (candidates, dirty, candidate_warnings) =
            apm_core::clean::candidates(&root, &config, force, untracked, dry_run)?;
        for w in &candidate_warnings {
            log.push(w.clone());
        }

        for dw in &dirty {
            if !dw.modified_tracked.is_empty() {
                for f in &dw.modified_tracked {
                    log.push(format!("  M {}", f.display()));
                }
                log.push(format!(
                    "warning: {} has modified tracked files — manual cleanup required — skipping",
                    dw.branch
                ));
            } else {
                for f in &dw.other_untracked {
                    log.push(format!("  ? {}", f.display()));
                }
                log.push(format!(
                    "warning: {} has untracked files — re-run with --untracked to remove — skipping",
                    dw.branch
                ));
            }
        }

        for candidate in &candidates {
            if dry_run {
                if let Some(ref path) = candidate.worktree {
                    log.push(format!(
                        "would remove worktree {} (ticket #{}, state: {})",
                        path.display(),
                        candidate.ticket_id,
                        candidate.reason
                    ));
                }
                if branches && candidate.local_branch_exists && (candidate.branch_merged || force) {
                    log.push(format!(
                        "would remove branch {} (state: {})",
                        candidate.branch, candidate.reason
                    ));
                } else if branches && candidate.local_branch_exists && !candidate.branch_merged {
                    log.push(format!(
                        "would keep branch {} (not merged into main)",
                        candidate.branch
                    ));
                }
            } else if force {
                log.push(format!(
                    "warning: force-removing {} — branch may not be merged",
                    candidate.branch
                ));
                let remove_out = apm_core::clean::remove(&root, candidate, true, branches)?;
                if let Some(ref path) = candidate.worktree {
                    log.push(format!("removed worktree {}", path.display()));
                    count += 1;
                }
                if branches && candidate.local_branch_exists {
                    log.push(format!("removed branch {}", candidate.branch));
                }
                for w in &remove_out.warnings {
                    log.push(w.clone());
                }
            } else {
                let remove_out = apm_core::clean::remove(&root, candidate, false, branches)?;
                if let Some(ref path) = candidate.worktree {
                    log.push(format!("removed worktree {}", path.display()));
                    count += 1;
                }
                if branches && candidate.local_branch_exists && candidate.branch_merged {
                    log.push(format!("removed branch {}", candidate.branch));
                } else if branches && candidate.local_branch_exists && !candidate.branch_merged {
                    log.push(format!("kept branch {} (not merged into main)", candidate.branch));
                }
                for w in &remove_out.warnings {
                    log.push(w.clone());
                }
            }
        }

        if remote {
            let threshold_str = older_than.as_deref().unwrap();
            let threshold = apm_core::clean::parse_older_than(threshold_str)?;
            let remote_candidates = apm_core::clean::remote_candidates(&root, &config, threshold)?;
            if remote_candidates.is_empty() {
                log.push("No remote branches to clean.".to_string());
            }
            for rc in &remote_candidates {
                if dry_run {
                    log.push(format!(
                        "would delete remote branch {} (last commit: {})",
                        rc.branch,
                        rc.last_commit.format("%Y-%m-%d")
                    ));
                } else {
                    apm_core::git::delete_remote_branch(&root, &rc.branch)?;
                    log.push(format!("deleted remote branch {}", rc.branch));
                }
            }
        }

        if epics {
            let local_output = std::process::Command::new("git")
                .current_dir(&root)
                .args(["branch", "--list", "epic/*"])
                .output()?;
            let local_branches: Vec<String> = String::from_utf8_lossy(&local_output.stdout)
                .lines()
                .map(|l| l.trim().trim_start_matches(['*', '+']).trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();

            let tickets = apm_core::ticket::load_all_from_git(&root, &config.tickets.dir)?;

            let mut epic_candidates: Vec<String> = Vec::new();
            for branch in &local_branches {
                let after_prefix = branch.trim_start_matches("epic/");
                let id_end = after_prefix.find('-').unwrap_or(after_prefix.len()).min(8);
                let id = &after_prefix[..id_end];
                let epic_tickets: Vec<_> = tickets
                    .iter()
                    .filter(|t| t.frontmatter.epic.as_deref() == Some(id))
                    .collect();
                let state_configs: Vec<&apm_core::config::StateConfig> = epic_tickets
                    .iter()
                    .filter_map(|t| config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
                    .collect();
                if apm_core::epic::derive_epic_state(&state_configs) == "done" {
                    epic_candidates.push(branch.clone());
                }
            }

            if epic_candidates.is_empty() {
                log.push("No done epics to clean.".to_string());
            }

            for branch in &epic_candidates {
                let after_prefix = branch.trim_start_matches("epic/");
                let id_end = after_prefix.find('-').unwrap_or(after_prefix.len()).min(8);
                let id = after_prefix[..id_end].to_string();

                if dry_run {
                    log.push(format!("would delete epic branch {branch}"));
                    continue;
                }

                let del_local = std::process::Command::new("git")
                    .current_dir(&root)
                    .args(["branch", "-d", branch])
                    .output()?;
                if !del_local.status.success() {
                    log.push(format!(
                        "error: failed to delete local branch {branch}: {}",
                        String::from_utf8_lossy(&del_local.stderr).trim()
                    ));
                    continue;
                }

                let del_remote = std::process::Command::new("git")
                    .current_dir(&root)
                    .args(["push", "origin", "--delete", branch])
                    .output()?;
                if !del_remote.status.success() {
                    let stderr = String::from_utf8_lossy(&del_remote.stderr);
                    if !stderr.contains("remote ref does not exist")
                        && !stderr.contains("error: unable to delete")
                    {
                        log.push(format!("warning: failed to delete remote {branch}: {}", stderr.trim()));
                    }
                }

                log.push(format!("deleted epic {branch}"));

                let epics_path = root.join(".apm").join("epics.toml");
                if epics_path.exists() {
                    let raw = std::fs::read_to_string(&epics_path)?;
                    let mut table: toml::value::Table = toml::from_str(&raw)?;
                    if table.remove(&id).is_some() {
                        std::fs::write(&epics_path, toml::to_string(&table)?)?;
                    }
                }
            }
        }

        Ok((log, count))
    }).await?;

    Ok(Json(serde_json::json!({ "log": log.join("\n"), "removed": removed })).into_response())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn sync_in_memory_returns_not_implemented() {
        let app = crate::build_app_with_tickets(crate::tests::test_tickets());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/sync")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }
}
