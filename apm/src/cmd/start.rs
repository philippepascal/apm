use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, no_aggressive: bool, spawn: bool, skip_permissions: bool) -> Result<()> {
    let agent_name = std::env::var("APM_AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("APM_AGENT_NAME is not set"))?;

    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    // apm start is only valid from "ready" — spec-writing states (new, ammend)
    // use the branch directly; blocked tickets go back to ready before restarting.
    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();

    let mut tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let id = ticket::resolve_id_in_slice(&tickets, id_arg)?;

    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket {id:?} not found");
    };

    let fm = &t.frontmatter;
    if !startable.is_empty() && !startable.contains(&fm.state.as_str()) {
        bail!(
            "ticket {id:?} is in state {:?} — not startable\n\
             Use `apm start` only from: {}",
            fm.state,
            startable.join(", ")
        );
    }

    let now = Utc::now();
    let old_state = t.frontmatter.state.clone();

    // Find the target state for this ticket's command:start transition.
    let new_state = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|s| s.transitions.iter().find(|tr| tr.trigger == "command:start"))
        .map(|tr| tr.to.clone())
        .unwrap_or_else(|| "in_progress".into());

    t.frontmatter.agent = Some(agent_name.clone());
    t.frontmatter.state = new_state.clone();
    t.frontmatter.updated_at = Some(now);
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
    super::state::append_history(&mut t.body, &old_state, &new_state, &when, &agent_name);

    let content = t.serialize()?;
    let rel_path = format!(
        "{}/{}",
        config.tickets.dir.to_string_lossy(),
        t.path.file_name().unwrap().to_string_lossy()
    );
    let branch = t
        .frontmatter
        .branch
        .clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));

    let default_branch = &config.project.default_branch;

    if aggressive {
        if let Err(e) = git::fetch_branch(root, &branch) {
            eprintln!("warning: fetch failed: {e:#}");
        }
        if let Err(e) = git::fetch_branch(root, default_branch) {
            eprintln!("warning: fetch {} failed: {e:#}", default_branch);
        }
    }

    git::commit_to_branch(root, &branch, &rel_path, &content, &format!("ticket({id}): start — {old_state} → {new_state}"))?;

    // Provision permanent worktree.
    // Worktree dir name: ticket-<id>-<slug> (branch name with / replaced by -)
    let wt_name = branch.replace('/', "-");
    let worktrees_base = root.join(&config.worktrees.dir);
    std::fs::create_dir_all(&worktrees_base)?;
    let wt_path = worktrees_base.join(&wt_name);

    if git::find_worktree_for_branch(root, &branch).is_none() {
        git::add_worktree(root, &wt_path, &branch)?;
    }

    let wt_display = git::find_worktree_for_branch(root, &branch)
        .unwrap_or(wt_path);

    // Merge the default branch into the ticket branch so the agent starts from current code.
    let remote_ref = format!("origin/{default_branch}");
    let merge_ref = if std::process::Command::new("git")
        .args(["rev-parse", "--verify", &remote_ref])
        .current_dir(&wt_display)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        remote_ref.as_str()
    } else {
        default_branch.as_str()
    };
    match std::process::Command::new("git")
        .args(["merge", merge_ref, "--no-edit"])
        .current_dir(&wt_display)
        .output()
    {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if !stdout.contains("Already up to date") {
                println!("Merged {merge_ref} into branch.");
            }
        }
        Ok(out) => eprintln!(
            "warning: merge {} failed: {}",
            merge_ref,
            String::from_utf8_lossy(&out.stderr).trim()
        ),
        Err(e) => eprintln!("warning: merge failed: {e}"),
    }

    println!("{id}: {old_state} → {new_state} (agent: {agent_name}, branch: {branch})");
    println!("Worktree: {}", wt_display.display());

    if !spawn {
        return Ok(());
    }

    // Generate worker agent name
    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("claude-{}-{:04x}", now_str, rand_u16());

    // Read worker instructions: prefer .apm/worker.md, fall back to apm.worker.md
    let worker_system = {
        let p1 = root.join(".apm/worker.md");
        let p2 = root.join("apm.worker.md");
        if p1.exists() { std::fs::read_to_string(p1) } else { std::fs::read_to_string(p2) }
            .unwrap_or_else(|_| "You are an APM worker agent.".to_string())
    };

    // Get ticket content, prepend role line
    let ticket_content = format!("You are a Worker agent assigned to ticket #{id}.\n\n{content}");

    // Build log path
    let log_path = wt_display.join(".apm-worker.log");

    // Build claude command
    let mut cmd = std::process::Command::new("claude");
    cmd.arg("--print");
    cmd.args(["--system-prompt", &worker_system]);
    if skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }
    cmd.arg(&ticket_content);
    cmd.env("APM_AGENT_NAME", &worker_name);
    cmd.current_dir(&wt_display);

    // Redirect stdout+stderr to log file
    let log_file = std::fs::File::create(&log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);

    // Spawn detached
    let mut child = cmd.spawn()?;
    let pid = child.id();

    // Write PID file; background thread waits for exit and removes it.
    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;
    let pid_path_cleanup = pid_path.clone();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = std::fs::remove_file(&pid_path_cleanup);
    });

    println!("Worker spawned: PID={pid}, log={}", log_path.display());
    println!("Agent name: {worker_name}");

    Ok(())
}

