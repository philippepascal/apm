use anyhow::{bail, Result};
use crate::{config::Config, git, ticket};
use chrono::Utc;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

pub struct StartOutput {
    pub id: String,
    pub old_state: String,
    pub new_state: String,
    pub agent_name: String,
    pub branch: String,
    pub worktree_path: PathBuf,
    pub merge_message: Option<String>,
    pub worker_pid: Option<u32>,
    pub log_path: Option<PathBuf>,
    pub worker_name: Option<String>,
}

pub fn resolve_agent_name() -> String {
    std::env::var("APM_AGENT_NAME")
        .or_else(|_| std::env::var("USER"))
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "apm".to_string())
}

fn git_config_value(root: &Path, key: &str) -> Option<String> {
    std::process::Command::new("git")
        .args(["config", key])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn spawn_container_worker(
    root: &Path,
    wt: &Path,
    image: &str,
    keychain: &std::collections::HashMap<String, String>,
    worker_name: &str,
    worker_system: &str,
    ticket_content: &str,
    skip_permissions: bool,
    log_path: &Path,
) -> anyhow::Result<std::process::Child> {
    let api_key = crate::credentials::resolve(
        "ANTHROPIC_API_KEY",
        keychain.get("ANTHROPIC_API_KEY").map(|s| s.as_str()),
    )?;

    let author_name = std::env::var("GIT_AUTHOR_NAME").ok()
        .filter(|v| !v.is_empty())
        .or_else(|| git_config_value(root, "user.name"))
        .unwrap_or_default();
    let author_email = std::env::var("GIT_AUTHOR_EMAIL").ok()
        .filter(|v| !v.is_empty())
        .or_else(|| git_config_value(root, "user.email"))
        .unwrap_or_default();
    let committer_name = std::env::var("GIT_COMMITTER_NAME").ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| author_name.clone());
    let committer_email = std::env::var("GIT_COMMITTER_EMAIL").ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| author_email.clone());

    let mut cmd = std::process::Command::new("docker");
    cmd.arg("run");
    cmd.arg("--rm");
    cmd.args(["--volume", &format!("{}:/workspace", wt.display())]);
    cmd.args(["--workdir", "/workspace"]);
    cmd.args(["--env", &format!("ANTHROPIC_API_KEY={api_key}")]);
    if !author_name.is_empty() {
        cmd.args(["--env", &format!("GIT_AUTHOR_NAME={author_name}")]);
    }
    if !author_email.is_empty() {
        cmd.args(["--env", &format!("GIT_AUTHOR_EMAIL={author_email}")]);
    }
    if !committer_name.is_empty() {
        cmd.args(["--env", &format!("GIT_COMMITTER_NAME={committer_name}")]);
    }
    if !committer_email.is_empty() {
        cmd.args(["--env", &format!("GIT_COMMITTER_EMAIL={committer_email}")]);
    }
    cmd.args(["--env", &format!("APM_AGENT_NAME={worker_name}")]);
    cmd.arg(image);
    cmd.arg("claude");
    cmd.arg("--print");
    cmd.args(["--system-prompt", worker_system]);
    if skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }
    cmd.arg(ticket_content);

    let log_file = std::fs::File::create(log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);
    cmd.process_group(0);

    let child = cmd.spawn()?;
    Ok(child)
}

fn build_spawn_command(
    config: &Config,
    wt: &Path,
    worker_name: &str,
    worker_system: &str,
    ticket_content: &str,
    skip_permissions: bool,
    log_path: &Path,
) -> Result<std::process::Child> {
    let wc = &config.workers;
    let mut cmd = std::process::Command::new(&wc.command);
    for arg in &wc.args {
        cmd.arg(arg);
    }
    if let Some(ref model) = wc.model {
        cmd.args(["--model", model]);
    }
    cmd.args(["--system-prompt", worker_system]);
    if skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }
    cmd.arg(ticket_content);
    cmd.env("APM_AGENT_NAME", worker_name);
    for (k, v) in &wc.env {
        cmd.env(k, v);
    }
    cmd.current_dir(wt);

    let log_file = std::fs::File::create(log_path)?;
    let log_clone = log_file.try_clone()?;
    cmd.stdout(log_file);
    cmd.stderr(log_clone);
    cmd.process_group(0);

    Ok(cmd.spawn()?)
}

fn owner_can_claim(ticket: &ticket::Ticket, new_owner: &str) -> bool {
    match ticket.frontmatter.owner.as_deref() {
        None => true,
        Some(existing) => existing == new_owner,
    }
}

