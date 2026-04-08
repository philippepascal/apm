use anyhow::{bail, Result};
use crate::{config::{Config, WorkerProfileConfig, WorkersConfig}, git, ticket};
use chrono::Utc;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

pub struct EffectiveWorkerParams {
    pub command: String,
    pub args: Vec<String>,
    pub model: Option<String>,
    pub env: std::collections::HashMap<String, String>,
    pub container: Option<String>,
}

fn resolve_profile<'a>(transition: &crate::config::TransitionConfig, config: &'a Config, warnings: &mut Vec<String>) -> Option<&'a WorkerProfileConfig> {
    let name = transition.profile.as_deref()?;
    match config.worker_profiles.get(name) {
        Some(p) => Some(p),
        None => {
            warnings.push(format!("warning: worker profile {name:?} not found — using global [workers] config"));
            None
        }
    }
}

pub fn effective_spawn_params(profile: Option<&WorkerProfileConfig>, workers: &WorkersConfig) -> EffectiveWorkerParams {
    let command = profile.and_then(|p| p.command.clone()).unwrap_or_else(|| workers.command.clone());
    let args = profile.and_then(|p| p.args.clone()).unwrap_or_else(|| workers.args.clone());
    let model = profile.and_then(|p| p.model.clone()).or_else(|| workers.model.clone());
    let container = profile.and_then(|p| p.container.clone()).or_else(|| workers.container.clone());
    let mut env = workers.env.clone();
    if let Some(p) = profile {
        for (k, v) in &p.env {
            env.insert(k.clone(), v.clone());
        }
    }
    EffectiveWorkerParams { command, args, model, env, container }
}

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
    pub warnings: Vec<String>,
}

pub struct RunNextOutput {
    pub ticket_id: Option<String>,
    pub messages: Vec<String>,
    pub warnings: Vec<String>,
    pub worker_pid: Option<u32>,
    pub log_path: Option<PathBuf>,
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
    params: &EffectiveWorkerParams,
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
    for (k, v) in &params.env {
        cmd.args(["--env", &format!("{k}={v}")]);
    }
    cmd.arg(image);
    cmd.arg(&params.command);
    for arg in &params.args {
        cmd.arg(arg);
    }
    if let Some(ref model) = params.model {
        cmd.args(["--model", model]);
    }
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
    params: &EffectiveWorkerParams,
    wt: &Path,
    worker_name: &str,
    worker_system: &str,
    ticket_content: &str,
    skip_permissions: bool,
    log_path: &Path,
) -> Result<std::process::Child> {
    let mut cmd = std::process::Command::new(&params.command);
    for arg in &params.args {
        cmd.arg(arg);
    }
    if let Some(ref model) = params.model {
        cmd.args(["--model", model]);
    }
    cmd.args(["--system-prompt", worker_system]);
    if skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }
    cmd.arg(ticket_content);
    cmd.env("APM_AGENT_NAME", worker_name);
    for (k, v) in &params.env {
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

pub fn run(root: &Path, id_arg: &str, no_aggressive: bool, spawn: bool, skip_permissions: bool, agent_name: &str) -> Result<StartOutput> {
    let mut warnings: Vec<String> = Vec::new();
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

    let triggering_transition = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|s| s.transitions.iter().find(|tr| tr.trigger == "command:start"));

    let new_state = triggering_transition
        .map(|tr| tr.to.clone())
        .unwrap_or_else(|| "in_progress".into());

    t.frontmatter.state = new_state.clone();
    t.frontmatter.updated_at = Some(now);
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
    crate::state::append_history(&mut t.body, &old_state, &new_state, &when, agent_name);

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
            warnings.push(format!("warning: fetch failed: {e:#}"));
        }
        if let Err(e) = git::fetch_branch(root, default_branch) {
            warnings.push(format!("warning: fetch {} failed: {e:#}", default_branch));
        }
    }

    git::commit_to_branch(root, &branch, &rel_path, &content, &format!("ticket({id}): start — {old_state} → {new_state}"))?;

    let worktrees_base = root.join(&config.worktrees.dir);
    let wt_display = git::ensure_worktree(root, &worktrees_base, &branch)?;
    git::sync_agent_dirs(root, &wt_display, &config.worktrees.agent_dirs, &mut warnings);

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
            warnings.push(format!(
                "warning: merge {} failed: {}",
                merge_ref,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
            None
        }
        Err(e) => {
            warnings.push(format!("warning: merge failed: {e}"));
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
            warnings,
        });
    }

    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("claude-{}-{:04x}", now_str, rand_u16());

    let profile = triggering_transition.and_then(|tr| resolve_profile(tr, &config, &mut warnings));
    let state_instructions = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_deref());
    let worker_system = resolve_system_prompt(root, profile, state_instructions);
    let ticket_content = format!("{}\n\n{content}", agent_role_prefix(profile, &id));
    let params = effective_spawn_params(profile, &config.workers);

    let log_path = wt_display.join(".apm-worker.log");

    let mut child = if let Some(ref image) = params.container.clone() {
        spawn_container_worker(
            root,
            &wt_display,
            image,
            &params,
            &config.workers.keychain,
            &worker_name,
            &worker_system,
            &ticket_content,
            skip_permissions,
            &log_path,
        )?
    } else {
        build_spawn_command(&params, &wt_display, &worker_name, &worker_system, &ticket_content, skip_permissions, &log_path)?
    };
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;

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
        warnings,
    })
}

