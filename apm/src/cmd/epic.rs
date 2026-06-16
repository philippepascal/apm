use anyhow::Result;
use std::io::{BufRead, IsTerminal, Write};
use std::path::Path;
use crate::ctx::CmdContext;
use apm_core::epic::{branch_to_title, epic_id_from_branch, MergeStatus};

fn freshness_label(ahead: usize, clean: bool) -> String {
    if ahead == 0 {
        "up to date".to_string()
    } else if clean {
        format!("↓{ahead} clean")
    } else {
        format!("↓{ahead} CONFLICTS")
    }
}

pub fn run_list(root: &Path) -> Result<()> {
    let ctx = CmdContext::load(root, false)?;

    let epic_branches = apm_core::epic::epic_branches(root)?;
    if epic_branches.is_empty() {
        return Ok(());
    }

    let tickets = ctx.tickets;
    let default_branch = &ctx.config.project.default_branch;

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

        let s = apm_core::epic::merge_tree_status(root, default_branch, branch)
            .unwrap_or(MergeStatus { ahead: 0, clean: true });
        println!("{id:<8} [{derived:<12}] {title:<40} {counts_str:<30} {}", freshness_label(s.ahead, s.clean));
    }

    Ok(())
}

pub fn run_new(root: &Path, title: String) -> Result<()> {
    let config = apm_core::config::Config::load(root)?;
    let branch = apm_core::epic::create(root, &title, &config)?;
    println!("{branch}");
    Ok(())
}

pub fn run_submit(root: &Path, id_arg: &str, merge: bool, _pr: bool, auto_mode: bool) -> Result<()> {
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

    let epic_id = epic_id_from_branch(&epic_branch);
    let pr_title = branch_to_title(&epic_branch);
    let default_branch = &config.project.default_branch;

    // 2. Determine whether to merge locally or push+PR.
    let do_merge = merge || (auto_mode && {
        let s = apm_core::epic::merge_tree_status(root, default_branch, &epic_branch)?;
        s.clean
    });

    if do_merge {
        let main_root = apm_core::git_util::main_worktree_root(root)
            .unwrap_or_else(|| root.to_path_buf());
        let head_out = std::process::Command::new("git")
            .current_dir(&main_root)
            .args(["symbolic-ref", "--short", "HEAD"])
            .output()?;
        let head = String::from_utf8_lossy(&head_out.stdout);
        if head.trim() != default_branch {
            anyhow::bail!(
                "cannot merge: main worktree is on '{}', not '{default_branch}'. \
                 Check out {default_branch} first, or use --pr.",
                head.trim()
            );
        }
        let mut messages = vec![];
        match apm_core::git_util::merge_ref(&main_root, &epic_branch, &mut messages) {
            Some(msg) => {
                for m in &messages { println!("{m}"); }
                println!("{msg}");
            }
            None => {
                if auto_mode {
                    // Auto fell back to PR due to conflict.
                    println!("merge would conflict; falling back to --pr");
                    apm_core::git::push_branch_tracking(root, &epic_branch)?;
                    let mut pr_messages = vec![];
                    apm_core::github::gh_pr_create_or_update(
                        root,
                        &epic_branch,
                        default_branch,
                        epic_id,
                        &pr_title,
                        &format!("Epic: {epic_branch}"),
                        &mut pr_messages,
                    )?;
                    for m in &pr_messages { println!("{m}"); }
                } else {
                    anyhow::bail!(
                        "merge conflict — resolve manually after checking out {default_branch}, \
                         or use --pr to open a PR instead"
                    );
                }
            }
        }
    } else {
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
        for m in &messages { println!("{m}"); }
    }
    Ok(())
}

