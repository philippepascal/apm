use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, no_aggressive: bool, spawn: bool, skip_permissions: bool, agent_name: &str) -> Result<()> {
    let out = apm_core::start::run(root, id_arg, no_aggressive, spawn, skip_permissions, agent_name)?;
    for w in &out.warnings {
        eprintln!("{w}");
    }
    if let Some(ref msg) = out.merge_message {
        println!("{msg}");
    }
    println!("{}: {} → {} (agent: {}, branch: {})", out.id, out.old_state, out.new_state, out.agent_name, out.branch);
    println!("Worktree: {}", out.worktree_path.display());
    if let (Some(pid), Some(ref log)) = (out.worker_pid, out.log_path.as_ref()) {
        println!("Worker spawned: PID={pid}, log={}", log.display());
    }
    if let Some(ref wn) = out.worker_name {
        println!("Agent name: {wn}");
    }
    Ok(())
}

pub fn run_next(root: &Path, no_aggressive: bool, spawn: bool, skip_permissions: bool) -> Result<()> {
    let out = apm_core::start::run_next(root, no_aggressive, spawn, skip_permissions)?;
    for w in &out.warnings {
        eprintln!("{w}");
    }
    for msg in &out.messages {
        println!("{msg}");
    }
    Ok(())
}

pub fn spawn_next_worker(
    root: &Path,
    no_aggressive: bool,
    skip_permissions: bool,
    epic_filter: Option<&str>,
    blocked_epics: &[String],
) -> Result<Option<(String, Option<String>, std::process::Child, std::path::PathBuf)>> {
    let mut messages = Vec::new();
    let mut warnings = Vec::new();
    let result = apm_core::start::spawn_next_worker(root, no_aggressive, skip_permissions, epic_filter, blocked_epics, &mut messages, &mut warnings)?;
    for w in &warnings {
        eprintln!("{w}");
    }
    for msg in &messages {
        println!("{msg}");
    }
    Ok(result)
}