pub fn run_next(root: &Path, no_aggressive: bool, spawn: bool, skip_permissions: bool) -> Result<RunNextOutput> {
    let mut messages: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
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
        messages.push("No actionable tickets.".to_string());
        return Ok(RunNextOutput { ticket_id: None, messages, warnings, worker_pid: None, log_path: None });
    };

    let id = candidate.frontmatter.id.clone();
    let old_state = candidate.frontmatter.state.clone();

    let triggering_transition_owned = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|s| s.transitions.iter().find(|tr| tr.trigger == "command:start"))
        .cloned();
    let profile = triggering_transition_owned.as_ref().and_then(|tr| resolve_profile(tr, &config, &mut warnings));
    let state_instructions = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_deref())
        .map(|s| s.to_string());
    let instructions_text = profile
        .and_then(|p| p.instructions.as_deref())
        .map(|path| {
            match std::fs::read_to_string(root.join(path)) {
                Ok(s) => s,
                Err(_) => { warnings.push("warning: instructions file not found".to_string()); String::new() }
            }
        })
        .filter(|s| !s.is_empty())
        .or_else(|| state_instructions.as_deref()
            .and_then(|path| {
                std::fs::read_to_string(root.join(path)).ok()
                    .or_else(|| { warnings.push("warning: instructions file not found".to_string()); None })
            }));
    let start_out = run(root, &id, no_aggressive, false, false, &agent_name)?;
    warnings.extend(start_out.warnings);

    if let Some(ref msg) = start_out.merge_message {
        messages.push(msg.clone());
    }
    messages.push(format!("{}: {} → {} (agent: {}, branch: {})", start_out.id, start_out.old_state, start_out.new_state, start_out.agent_name, start_out.branch));
    messages.push(format!("Worktree: {}", start_out.worktree_path.display()));

    let tickets2 = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(t) = tickets2.iter().find(|t| t.frontmatter.id == id) else {
        return Ok(RunNextOutput { ticket_id: Some(id), messages, warnings, worker_pid: None, log_path: None });
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
            messages.push(format!("Prompt:\n{prompt}"));
        }
        return Ok(RunNextOutput { ticket_id: Some(id), messages, warnings, worker_pid: None, log_path: None });
    }

    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("claude-{}-{:04x}", now_str, rand_u16());

    let profile2 = triggering_transition_owned.as_ref().and_then(|tr| resolve_profile(tr, &config, &mut warnings));
    let state_instr2 = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_deref());
    let worker_system = resolve_system_prompt(root, profile2, state_instr2);

    let raw = t.serialize()?;
    let ticket_content = format!("{}\n\n{raw}", agent_role_prefix(profile2, &id));
    let params = effective_spawn_params(profile2, &config.workers);

    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt_name = branch.replace('/', "-");
    let wt_path = root.join(&config.worktrees.dir).join(&wt_name);
    let wt_display = git::find_worktree_for_branch(root, &branch).unwrap_or(wt_path);

    let log_path = wt_display.join(".apm-worker.log");

    let mut child = if let Some(ref image) = params.container.clone() {
        spawn_container_worker(
            root,
            &wt_display,
            image,
            &params,
            &config.workers.keychain,
            &worker_name,
            &worker_system,
            &ticket_content,
            skip_permissions,
            &log_path,
        )?
    } else {
        build_spawn_command(&params, &wt_display, &worker_name, &worker_system, &ticket_content, skip_permissions, &log_path)?
    };
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });

    messages.push(format!("Worker spawned: PID={pid}, log={}", log_path.display()));
    messages.push(format!("Agent name: {worker_name}"));

    Ok(RunNextOutput { ticket_id: Some(id), messages, warnings, worker_pid: Some(pid), log_path: Some(log_path) })
}