pub fn run(root: &Path, id_arg: &str, no_aggressive: bool, spawn: bool, skip_permissions: bool, agent_name: &str) -> Result<StartOutput> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;
    let skip_permissions = skip_permissions || config.agents.skip_permissions;

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

    let new_state = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|s| s.transitions.iter().find(|tr| tr.trigger == "command:start"))
        .map(|tr| tr.to.clone())
        .unwrap_or_else(|| "in_progress".into());

    t.frontmatter.state = new_state.clone();
    t.frontmatter.updated_at = Some(now);
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
    crate::state::append_history(&mut t.body, &old_state, &new_state, &when, agent_name);

    let claimed = owner_can_claim(t, agent_name);
    if claimed {
        t.frontmatter.owner = Some(agent_name.to_string());
    } else {
        eprintln!(
            "warning: ticket {} is owned by {}; not overwriting (use `apm set {} owner <name>` to reassign)",
            id, t.frontmatter.owner.as_deref().unwrap_or("unknown"), id
        );
    }

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
    let merge_base = t.frontmatter.target_branch.clone()
        .unwrap_or_else(|| default_branch.to_string());

    if aggressive {
        if let Err(e) = git::fetch_branch(root, &branch) {
            eprintln!("warning: fetch failed: {e:#}");
        }
        if let Err(e) = git::fetch_branch(root, default_branch) {
            eprintln!("warning: fetch {} failed: {e:#}", default_branch);
        }
    }

    git::commit_to_branch(root, &branch, &rel_path, &content, &format!("ticket({id}): start — {old_state} → {new_state}"))?;

    let worktrees_base = root.join(&config.worktrees.dir);
    let wt_display = git::ensure_worktree(root, &worktrees_base, &branch)?;

    let remote_ref = format!("origin/{merge_base}");
    let merge_ref = if std::process::Command::new("git")
        .args(["rev-parse", "--verify", &remote_ref])
        .current_dir(&wt_display)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        remote_ref.as_str()
    } else {
        merge_base.as_str()
    };
    let merge_message = match std::process::Command::new("git")
        .args(["merge", merge_ref, "--no-edit"])
        .current_dir(&wt_display)
        .output()
    {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if !stdout.contains("Already up to date") {
                Some(format!("Merged {merge_ref} into branch."))
            } else {
                None
            }
        }
        Ok(out) => {
            eprintln!(
                "warning: merge {} failed: {}",
                merge_ref,
                String::from_utf8_lossy(&out.stderr).trim()
            );
            None
        }
        Err(e) => {
            eprintln!("warning: merge failed: {e}");
            None
        }
    };

    if !spawn {
        return Ok(StartOutput {
            id,
            old_state,
            new_state,
            agent_name: agent_name.to_string(),
            branch,
            worktree_path: wt_display,
            merge_message,
            worker_pid: None,
            log_path: None,
            worker_name: None,
        });
    }

    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("claude-{}-{:04x}", now_str, rand_u16());

    let worker_system = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_ref())
        .and_then(|path| std::fs::read_to_string(root.join(path)).ok()
            .or_else(|| { eprintln!("warning: instructions file not found"); None }))
        .or_else(|| std::fs::read_to_string(root.join(".apm/apm.worker.md")).ok())
        .unwrap_or_else(|| "You are an APM worker agent.".to_string());

    let ticket_content = format!("{}\n\n{content}", agent_role_prefix(&old_state, &id));

    let log_path = wt_display.join(".apm-worker.log");

    let mut child = if let Some(image) = &config.workers.container {
        spawn_container_worker(
            root,
            &wt_display,
            image,
            &config.workers.keychain,
            &worker_name,
            &worker_system,
            &ticket_content,
            skip_permissions,
            &log_path,
        )?
    } else {
        build_spawn_command(&config, &wt_display, &worker_name, &worker_system, &ticket_content, skip_permissions, &log_path)?
    };
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;

    if claimed {
        t.frontmatter.owner = Some(worker_name.clone());
        t.frontmatter.updated_at = Some(chrono::Utc::now());
        let spawn_content = t.serialize()?;
        git::commit_to_branch(root, &branch, &rel_path, &spawn_content,
            &format!("ticket({id}): set owner to spawned worker"))?;
    }

    std::thread::spawn(move || {
        let _ = child.wait();
    });

    Ok(StartOutput {
        id,
        old_state,
        new_state,
        agent_name: agent_name.to_string(),
        branch,
        worktree_path: wt_display,
        merge_message,
        worker_pid: Some(pid),
        log_path: Some(log_path),
        worker_name: Some(worker_name),
    })
}

