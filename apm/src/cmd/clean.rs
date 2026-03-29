use anyhow::Result;
use apm_core::{config::Config, git, ticket};
use std::path::Path;
use std::process::Command;

pub fn run(root: &Path, dry_run: bool) -> Result<()> {
    let config = Config::load(root)?;

    let terminal_states: Vec<&str> = config
        .workflow
        .states
        .iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();

    let default_branch = &config.project.default_branch;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let merged = git::merged_into_main(root, default_branch)?;
    let merged_set: std::collections::HashSet<&str> =
        merged.iter().map(|s| s.as_str()).collect();

    let mut did_anything = false;

    for t in &tickets {
        if !terminal_states.contains(&t.frontmatter.state.as_str()) {
            continue;
        }

        let branch = t
            .frontmatter
            .branch
            .clone()
            .or_else(|| git::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{:04}", t.frontmatter.id));

        if !merged_set.contains(branch.as_str()) {
            eprintln!("warning: {branch} not merged — skipping");
            continue;
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

        if dry_run {
            if let Some(ref path) = wt_path {
                println!("would remove worktree {}", path.display());
            }
            println!("would remove branch {branch}");
        } else {
            if let Some(ref path) = wt_path {
                git::remove_worktree(root, path)?;
                println!("removed worktree {}", path.display());
            }

            let result = Command::new("git")
                .args(["-C", &root.to_string_lossy(), "branch", "-d", &branch])
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

        did_anything = true;
    }

    if !did_anything {
        println!("Nothing to clean.");
    }

    Ok(())
}