pub fn spawn_next_worker(
    root: &Path,
    no_aggressive: bool,
    skip_permissions: bool,
    epic_filter: Option<&str>,
    messages: &mut Vec<String>,
    warnings: &mut Vec<String>,
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

    let triggering_transition_owned = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|s| s.transitions.iter().find(|tr| tr.trigger == "command:start"))
        .cloned();
    let profile = triggering_transition_owned.as_ref().and_then(|tr| resolve_profile(tr, &config, warnings));
    let state_instructions = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_deref())
        .map(|s| s.to_string());
    let instructions_text = profile
        .and_then(|p| p.instructions.as_deref())
        .map(|path| {
            match std::fs::read_to_string(root.join(path)) {
                Ok(s) => s,
                Err(_) => { warnings.push("warning: instructions file not found".to_string()); String::new() }
            }
        })
        .filter(|s| !s.is_empty())
        .or_else(|| state_instructions.as_deref()
            .and_then(|path| {
                std::fs::read_to_string(root.join(path)).ok()
                    .or_else(|| { warnings.push("warning: instructions file not found".to_string()); None })
            }));
    let start_out = run(root, &id, no_aggressive, false, false, &agent_name)?;
    warnings.extend(start_out.warnings);

    if let Some(ref msg) = start_out.merge_message {
        messages.push(msg.clone());
    }
    messages.push(format!("{}: {} → {} (agent: {}, branch: {})", start_out.id, start_out.old_state, start_out.new_state, start_out.agent_name, start_out.branch));
    messages.push(format!("Worktree: {}", start_out.worktree_path.display()));

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
    let _ = prompt; // prompt used only for run_next, not spawn_next_worker

    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("claude-{}-{:04x}", now_str, rand_u16());

    let profile2 = triggering_transition_owned.as_ref().and_then(|tr| resolve_profile(tr, &config, warnings));
    let state_instr2 = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_deref());
    let worker_system = resolve_system_prompt(root, profile2, state_instr2);

    let raw = t.serialize()?;
    let ticket_content = format!("{}\n\n{raw}", agent_role_prefix(profile2, &id));
    let params = effective_spawn_params(profile2, &config.workers);
    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt_name = branch.replace('/', "-");
    let wt_path = root.join(&config.worktrees.dir).join(&wt_name);
    let wt_display = git::find_worktree_for_branch(root, &branch).unwrap_or(wt_path);

    let log_path = wt_display.join(".apm-worker.log");

    let child = if let Some(ref image) = params.container.clone() {
        spawn_container_worker(
            root,
            &wt_display,
            image,
            &params,
            &config.workers.keychain,
            &worker_name,
            &worker_system,
            &ticket_content,
            skip_permissions,
            &log_path,
        )?
    } else {
        build_spawn_command(&params, &wt_display, &worker_name, &worker_system, &ticket_content, skip_permissions, &log_path)?
    };
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;

    messages.push(format!("Worker spawned: PID={pid}, log={}", log_path.display()));
    messages.push(format!("Agent name: {worker_name}"));

    Ok(Some((id, child, pid_path)))
}

fn resolve_system_prompt(root: &Path, profile: Option<&WorkerProfileConfig>, state_instructions: Option<&str>) -> String {
    if let Some(p) = profile {
        if let Some(ref instr_path) = p.instructions {
            if let Ok(content) = std::fs::read_to_string(root.join(instr_path)) {
                return content;
            }
        }
    }
    if let Some(path) = state_instructions {
        if let Ok(content) = std::fs::read_to_string(root.join(path)) {
            return content;
        }
    }
    let p = root.join(".apm/apm.worker.md");
    std::fs::read_to_string(p)
        .unwrap_or_else(|_| "You are an APM worker agent.".to_string())
}