pub fn run_next(root: &Path, no_aggressive: bool, spawn: bool, skip_permissions: bool) -> Result<()> {
    let config = Config::load(root)?;
    let skip_permissions = skip_permissions || config.agents.skip_permissions;
    let p = &config.workflow.prioritization;
    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();
    let actionable_owned = config.actionable_states_for("agent");
    let actionable: Vec<&str> = actionable_owned.iter().map(|s| s.as_str()).collect();
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let agent_name = resolve_agent_name();

    let Some(candidate) = ticket::pick_next(&tickets, &actionable, &startable, p.priority_weight, p.effort_weight, p.risk_weight, &config, Some(&agent_name)) else {
        println!("No actionable tickets.");
        return Ok(());
    };

    let id = candidate.frontmatter.id.clone();
    let old_state = candidate.frontmatter.state.clone();

    let instructions_text = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_ref())
        .and_then(|path| std::fs::read_to_string(root.join(path)).ok()
            .or_else(|| { eprintln!("warning: instructions file not found"); None }));
    let start_out = run(root, &id, no_aggressive, false, false, &agent_name)?;

    if let Some(ref msg) = start_out.merge_message {
        println!("{msg}");
    }
    println!("{}: {} → {} (agent: {}, branch: {})", start_out.id, start_out.old_state, start_out.new_state, start_out.agent_name, start_out.branch);
    println!("Worktree: {}", start_out.worktree_path.display());

    let tickets2 = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(t) = tickets2.iter().find(|t| t.frontmatter.id == id) else {
        return Ok(());
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
        git::commit_to_branch(root, &branch, &rel_path, &cleared, &format!("ticket({id}): clear focus_section"))?;
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

    if !spawn {
        if !prompt.is_empty() {
            println!("Prompt:\n{prompt}");
        }
        return Ok(());
    }

    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("claude-{}-{:04x}", now_str, rand_u16());

    let worker_system = resolve_system_prompt(root, &old_state);

    let raw = t.serialize()?;
    let ticket_content = format!("{}\n\n{raw}", agent_role_prefix(&old_state, &id));

    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt_name = branch.replace('/', "-");
    let wt_path = root.join(&config.worktrees.dir).join(&wt_name);
    let wt_display = git::find_worktree_for_branch(root, &branch).unwrap_or(wt_path);

    let log_path = wt_display.join(".apm-worker.log");

    let mut child = if let Some(image) = &config.workers.container {
        spawn_container_worker(
            root,
            &wt_display,
            image,
            &config.workers.keychain,
            &worker_name,
            &worker_system,
            &ticket_content,
            skip_permissions,
            &log_path,
        )?
    } else {
        build_spawn_command(&config, &wt_display, &worker_name, &worker_system, &ticket_content, skip_permissions, &log_path)?
    };
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });

    println!("Worker spawned: PID={pid}, log={}", log_path.display());
    println!("Agent name: {worker_name}");

    Ok(())
}

pub fn spawn_next_worker(
    root: &Path,
    no_aggressive: bool,
    skip_permissions: bool,
    epic_filter: Option<&str>,
) -> Result<Option<(String, std::process::Child, PathBuf)>> {
    let config = Config::load(root)?;
    let skip_permissions = skip_permissions || config.agents.skip_permissions;
    let p = &config.workflow.prioritization;
    let startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.transitions.iter().any(|tr| tr.trigger == "command:start"))
        .map(|s| s.id.as_str())
        .collect();
    let actionable_owned = config.actionable_states_for("agent");
    let actionable: Vec<&str> = actionable_owned.iter().map(|s| s.as_str()).collect();
    let all_tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let tickets: Vec<ticket::Ticket> = match epic_filter {
        Some(epic_id) => all_tickets.into_iter()
            .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
            .collect(),
        None => all_tickets,
    };
    let agent_name = resolve_agent_name();

    let Some(candidate) = ticket::pick_next(&tickets, &actionable, &startable, p.priority_weight, p.effort_weight, p.risk_weight, &config, Some(&agent_name)) else {
        return Ok(None);
    };

    let id = candidate.frontmatter.id.clone();
    let old_state = candidate.frontmatter.state.clone();

    let instructions_text = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_ref())
        .and_then(|path| std::fs::read_to_string(root.join(path)).ok()
            .or_else(|| { eprintln!("warning: instructions file not found"); None }));
    let start_out = run(root, &id, no_aggressive, false, false, &agent_name)?;

    if let Some(ref msg) = start_out.merge_message {
        println!("{msg}");
    }
    println!("{}: {} → {} (agent: {}, branch: {})", start_out.id, start_out.old_state, start_out.new_state, start_out.agent_name, start_out.branch);
    println!("Worktree: {}", start_out.worktree_path.display());

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

    let worker_system = resolve_system_prompt(root, &old_state);

    let raw = t.serialize()?;
    let ticket_content = format!("{}\n\n{raw}", agent_role_prefix(&old_state, &id));
    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt_name = branch.replace('/', "-");
    let wt_path = root.join(&config.worktrees.dir).join(&wt_name);
    let wt_display = git::find_worktree_for_branch(root, &branch).unwrap_or(wt_path);

    let log_path = wt_display.join(".apm-worker.log");

    let child = if let Some(image) = &config.workers.container {
        spawn_container_worker(
            root,
            &wt_display,
            image,
            &config.workers.keychain,
            &worker_name,
            &worker_system,
            &ticket_content,
            skip_permissions,
            &log_path,
        )?
    } else {
        build_spawn_command(&config, &wt_display, &worker_name, &worker_system, &ticket_content, skip_permissions, &log_path)?
    };
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;

    println!("Worker spawned: PID={pid}, log={}", log_path.display());
    println!("Agent name: {worker_name}");

    Ok(Some((id, child, pid_path)))
}

