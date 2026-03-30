use anyhow::Result;
use apm_core::{config::Config, git, ticket};
use std::path::Path;
use std::process::Command;

pub fn run(root: &Path, dry_run: bool) -> Result<()> {
    let config = Config::load(root)?;

    // "closed" is always terminal, regardless of what apm.toml says.
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
    let merged_set: std::collections::HashSet<&str> =
        merged.iter().map(|s| s.as_str()).collect();

    let mut did_anything = false;

    for t in &tickets {
        if !terminal_states.contains(t.frontmatter.state.as_str()) {
            continue;
        }

        let branch = t
            .frontmatter
            .branch
            .clone()
            .or_else(|| git::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{:04}", t.frontmatter.id));

        let id = t.frontmatter.id;
        let branch_state = &t.frontmatter.state;

        if !merged_set.contains(branch.as_str()) {
            eprintln!("warning: {branch} not merged — skipping");
            continue;
        }

        // Ancestor check: branch tip must be a git ancestor of the default branch.
        let local_tip = git::branch_tip(root, &branch);
        if let Some(ref tip) = local_tip {
            if !git::is_ancestor(root, tip, default_branch) {
                eprintln!(
                    "warning: {branch} tip is not a git ancestor of {default_branch} — skipping"
                );
                continue;
            }
        }

        // Cross-check: ticket state on the default branch must also be terminal.
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
            _ => {} // states agree
        }

        // Local vs remote tip agreement: warn and skip if they diverge.
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

        // Nothing to do locally — branch and worktree are already gone.
        if wt_path.is_none() && !local_branch_exists {
            continue;
        }

        if dry_run {
            if let Some(ref path) = wt_path {
                println!(
                    "would remove worktree {} (ticket #{id}, state: {branch_state})",
                    path.display()
                );
            }
            if local_branch_exists {
                println!("would remove branch {branch} (state: {branch_state})");
            }
        } else {
            if let Some(ref path) = wt_path {
                git::remove_worktree(root, path)?;
                println!("removed worktree {}", path.display());
            }

            if local_branch_exists {
                let result = Command::new("git")
                    .args(["-C", &root.to_string_lossy(), "branch", "-D", &branch])
                    .output();
                match result {
                    Ok(o) if o.status.success() => {
                        println!("removed branch {branch}");
                    }
                    Ok(o) => {
                        let msg = String::from_utf8_lossy(&o.stderr);
                        eprintln!("warning: could not delete branch {branch}: {}", msg.trim());
                    }
                    Err(e) => {
                        eprintln!("warning: could not delete branch {branch}: {e}");
                    }
                }
            }
        }

        did_anything = true;
    }

    if !did_anything {
        println!("Nothing to clean.");
    }

    Ok(())
}