pub fn run_close(root: &Path, id_arg: &str, force: bool) -> Result<()> {
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

    let epic_id = epic_id_from_branch(&epic_branch);
    let default_branch = &config.project.default_branch;

    // 2. Live-worker safety check (skipped when --force).
    if !force {
        let all_tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;
        let mut live_workers: Vec<(String, u32)> = Vec::new();
        for t in all_tickets.iter().filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id)) {
            let ticket_branch = t.frontmatter.branch.clone()
                .or_else(|| apm_core::ticket_fmt::branch_name_from_path(&t.path));
            if let Some(branch) = ticket_branch {
                if let Some(wt_path) = apm_core::worktree::find_worktree_for_branch(root, &branch) {
                    let pid_file = wt_path.join(".apm-worker.pid");
                    if pid_file.exists() {
                        if let Ok((pid, _)) = apm_core::worker::read_pid_file(&pid_file) {
                            if apm_core::worker::is_alive(pid) {
                                live_workers.push((t.frontmatter.id.clone(), pid));
                            }
                        }
                    }
                }
            }
        }
        if !live_workers.is_empty() {
            let rows = live_workers.iter()
                .map(|(id, pid)| format!("  {id:<8}  PID {pid}"))
                .collect::<Vec<_>>()
                .join("\n");
            anyhow::bail!(
                "epic has active worker(s):\n{rows}\nUse --force to close unconditionally."
            );
        }

        // Implemented-state guard.
        let epic_tickets: Vec<_> = all_tickets.iter()
            .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
            .collect();
        let state_configs: Vec<&apm_core::config::StateConfig> = epic_tickets
            .iter()
            .filter_map(|t| config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
            .collect();
        let derived = apm_core::epic::derive_epic_state(&state_configs);
        if derived == "implemented" {
            let terminal = config.terminal_state_ids();
            let non_terminal: Vec<_> = epic_tickets.iter()
                .filter(|t| !terminal.contains(&t.frontmatter.state))
                .collect();
            let rows = non_terminal.iter()
                .map(|t| format!("  {} \u{2014} {} ({})", t.frontmatter.id, t.frontmatter.title, t.frontmatter.state))
                .collect::<Vec<_>>()
                .join("\n");
            anyhow::bail!(
                "epic is in state 'implemented'; close these tickets first:\n{rows}\nUse --force to close unconditionally."
            );
        }
    }

    // 3. Determine main ref (origin preferred).
    let remote_ref = format!("refs/remotes/origin/{default_branch}");
    let main_ref = if std::process::Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "--verify", &remote_ref])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        format!("origin/{default_branch}")
    } else {
        default_branch.clone()
    };

    // 4. Check whether the epic branch is merged.
    let is_merged = apm_core::git::is_branch_content_merged(root, default_branch, &epic_branch)?;

    // 5. If not merged and not --force, bail with ahead count.
    if !is_merged && !force {
        let count_out = std::process::Command::new("git")
            .current_dir(root)
            .args(["rev-list", "--count", &format!("{main_ref}..{epic_branch}")])
            .output()?;
        let count = String::from_utf8_lossy(&count_out.stdout).trim().parse::<u64>().unwrap_or(0);
        anyhow::bail!(
            "epic has {count} commit(s) not yet in {default_branch}. \
             Use --force to delete unconditionally."
        );
    }

    // 6. Remove worktree if present.
    if let Some(wt_path) = apm_core::worktree::find_worktree_for_branch(root, &epic_branch) {
        apm_core::worktree::remove_worktree(root, &wt_path, true)?;
    }

    // 7. Delete local branch (force-delete).
    let del = std::process::Command::new("git")
        .current_dir(root)
        .args(["branch", "-D", &epic_branch])
        .output()?;
    if !del.status.success() {
        eprintln!(
            "warning: could not delete local branch {epic_branch}: {}",
            String::from_utf8_lossy(&del.stderr).trim()
        );
    }

    // 8. Delete remote branch (suppress "remote ref does not exist").
    let del_remote = std::process::Command::new("git")
        .current_dir(root)
        .args(["push", "origin", "--delete", &epic_branch])
        .output()?;
    if !del_remote.status.success() {
        let stderr = String::from_utf8_lossy(&del_remote.stderr);
        if !stderr.contains("remote ref does not exist") && !stderr.contains("error: unable to delete") {
            eprintln!(
                "warning: could not delete remote branch {epic_branch}: {}",
                stderr.trim()
            );
        }
    }

    println!("deleted epic/{epic_id}");
    Ok(())
}