fn resolve_system_prompt(root: &Path, pre_transition_state: &str) -> String {
    let spec_writer_states = ["groomed", "ammend"];
    if spec_writer_states.contains(&pre_transition_state) {
        let p = root.join(".apm/apm.spec-writer.md");
        if let Ok(content) = std::fs::read_to_string(&p) {
            return content;
        }
    }
    let p = root.join(".apm/apm.worker.md");
    std::fs::read_to_string(p)
        .unwrap_or_else(|_| "You are an APM worker agent.".to_string())
}

fn agent_role_prefix(pre_transition_state: &str, id: &str) -> String {
    let spec_writer_states = ["groomed", "ammend"];
    if spec_writer_states.contains(&pre_transition_state) {
        format!("You are a Spec-Writer agent assigned to ticket #{id}.")
    } else {
        format!("You are a Worker agent assigned to ticket #{id}.")
    }
}

fn write_pid_file(path: &Path, pid: u32, ticket_id: &str) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::{owner_can_claim, resolve_agent_name, resolve_system_prompt, agent_role_prefix};
    use crate::ticket::Ticket;
    use std::path::Path;
    use std::sync::Mutex;

    fn make_ticket(owner: Option<&str>) -> Ticket {
        let owner_line = owner.map(|o| format!("owner = \"{o}\"\n")).unwrap_or_default();
        let raw = format!("+++\nid = \"abc\"\ntitle = \"T\"\nstate = \"ready\"\n{owner_line}+++\n");
        Ticket::parse(Path::new("tickets/abc.md"), &raw).unwrap()
    }

    #[test]
    fn owner_can_claim_when_unowned() {
        let t = make_ticket(None);
        assert!(owner_can_claim(&t, "alice"));
    }

    #[test]
    fn owner_can_claim_when_same_owner_resumes() {
        let t = make_ticket(Some("alice"));
        assert!(owner_can_claim(&t, "alice"));
    }

    #[test]
    fn owner_can_claim_blocked_when_different_owner() {
        let t = make_ticket(Some("alice"));
        assert!(!owner_can_claim(&t, "bob"));
    }

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    // --- resolve_system_prompt ---

    #[test]
    fn resolve_system_prompt_uses_spec_writer_for_groomed() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/apm.spec-writer.md"), "SPEC WRITER").unwrap();
        std::fs::write(p.join(".apm/apm.worker.md"), "WORKER").unwrap();
        assert_eq!(resolve_system_prompt(p, "groomed"), "SPEC WRITER");
    }

    #[test]
    fn resolve_system_prompt_uses_spec_writer_for_ammend() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/apm.spec-writer.md"), "SPEC WRITER").unwrap();
        std::fs::write(p.join(".apm/apm.worker.md"), "WORKER").unwrap();
        assert_eq!(resolve_system_prompt(p, "ammend"), "SPEC WRITER");
    }

    #[test]
    fn resolve_system_prompt_falls_back_to_worker_when_spec_writer_absent() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        // No apm.spec-writer.md — only worker
        std::fs::write(p.join(".apm/apm.worker.md"), "WORKER").unwrap();
        assert_eq!(resolve_system_prompt(p, "groomed"), "WORKER");
    }

    #[test]
    fn resolve_system_prompt_uses_worker_for_ready() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/apm.spec-writer.md"), "SPEC WRITER").unwrap();
        std::fs::write(p.join(".apm/apm.worker.md"), "WORKER").unwrap();
        assert_eq!(resolve_system_prompt(p, "ready"), "WORKER");
    }

    #[test]
    fn resolve_system_prompt_uses_worker_for_in_progress() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/apm.spec-writer.md"), "SPEC WRITER").unwrap();
        std::fs::write(p.join(".apm/apm.worker.md"), "WORKER").unwrap();
        assert_eq!(resolve_system_prompt(p, "in_progress"), "WORKER");
    }

    // --- agent_role_prefix ---

    #[test]
    fn agent_role_prefix_spec_writer_for_groomed() {
        assert_eq!(
            agent_role_prefix("groomed", "abc123"),
            "You are a Spec-Writer agent assigned to ticket #abc123."
        );
    }

    #[test]
    fn agent_role_prefix_spec_writer_for_ammend() {
        assert_eq!(
            agent_role_prefix("ammend", "abc123"),
            "You are a Spec-Writer agent assigned to ticket #abc123."
        );
    }

    #[test]
    fn agent_role_prefix_worker_for_ready() {
        assert_eq!(
            agent_role_prefix("ready", "abc123"),
            "You are a Worker agent assigned to ticket #abc123."
        );
    }

    #[test]
    fn agent_role_prefix_worker_for_in_progress() {
        assert_eq!(
            agent_role_prefix("in_progress", "abc123"),
            "You are a Worker agent assigned to ticket #abc123."
        );
    }

    #[test]
    fn prefers_apm_agent_name() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("APM_AGENT_NAME", "explicit-agent");
        assert_eq!(resolve_agent_name(), "explicit-agent");
        std::env::remove_var("APM_AGENT_NAME");
    }

    #[test]
    fn falls_back_to_user() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("APM_AGENT_NAME");
        std::env::set_var("USER", "unix-user");
        std::env::remove_var("USERNAME");
        assert_eq!(resolve_agent_name(), "unix-user");
        std::env::remove_var("USER");
    }

    #[test]
    fn falls_back_to_apm_literal() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("APM_AGENT_NAME");
        std::env::remove_var("USER");
        std::env::remove_var("USERNAME");
        assert_eq!(resolve_agent_name(), "apm");
    }

    #[test]
    fn epic_filter_keeps_only_matching_tickets() {
        use crate::ticket::Ticket;
        use std::path::Path;

        let make_ticket = |id: &str, epic: Option<&str>| {
            let epic_line = epic.map(|e| format!("epic = \"{e}\"\n")).unwrap_or_default();
            let raw = format!(
                "+++\nid = \"{id}\"\ntitle = \"T\"\nstate = \"ready\"\n{epic_line}+++\n"
            );
            Ticket::parse(Path::new("tickets/dummy.md"), &raw).unwrap()
        };

        let all_tickets = vec![
            make_ticket("aaa", Some("epic1")),
            make_ticket("bbb", Some("epic2")),
            make_ticket("ccc", None),
        ];

        let epic_id = "epic1";
        let filtered: Vec<Ticket> = all_tickets.into_iter()
            .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].frontmatter.id, "aaa");
    }

    #[test]
    fn no_epic_filter_keeps_all_tickets() {
        use crate::ticket::Ticket;
        use std::path::Path;

        let make_ticket = |id: &str, epic: Option<&str>| {
            let epic_line = epic.map(|e| format!("epic = \"{e}\"\n")).unwrap_or_default();
            let raw = format!(
                "+++\nid = \"{id}\"\ntitle = \"T\"\nstate = \"ready\"\n{epic_line}+++\n"
            );
            Ticket::parse(Path::new("tickets/dummy.md"), &raw).unwrap()
        };

        let all_tickets: Vec<Ticket> = vec![
            make_ticket("aaa", Some("epic1")),
            make_ticket("bbb", Some("epic2")),
            make_ticket("ccc", None),
        ];

        let count = all_tickets.len();
        let epic_filter: Option<&str> = None;
        let filtered: Vec<Ticket> = match epic_filter {
            Some(eid) => all_tickets.into_iter()
                .filter(|t| t.frontmatter.epic.as_deref() == Some(eid))
                .collect(),
            None => all_tickets,
        };
        assert_eq!(filtered.len(), count);
    }
}