fn agent_role_prefix(profile: Option<&WorkerProfileConfig>, id: &str) -> String {
    if let Some(p) = profile {
        if let Some(ref prefix) = p.role_prefix {
            return prefix.replace("<id>", id);
        }
    }
    format!("You are a Worker agent assigned to ticket #{id}.")
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
    use super::{resolve_agent_name, resolve_system_prompt, agent_role_prefix, resolve_profile, effective_spawn_params};
    use crate::config::{WorkerProfileConfig, WorkersConfig, TransitionConfig, CompletionStrategy, SatisfiesDeps};
    use std::sync::Mutex;
    use std::collections::HashMap;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn make_transition(profile: Option<&str>) -> TransitionConfig {
        TransitionConfig {
            to: "in_progress".into(),
            trigger: "command:start".into(),
            label: String::new(),
            hint: String::new(),
            completion: CompletionStrategy::None,
            focus_section: None,
            context_section: None,
            warning: None,
            profile: profile.map(|s| s.to_string()),
        }
    }

    fn make_profile(instructions: Option<&str>, role_prefix: Option<&str>) -> WorkerProfileConfig {
        WorkerProfileConfig {
            instructions: instructions.map(|s| s.to_string()),
            role_prefix: role_prefix.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    fn make_workers(command: &str, model: Option<&str>) -> WorkersConfig {
        WorkersConfig {
            command: command.to_string(),
            args: vec!["--print".to_string()],
            model: model.map(|s| s.to_string()),
            env: HashMap::new(),
            container: None,
            keychain: HashMap::new(),
        }
    }

    // --- resolve_profile ---

    #[test]
    fn resolve_profile_returns_profile_when_found() {
        let mut config = crate::config::Config {
            project: crate::config::ProjectConfig {
                name: "test".into(),
                description: String::new(),
                default_branch: "main".into(),
                collaborators: vec![],
            },
            ticket: Default::default(),
            tickets: Default::default(),
            workflow: Default::default(),
            agents: Default::default(),
            worktrees: Default::default(),
            sync: Default::default(),
            logging: Default::default(),
            workers: make_workers("claude", None),
            work: Default::default(),
            server: Default::default(),
            git_host: Default::default(),
            worker_profiles: HashMap::new(),
            load_warnings: vec![],
        };
        let profile = make_profile(Some(".apm/spec.md"), Some("Spec-Writer for #<id>"));
        config.worker_profiles.insert("spec_agent".into(), profile);

        let tr = make_transition(Some("spec_agent"));
        let mut w = Vec::new();
        assert!(resolve_profile(&tr, &config, &mut w).is_some());
    }

    #[test]
    fn resolve_profile_returns_none_for_missing_profile() {
        let config = crate::config::Config {
            project: crate::config::ProjectConfig {
                name: "test".into(),
                description: String::new(),
                default_branch: "main".into(),
                collaborators: vec![],
            },
            ticket: Default::default(),
            tickets: Default::default(),
            workflow: Default::default(),
            agents: Default::default(),
            worktrees: Default::default(),
            sync: Default::default(),
            logging: Default::default(),
            workers: make_workers("claude", None),
            work: Default::default(),
            server: Default::default(),
            git_host: Default::default(),
            worker_profiles: HashMap::new(),
            load_warnings: vec![],
        };
        let tr = make_transition(Some("nonexistent_profile"));
        let mut w = Vec::new();
        assert!(resolve_profile(&tr, &config, &mut w).is_none());
    }

    #[test]
    fn resolve_profile_returns_none_when_no_profile_on_transition() {
        let config = crate::config::Config {
            project: crate::config::ProjectConfig {
                name: "test".into(),
                description: String::new(),
                default_branch: "main".into(),
                collaborators: vec![],
            },
            ticket: Default::default(),
            tickets: Default::default(),
            workflow: Default::default(),
            agents: Default::default(),
            worktrees: Default::default(),
            sync: Default::default(),
            logging: Default::default(),
            workers: make_workers("claude", None),
            work: Default::default(),
            server: Default::default(),
            git_host: Default::default(),
            worker_profiles: HashMap::new(),
            load_warnings: vec![],
        };
        let tr = make_transition(None);
        let mut w = Vec::new();
        assert!(resolve_profile(&tr, &config, &mut w).is_none());
    }

    // --- effective_spawn_params ---

    #[test]
    fn effective_spawn_params_profile_command_overrides_global() {
        let workers = make_workers("claude", Some("sonnet"));
        let profile = WorkerProfileConfig {
            command: Some("my-claude".into()),
            ..Default::default()
        };
        let params = effective_spawn_params(Some(&profile), &workers);
        assert_eq!(params.command, "my-claude");
    }

    #[test]
    fn effective_spawn_params_falls_back_to_global_command() {
        let workers = make_workers("claude", None);
        let params = effective_spawn_params(None, &workers);
        assert_eq!(params.command, "claude");
    }

    #[test]
    fn effective_spawn_params_profile_model_overrides_global() {
        let workers = make_workers("claude", Some("sonnet"));
        let profile = WorkerProfileConfig {
            model: Some("opus".into()),
            ..Default::default()
        };
        let params = effective_spawn_params(Some(&profile), &workers);
        assert_eq!(params.model.as_deref(), Some("opus"));
    }

    #[test]
    fn effective_spawn_params_falls_back_to_global_model() {
        let workers = make_workers("claude", Some("sonnet"));
        let params = effective_spawn_params(None, &workers);
        assert_eq!(params.model.as_deref(), Some("sonnet"));
    }

    #[test]
    fn effective_spawn_params_profile_env_merged_over_global() {
        let mut workers = make_workers("claude", None);
        workers.env.insert("FOO".into(), "global".into());
        workers.env.insert("BAR".into(), "bar".into());

        let mut profile_env = HashMap::new();
        profile_env.insert("FOO".into(), "profile".into());
        let profile = WorkerProfileConfig {
            env: profile_env,
            ..Default::default()
        };
        let params = effective_spawn_params(Some(&profile), &workers);
        assert_eq!(params.env.get("FOO").map(|s| s.as_str()), Some("profile"));
        assert_eq!(params.env.get("BAR").map(|s| s.as_str()), Some("bar"));
    }

    #[test]
    fn effective_spawn_params_profile_container_overrides_global() {
        let mut workers = make_workers("claude", None);
        workers.container = Some("global-image".into());
        let profile = WorkerProfileConfig {
            container: Some("profile-image".into()),
            ..Default::default()
        };
        let params = effective_spawn_params(Some(&profile), &workers);
        assert_eq!(params.container.as_deref(), Some("profile-image"));
    }

    // --- resolve_system_prompt ---

    #[test]
    fn resolve_system_prompt_uses_profile_instructions() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/spec.md"), "SPEC WRITER").unwrap();
        std::fs::write(p.join(".apm/apm.worker.md"), "WORKER").unwrap();
        let profile = make_profile(Some(".apm/spec.md"), None);
        assert_eq!(resolve_system_prompt(p, Some(&profile), None), "SPEC WRITER");
    }

    #[test]
    fn resolve_system_prompt_falls_back_to_state_instructions() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/state.md"), "STATE INSTRUCTIONS").unwrap();
        std::fs::write(p.join(".apm/apm.worker.md"), "WORKER").unwrap();
        assert_eq!(resolve_system_prompt(p, None, Some(".apm/state.md")), "STATE INSTRUCTIONS");
    }

    #[test]
    fn resolve_system_prompt_falls_back_to_worker_when_no_profile_no_state() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm")).unwrap();
        std::fs::write(p.join(".apm/apm.worker.md"), "WORKER").unwrap();
        assert_eq!(resolve_system_prompt(p, None, None), "WORKER");
    }

    // --- agent_role_prefix ---

    #[test]
    fn agent_role_prefix_uses_profile_role_prefix() {
        let profile = make_profile(None, Some("You are a Spec-Writer agent assigned to ticket #<id>."));
        assert_eq!(
            agent_role_prefix(Some(&profile), "abc123"),
            "You are a Spec-Writer agent assigned to ticket #abc123."
        );
    }

    #[test]
    fn agent_role_prefix_falls_back_to_worker_default() {
        assert_eq!(
            agent_role_prefix(None, "abc123"),
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