pub fn run_refresh_epic(root: &Path, id_arg: &str, merge: bool, pr: bool, auto_mode: bool, push: bool, no_push: bool) -> Result<()> {
    let mut merge = merge;
    let mut pr = pr;
    let mut auto_mode = auto_mode;
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
    let default_branch = &config.project.default_branch;

    let status = apm_core::epic::merge_tree_status(root, default_branch, &epic_branch)?;

    let acting = merge || pr || auto_mode;

    if !acting {
        if status.ahead == 0 {
            println!("epic branch is up to date with {default_branch}");
            return Ok(());
        }
        let cleanliness = if status.clean { "clean" } else { "conflicted" };
        println!("{} commit(s) ahead on {default_branch}; merge would be {cleanliness}", status.ahead);
        if !std::io::stdout().is_terminal() {
            return Ok(());
        }
        print!("\nWhat would you like to do?\n  [1] Merge locally\n  [2] Open / update PR\n  [3] Auto (merge if clean, fall back to PR)\n  [4] Skip\nChoice [1-4]: ");
        std::io::stdout().flush()?;
        let mut choice = String::new();
        std::io::stdin().lock().read_line(&mut choice)?;
        match choice.trim() {
            "1" => merge = true,
            "2" => pr = true,
            "3" => auto_mode = true,
            _ => return Ok(()),
        }
    }

    let worktrees = apm_core::worktree::list_ticket_worktrees(root)?;
    let blockers = apm_core::epic::epic_is_quiescent(root, epic_id, &config, &worktrees)?;
    if !blockers.is_empty() {
        anyhow::bail!(
            "cannot refresh epic: the following tickets are not quiescent:\n{}",
            blockers.join("\n")
        );
    }

    if status.ahead == 0 {
        println!("epic branch is up to date with {default_branch}");
        return Ok(());
    }

    let do_merge = merge || (auto_mode && status.clean);

    if do_merge {
        let main_root = apm_core::git_util::main_worktree_root(root)
            .unwrap_or_else(|| root.to_path_buf());
        let worktrees_base = main_root.join(&config.worktrees.dir);
        let epic_wt_path = apm_core::worktree::find_worktree_for_branch(root, &epic_branch)
            .map(Ok)
            .unwrap_or_else(|| apm_core::worktree::ensure_worktree(root, &worktrees_base, &epic_branch))?;
        let mut messages = vec![];
        match apm_core::git_util::merge_ref(&epic_wt_path, default_branch, &mut messages) {
            Some(msg) => {
                for m in &messages {
                    println!("{m}");
                }
                println!("{msg}");
            }
            None => {
                anyhow::bail!(
                    "merge conflict — resolve manually after checking out {epic_branch}, or use --pr to open a PR instead"
                );
            }
        }

        let should_push = if push {
            true
        } else if no_push {
            false
        } else if std::io::stdout().is_terminal() {
            crate::util::prompt_yes_no_default_yes("Push refreshed epic to origin? [Y/n] ")?
        } else {
            false
        };

        if should_push {
            apm_core::git::push_branch_tracking(root, &epic_branch)?;
            println!("pushed {epic_branch} to origin");
        } else {
            eprintln!(
                "warning: {epic_branch} was not pushed; \
                 downstream `apm start` will read stale origin content until pushed manually"
            );
        }
    } else {
        let log_out = std::process::Command::new("git")
            .current_dir(root)
            .args(["log", "--oneline", "--no-decorate", &format!("{epic_branch}..{default_branch}")])
            .output()?;
        let pr_body = String::from_utf8_lossy(&log_out.stdout).trim().to_string();
        let pr_title = format!("{epic_id}: refresh from {default_branch}");

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

    let s = apm_core::epic::merge_tree_status(root, &ctx.config.project.default_branch, &branch)
        .unwrap_or(MergeStatus { ahead: 0, clean: true });

    println!("Epic:   {title}");
    println!("Branch: {branch}");
    println!("State:  {derived}");
    println!("Freshness: {}", freshness_label(s.ahead, s.clean));

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
        "{:<id_w$}  {:<state_w$}  {:<title_w$}  Depends on",
        "ID", "State", "Title"
    );
    println!(
        "{:-<id_w$}  {:-<state_w$}  {:-<title_w$}  ----------",
        "", "", ""
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