pub fn run_next(root: &Path, no_aggressive: bool, spawn: bool, skip_permissions: bool) -> Result<()> {
    let agent_name = std::env::var("APM_AGENT_NAME")
        .map_err(|_| anyhow::anyhow!("APM_AGENT_NAME is not set"))?;

    let config = Config::load(root)?;
    let p = &config.workflow.prioritization;
    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();
    let actionable = config.actionable_states_for("agent");
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;

    let Some(candidate) = ticket::pick_next(&tickets, &actionable, &startable, p.priority_weight, p.effort_weight, p.risk_weight) else {
        println!("No actionable tickets.");
        return Ok(());
    };

    let id = candidate.frontmatter.id.clone();
    let old_state = candidate.frontmatter.state.clone();

    // Look up state config for instructions and focus_section before claiming
    let instructions_text = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_ref())
        .and_then(|path| std::fs::read_to_string(root.join(path)).ok()
            .or_else(|| { eprintln!("warning: instructions file not found"); None }));

    // Run the normal start flow
    run(root, &id, no_aggressive, false, false)?;

    // Re-read the ticket from branch to get focus_section (it may have been set by supervisor)
    let tickets2 = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(t) = tickets2.iter().find(|t| t.frontmatter.id == id) else {
        return Ok(());
    };

    let focus_hint = if let Some(ref section) = t.frontmatter.focus_section {
        let hint = format!("Pay special attention to section: {section}");
        // Clear focus_section from ticket
        let rel_path = format!(
            "{}/{}",
            config.tickets.dir.to_string_lossy(),
            t.path.file_name().unwrap().to_string_lossy()
        );
        let branch = t.frontmatter.branch.clone()
            .or_else(|| git::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{id}"));
        let mut t_mut = t.clone();
        t_mut.frontmatter.focus_section = None;
        let cleared = t_mut.serialize()?;
        git::commit_to_branch(root, &branch, &rel_path, &cleared, &format!("ticket({id}): clear focus_section"))?;
        Some(hint)
    } else {
        None
    };

    // Compose prompt
    let mut prompt = String::new();
    if let Some(ref instr) = instructions_text {
        prompt.push_str(instr.trim());
        prompt.push('\n');
    }
    if let Some(ref hint) = focus_hint {
        if !prompt.is_empty() { prompt.push('\n'); }
        prompt.push_str(hint);
        prompt.push('\n');
    }

    if !spawn {
        if !prompt.is_empty() {
            println!("Prompt:\n{prompt}");
        }
        return Ok(());
    }

    // Spawn worker
    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("claude-{}-{:04x}", now_str, rand_u16());

    let worker_system = if !prompt.is_empty() {
        prompt
    } else {
        // Fall back to apm.worker.md or .apm/worker.md
        let wm = root.join(".apm/worker.md");
        let wm_old = root.join("apm.worker.md");
        if wm.exists() {
            std::fs::read_to_string(wm).unwrap_or_default()
        } else {
            std::fs::read_to_string(wm_old).unwrap_or_else(|_| "You are an APM worker agent.".to_string())
        }
    };

    let raw = t.serialize()?;
    let ticket_content = format!("You are a Worker agent assigned to ticket #{id}.\n\n{raw}");

    // Find the worktree
    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt_name = branch.replace('/', "-");
    let wt_path = root.join(&config.worktrees.dir).join(&wt_name);
    let wt_display = git::find_worktree_for_branch(root, &branch).unwrap_or(wt_path);

    let log_path = wt_display.join(".apm-worker.log");
    let mut cmd = std::process::Command::new("claude");
    cmd.arg("--print");
    cmd.args(["--system-prompt", &worker_system]);
    if skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }
    cmd.arg(&ticket_content);
    cmd.env("APM_AGENT_NAME", &worker_name);
    cmd.current_dir(&wt_display);

    let log_file = std::fs::File::create(&log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);

    let mut child = cmd.spawn()?;
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;
    let pid_path_cleanup = pid_path.clone();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = std::fs::remove_file(&pid_path_cleanup);
    });

    println!("Worker spawned: PID={pid}, log={}", log_path.display());
    println!("Agent name: {worker_name}");

    Ok(())
}

