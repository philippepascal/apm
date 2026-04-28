use anyhow::Result;
use std::io::IsTerminal;
use std::path::Path;
use crate::ctx::CmdContext;
use apm_core::epic::{branch_to_title, epic_id_from_branch};

pub fn run_list(root: &Path) -> Result<()> {
    let ctx = CmdContext::load(root, false)?;

    let epic_branches = apm_core::epic::epic_branches(root)?;
    if epic_branches.is_empty() {
        return Ok(());
    }

    let tickets = ctx.tickets;

    for branch in &epic_branches {
        let id = epic_id_from_branch(branch);
        let title = branch_to_title(branch);

        // Find tickets belonging to this epic.
        let epic_tickets: Vec<_> = tickets
            .iter()
            .filter(|t| t.frontmatter.epic.as_deref() == Some(id))
            .collect();

        // Collect StateConfig references for each ticket (skip unknown states).
        let state_configs: Vec<&apm_core::config::StateConfig> = epic_tickets
            .iter()
            .filter_map(|t| ctx.config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
            .collect();

        let derived = apm_core::epic::derive_epic_state(&state_configs);

        // Build per-state counts (non-zero only).
        let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for t in &epic_tickets {
            *counts.entry(t.frontmatter.state.clone()).or_insert(0) += 1;
        }
        let counts_str: String = counts
            .iter()
            .filter(|(_, &v)| v > 0)
            .map(|(k, v)| format!("{v} {k}"))
            .collect::<Vec<_>>()
            .join(", ");

        println!("{id:<8} [{derived:<12}] {title:<40} {counts_str}");
    }

    Ok(())
}

pub fn run_new(root: &Path, title: String) -> Result<()> {
    let config = apm_core::config::Config::load(root)?;
    let branch = apm_core::epic::create(root, &title, &config)?;
    println!("{branch}");
    Ok(())
}

pub fn run_close(root: &Path, id_arg: &str) -> Result<()> {
    let config = CmdContext::load_config_only(root)?;

    // 1. Resolve the epic branch from the id prefix.
    let matches = apm_core::epic::find_epic_branches(root, id_arg);
    let epic_branch = match matches.len() {
        0 => anyhow::bail!("no epic branch found matching '{id_arg}'"),
        1 => matches.into_iter().next().unwrap(),
        _ => anyhow::bail!(
            "ambiguous id '{id_arg}': matches {}\n  {}",
            matches.len(),
            matches.join("\n  ")
        ),
    };

    // 2. Parse the 8-char epic ID from the branch name: epic/<id>-<slug>
    let epic_id = epic_id_from_branch(&epic_branch);

    // 3. Quiescence check: no ticket may be in an active state or have a live worker.
    let worktrees = apm_core::worktree::list_ticket_worktrees(root)?;
    let blockers = apm_core::epic::epic_is_quiescent(root, epic_id, &config, &worktrees)?;
    if !blockers.is_empty() {
        anyhow::bail!(
            "cannot close epic: the following tickets are not quiescent:\n{}",
            blockers.join("\n")
        );
    }

    // 4. Derive a human-readable title from the branch name.
    let pr_title = branch_to_title(&epic_branch);

    // 5. Push the epic branch and create or reuse an open PR.
    let default_branch = &config.project.default_branch;
    apm_core::git::push_branch_tracking(root, &epic_branch)?;
    let mut messages = vec![];
    apm_core::github::gh_pr_create_or_update(
        root,
        &epic_branch,
        default_branch,
        epic_id,
        &pr_title,
        &format!("Epic: {epic_branch}"),
        &mut messages,
    )?;
    for m in &messages {
        println!("{m}");
    }
    Ok(())
}

pub fn run_refresh_epic(root: &Path, id_arg: &str) -> Result<()> {
    let config = CmdContext::load_config_only(root)?;

    let matches = apm_core::epic::find_epic_branches(root, id_arg);
    let epic_branch = match matches.len() {
        0 => anyhow::bail!("no epic branch found matching '{id_arg}'"),
        1 => matches.into_iter().next().unwrap(),
        _ => anyhow::bail!(
            "ambiguous id '{id_arg}': matches {}\n  {}",
            matches.len(),
            matches.join("\n  ")
        ),
    };

    let epic_id = epic_id_from_branch(&epic_branch);

    let worktrees = apm_core::worktree::list_ticket_worktrees(root)?;
    let blockers = apm_core::epic::epic_is_quiescent(root, epic_id, &config, &worktrees)?;
    if !blockers.is_empty() {
        anyhow::bail!(
            "cannot refresh epic: the following tickets are not quiescent:\n{}",
            blockers.join("\n")
        );
    }

    let default_branch = &config.project.default_branch;

    let log_out = std::process::Command::new("git")
        .current_dir(root)
        .args(["log", "--oneline", "--no-decorate", &format!("{epic_branch}..{default_branch}")])
        .output()?;
    if !log_out.status.success() {
        anyhow::bail!("git log failed: {}", String::from_utf8_lossy(&log_out.stderr).trim());
    }
    let log_str = String::from_utf8_lossy(&log_out.stdout);
    let log_str = log_str.trim();

    if log_str.is_empty() {
        println!("epic branch is up to date with {default_branch}");
        return Ok(());
    }

    let pr_title = format!("{epic_id}: refresh from {default_branch}");
    let pr_body = log_str.to_string();

    apm_core::git::push_branch_tracking(root, &epic_branch)?;

    let mut messages = vec![];
    apm_core::github::gh_pr_create_or_update_between(
        root,
        default_branch,
        &epic_branch,
        &pr_title,
        &pr_body,
        &mut messages,
    )?;
    for m in &messages {
        println!("{m}");
    }
    Ok(())
}

pub fn run_show(root: &std::path::Path, id_arg: &str, no_aggressive: bool) -> anyhow::Result<()> {
    let ctx = CmdContext::load(root, no_aggressive)?;

    let matches = apm_core::epic::find_epic_branches(root, id_arg);
    let branch = match matches.len() {
        0 => anyhow::bail!("no epic matching '{id_arg}'"),
        1 => matches.into_iter().next().unwrap(),
        _ => anyhow::bail!(
            "ambiguous prefix '{id_arg}', matches:\n  {}",
            matches.join("\n  ")
        ),
    };

    let epic_id = epic_id_from_branch(&branch);
    let title = branch_to_title(&branch);

    let epic_tickets: Vec<_> = ctx.tickets
        .iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
        .collect();

    let state_configs: Vec<&apm_core::config::StateConfig> = epic_tickets
        .iter()
        .filter_map(|t| ctx.config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
        .collect();

    let derived = apm_core::epic::derive_epic_state(&state_configs);

    println!("Epic:   {title}");
    println!("Branch: {branch}");
    println!("State:  {derived}");

    if epic_tickets.is_empty() {
        println!();
        println!("(no tickets)");
        return Ok(());
    }

    // Column widths
    let id_w = 8usize;
    let state_w = 13usize;
    let title_w = 32usize;

    println!();
    println!(
        "{:<id_w$}  {:<state_w$}  {:<title_w$}  {}",
        "ID", "State", "Title", "Depends on"
    );
    println!(
        "{:-<id_w$}  {:-<state_w$}  {:-<title_w$}  {}",
        "", "", "", "----------"
    );

    for t in &epic_tickets {
        let fm = &t.frontmatter;
        let deps = fm
            .depends_on
            .as_deref()
            .map(|d| d.join(", "))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<id_w$}  {:<state_w$}  {:<title_w$}  {}",
            fm.id, fm.state, fm.title, deps
        );
    }

    Ok(())
}

