use anyhow::Result;
use std::io::IsTerminal;
use std::path::Path;

fn warn_or_prompt_stale_epic(root: &Path, epic_id: &str) -> Result<bool> {
    let Ok(Some(ahead)) = apm_core::epic::ticket_epic_staleness(root, epic_id) else {
        return Ok(true);
    };
    let config = apm_core::config::Config::load(root)?;
    let default_branch = &config.project.default_branch;
    if std::io::stdout().is_terminal() {
        use std::io::Write;
        print!(
            "Warning: epic {epic_id} is {ahead} commit(s) behind {default_branch}. \
             Run `apm epic refresh {epic_id}` first. Start anyway? [Y/n] "
        );
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        if line.trim().eq_ignore_ascii_case("n") {
            return Ok(false);
        }
    } else {
        eprintln!("warning: epic {epic_id} is {ahead} commit(s) behind the default branch");
    }
    Ok(true)
}

pub fn run(root: &Path, id_arg: &str, no_aggressive: bool, spawn: bool, skip_permissions: bool, caller_name: &str) -> Result<()> {
    // Pre-flight: check epic staleness before the state transition.
    {
        let config = apm_core::config::Config::load(root)?;
        let tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;
        let id = apm_core::ticket::ticket_fmt::resolve_id_in_slice(&tickets, id_arg)?;
        if let Some(t) = tickets.iter().find(|t| t.frontmatter.id == id) {
            if let Some(ref epic_id) = t.frontmatter.epic {
                if !warn_or_prompt_stale_epic(root, epic_id)? {
                    return Ok(());
                }
            }
        }
    }
    let out = apm_core::start::run(root, id_arg, no_aggressive, spawn, skip_permissions, caller_name)?;
    for w in &out.warnings {
        eprintln!("{w}");
    }
    if let Some(ref msg) = out.merge_message {
        println!("{msg}");
    }
    println!("{}: {} → {} (caller: {}, branch: {})", out.id, out.old_state, out.new_state, out.caller_name, out.branch);
    println!("Worktree: {}", out.worktree_path.display());
    if let (Some(pid), Some(log)) = (out.worker_pid, out.log_path.as_ref()) {
        println!("Worker spawned: PID={pid}, log={}", log.display());
    }
    if let Some(ref wn) = out.worker_name {
        println!("Agent name: {wn}");
    }
    Ok(())
}

pub fn run_next(root: &Path, no_aggressive: bool, spawn: bool, skip_permissions: bool) -> Result<()> {
    // Pre-flight: check epic staleness of next candidate before transition.
    if let Some((_, Some(epic_id))) = apm_core::start::peek_next_candidate(root)? {
        if !warn_or_prompt_stale_epic(root, &epic_id)? {
            return Ok(());
        }
    }
    let out = apm_core::start::run_next(root, no_aggressive, spawn, skip_permissions)?;
    for w in &out.warnings {
        eprintln!("{w}");
    }
    for msg in &out.messages {
        println!("{msg}");
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn spawn_next_worker(
    root: &Path,
    no_aggressive: bool,
    skip_permissions: bool,
    epic_filter: Option<&str>,
    blocked_epics: &[String],
    default_blocked: bool,
) -> Result<Option<(String, Option<String>, apm_core::start::ManagedChild, std::path::PathBuf)>> {
    let mut messages = Vec::new();
    let mut warnings = Vec::new();
    let result = apm_core::start::spawn_next_worker(root, no_aggressive, skip_permissions, epic_filter, blocked_epics, default_blocked, &mut messages, &mut warnings)?;
    for w in &warnings {
        eprintln!("{w}");
    }
    for msg in &messages {
        println!("{msg}");
    }
    Ok(result)
}