/// Like `run_next` with spawn=true, but returns the spawned `Child` handle so
/// the caller can wait for it.  Returns `None` if no actionable tickets exist.
/// Also returns the path to the PID file for cleanup on exit.
pub fn spawn_next_worker(
    root: &Path,
    no_aggressive: bool,
    skip_permissions: bool,
) -> Result<Option<(String, std::process::Child, std::path::PathBuf)>> {
    let config = Config::load(root)?;
    let p = &config.workflow.prioritization;
    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();
    let actionable = config.actionable_states_for("agent");
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;

    let Some(candidate) = ticket::pick_next(&tickets, &actionable, &startable, p.priority_weight, p.effort_weight, p.risk_weight) else {
        return Ok(None);
    };

    let id = candidate.frontmatter.id.clone();
    let old_state = candidate.frontmatter.state.clone();

    let instructions_text = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_ref())
        .and_then(|path| std::fs::read_to_string(root.join(path)).ok()
            .or_else(|| { eprintln!("warning: instructions file not found"); None }));

    run(root, &id, no_aggressive, false, false)?;

    let tickets2 = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(t) = tickets2.iter().find(|t| t.frontmatter.id == id) else {
        return Ok(None);
    };

    let focus_hint = if let Some(ref section) = t.frontmatter.focus_section {
        let hint = format!("Pay special attention to section: {section}");
        let rel_path = format!(
            "{}/{}",
            config.tickets.dir.to_string_lossy(),
            t.path.file_name().unwrap().to_string_lossy()
        );
        let branch = t.frontmatter.branch.clone()
            .or_else(|| git::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{id}"));
        let mut t_mut = t.clone();
        t_mut.frontmatter.focus_section = None;
        let cleared = t_mut.serialize()?;
        git::commit_to_branch(root, &branch, &rel_path, &cleared,
            &format!("ticket({id}): clear focus_section"))?;
        Some(hint)
    } else {
        None
    };

    let mut prompt = String::new();
    if let Some(ref instr) = instructions_text {
        prompt.push_str(instr.trim());
        prompt.push('\n');
    }
    if let Some(ref hint) = focus_hint {
        if !prompt.is_empty() { prompt.push('\n'); }
        prompt.push_str(hint);
        prompt.push('\n');
    }

    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("claude-{}-{:04x}", now_str, rand_u16());

    let worker_system = if !prompt.is_empty() {
        prompt
    } else {
        let wm = root.join(".apm/worker.md");
        let wm_old = root.join("apm.worker.md");
        if wm.exists() {
            std::fs::read_to_string(wm).unwrap_or_default()
        } else {
            std::fs::read_to_string(wm_old)
                .unwrap_or_else(|_| "You are an APM worker agent.".to_string())
        }
    };

    let raw = t.serialize()?;
    let ticket_content = format!("You are a Worker agent assigned to ticket #{id}.\n\n{raw}");
    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt_name = branch.replace('/', "-");
    let wt_path = root.join(&config.worktrees.dir).join(&wt_name);
    let wt_display = git::find_worktree_for_branch(root, &branch).unwrap_or(wt_path);

    let log_path = wt_display.join(".apm-worker.log");
    let mut cmd = std::process::Command::new("claude");
    cmd.arg("--print");
    cmd.args(["--system-prompt", &worker_system]);
    if skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }
    cmd.arg(&ticket_content);
    cmd.env("APM_AGENT_NAME", &worker_name);
    cmd.current_dir(&wt_display);

    let log_file = std::fs::File::create(&log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);

    let child = cmd.spawn()?;
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;

    println!("Worker spawned: PID={pid}, log={}", log_path.display());
    println!("Agent name: {worker_name}");

    Ok(Some((id, child, pid_path)))
}

fn write_pid_file(path: &std::path::Path, pid: u32, ticket_id: &str) -> Result<()> {
    let started_at = chrono::Utc::now().format("%Y-%m-%dT%H:%MZ").to_string();
    let content = serde_json::json!({
        "pid": pid,
        "ticket_id": ticket_id,
        "started_at": started_at,
    })
    .to_string();
    std::fs::write(path, content)?;
    Ok(())
}

fn rand_u16() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos() as u16
}