pub fn run_set(root: &std::path::Path, id_arg: &str, field: &str, value: &str) -> anyhow::Result<()> {
    if field != "owner" {
        anyhow::bail!("unknown field {field:?}; valid fields: owner");
    }

    // Validate the epic exists.
    let matches = apm_core::epic::find_epic_branches(root, id_arg);
    if matches.is_empty() {
        eprintln!("error: no epic branch found matching '{id_arg}'");
        std::process::exit(1);
    }
    if matches.len() > 1 {
        anyhow::bail!(
            "ambiguous id '{id_arg}': matches {}\n  {}",
            matches.len(),
            matches.join("\n  ")
        );
    }
    let branch = &matches[0];
    let epic_id = epic_id_from_branch(branch).to_string();

    let config = apm_core::config::Config::load(root)?;

    // Pre-flight: validate the new owner
    let local = apm_core::config::LocalConfig::load(root);
    apm_core::validate::validate_owner(&config, &local, value)?;

    let (changed, skipped) = apm_core::epic::set_epic_owner(root, &epic_id, value, &config)?;
    println!("updated {changed} ticket(s), skipped {skipped} terminal ticket(s)");
    Ok(())
}


pub(crate) fn run_epic_clean(
    root: &Path,
    config: &apm_core::config::Config,
    dry_run: bool,
    yes: bool,
) -> Result<()> {
    // Get local epic branches.
    let local_output = std::process::Command::new("git")
        .current_dir(root)
        .args(["branch", "--list", "epic/*"])
        .output()?;

    let local_branches: Vec<String> = String::from_utf8_lossy(&local_output.stdout)
        .lines()
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    // Load all tickets.
    let tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;

    // Find epic branches whose derived state is "done".
    let mut candidates: Vec<String> = Vec::new();
    for branch in &local_branches {
        let id = apm_core::epic::epic_id_from_branch(branch);

        let epic_tickets: Vec<_> = tickets
            .iter()
            .filter(|t| t.frontmatter.epic.as_deref() == Some(id))
            .collect();

        let state_configs: Vec<&apm_core::config::StateConfig> = epic_tickets
            .iter()
            .filter_map(|t| config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
            .collect();

        if apm_core::epic::derive_epic_state(&state_configs) == "done" {
            candidates.push(branch.clone());
        }
    }

    if candidates.is_empty() {
        println!("Nothing to clean.");
        return Ok(());
    }

    // Print candidate list.
    println!("Would delete {} epic(s):", candidates.len());
    for branch in &candidates {
        let id = apm_core::epic::epic_id_from_branch(branch);
        let title = apm_core::epic::branch_to_title(branch);
        println!("  {id}  {title}");
    }

    if dry_run {
        println!("Dry run — no changes made.");
        return Ok(());
    }

    // Confirmation gate.
    if !yes {
        if std::io::stdout().is_terminal() {
            if !crate::util::prompt_yes_no(&format!("Delete {} epic(s)? [y/N] ", candidates.len()))? {
                println!("Aborted.");
                return Ok(());
            }
        } else {
            println!("Skipping — non-interactive terminal. Use --yes to confirm.");
            return Ok(());
        }
    }

    // Delete each candidate.
    for branch in &candidates {
        // Remove active worktree before attempting branch deletion.
        if let Some(wt_path) = apm_core::worktree::find_worktree_for_branch(root, branch) {
            if let Err(e) = apm_core::worktree::remove_worktree(root, &wt_path, false) {
                eprintln!(
                    "skipping {branch}: could not remove worktree at {}: {e}",
                    wt_path.display()
                );
                continue;
            }
        }

        // Delete local branch.
        let del_local = std::process::Command::new("git")
            .current_dir(root)
            .args(["branch", "-d", branch])
            .output()?;
        if !del_local.status.success() {
            eprintln!(
                "error: failed to delete local branch {branch}: {}",
                String::from_utf8_lossy(&del_local.stderr).trim()
            );
            continue;
        }

        // Delete remote branch; suppress "remote ref does not exist".
        let del_remote = std::process::Command::new("git")
            .current_dir(root)
            .args(["push", "origin", "--delete", branch])
            .output()?;
        if !del_remote.status.success() {
            let stderr = String::from_utf8_lossy(&del_remote.stderr);
            if !stderr.contains("remote ref does not exist")
                && !stderr.contains("error: unable to delete")
            {
                eprintln!(
                    "warning: failed to delete remote {branch}: {}",
                    stderr.trim()
                );
            }
        }

        println!("deleted {branch}");
    }

    Ok(())
}
