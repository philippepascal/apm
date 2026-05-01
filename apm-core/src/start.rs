use anyhow::{bail, Result};
use crate::{config::{Config, WorkerProfileConfig, WorkersConfig}, git, ticket, ticket_fmt};
use crate::wrapper::{WrapperContext, write_temp_file};
use chrono::Utc;
use std::path::{Path, PathBuf};

static DEPRECATION_WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
static DEPRECATION_TEST_LOG: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
#[cfg(test)]
static DEPRECATION_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn emit_deprecation_warning() {
    use std::sync::atomic::Ordering;
    if DEPRECATION_WARNED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
        let msg = "apm: deprecated: `[workers] command`, `args`, and `model` fields are deprecated — migrate to `agent` and `[workers.options]`";
        eprintln!("{msg}");
        #[cfg(test)]
        DEPRECATION_TEST_LOG.lock().unwrap().push(msg.to_string());
    }
}

pub struct EffectiveWorkerParams {
    pub command: String,
    pub args: Vec<String>,
    pub model: Option<String>,
    pub env: std::collections::HashMap<String, String>,
    pub container: Option<String>,
    pub agent: String,
    pub options: std::collections::HashMap<String, String>,
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
    // Legacy command/args (kept for check_output_format_supported backward compat)
    let command = profile.and_then(|p| p.command.clone())
        .or_else(|| workers.command.clone())
        .unwrap_or_else(|| "claude".to_string());
    let args = profile.and_then(|p| p.args.clone())
        .or_else(|| workers.args.clone())
        .unwrap_or_else(|| vec!["--print".to_string()]);

    // Agent resolution: profile > workers > default "claude"
    let raw_agent = profile.and_then(|p| p.agent.clone())
        .or_else(|| workers.agent.clone());

    // Emit deprecation warning when legacy fields present but agent absent
    let has_legacy = workers.command.is_some()
        || workers.args.is_some()
        || workers.model.is_some()
        || profile.map(|p| p.command.is_some() || p.args.is_some() || p.model.is_some()).unwrap_or(false);
    if raw_agent.is_none() && has_legacy {
        emit_deprecation_warning();
    }

    let agent = raw_agent.unwrap_or_else(|| "claude".to_string());

    // Options merge: workers.options base, profile.options overrides on collision
    let mut options = workers.options.clone();
    if let Some(p) = profile {
        for (k, v) in &p.options {
            options.insert(k.clone(), v.clone());
        }
    }

    // Model: options.model > legacy profile.model > legacy workers.model
    let model = options.get("model").cloned()
        .or_else(|| profile.and_then(|p| p.model.clone()))
        .or_else(|| workers.model.clone());

    // Env merge
    let mut env = workers.env.clone();
    if let Some(p) = profile {
        for (k, v) in &p.env {
            env.insert(k.clone(), v.clone());
        }
    }

    let container = profile.and_then(|p| p.container.clone())
        .or_else(|| workers.container.clone());

    EffectiveWorkerParams { command, args, model, env, container, agent, options }
}

fn apply_frontmatter_agent(
    agent: &mut String,
    frontmatter: &ticket_fmt::Frontmatter,
    profile_name: &str,
) {
    if let Some(ov) = frontmatter.agent_overrides.get(profile_name) {
        *agent = ov.clone();
    } else if let Some(a) = &frontmatter.agent {
        *agent = a.clone();
    }
    // else: keep config-resolved agent unchanged
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

pub(crate) fn check_output_format_supported(binary: &str) -> Result<()> {
    let out = std::process::Command::new(binary)
        .arg("--help")
        .output()
        .map_err(|e| anyhow::anyhow!(
            "failed to run `{binary} --help` to check worker-driver compatibility: {e}"
        ))?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if combined.contains("--output-format") {
        Ok(())
    } else {
        bail!(
            "worker binary `{binary}` does not advertise `--output-format` in its \
             --help output; the flag `--output-format stream-json` is required for \
             full transcript capture in .apm-worker.log.\n\
             Upgrade the binary to a version that supports this flag, or configure \
             an alternative worker command in your apm.toml [workers] section."
        )
    }
}

pub struct ManagedChild {
    pub inner: std::process::Child,
    temp_files: Vec<PathBuf>,
}

impl std::ops::Deref for ManagedChild {
    type Target = std::process::Child;
    fn deref(&self) -> &std::process::Child { &self.inner }
}

impl std::ops::DerefMut for ManagedChild {
    fn deref_mut(&mut self) -> &mut std::process::Child { &mut self.inner }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        for f in &self.temp_files {
            let _ = std::fs::remove_file(f);
        }
    }
}

fn spawn_worker(ctx: &WrapperContext, agent: &str, project_root: &Path) -> Result<std::process::Child> {
    use crate::wrapper::{resolve_wrapper, resolve_builtin, WrapperKind, Wrapper};
    use crate::wrapper::custom::CustomWrapper;

    match resolve_wrapper(project_root, agent)? {
        Some(WrapperKind::Custom { script_path, manifest }) => {
            CustomWrapper { script_path, manifest }.spawn(ctx)
        }
        Some(WrapperKind::Builtin(name)) => {
            resolve_builtin(&name).expect("known built-in").spawn(ctx)
        }
        None => anyhow::bail!(
            "agent {:?} not found: checked built-ins {{claude}} and '.apm/agents/{agent}/'",
            agent
        ),
    }
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

    let ticket_epic_id = t.frontmatter.epic.clone();
    let ticket_depends_on = t.frontmatter.depends_on.clone().unwrap_or_default();
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
        .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
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

    let wt_display = crate::worktree::provision_worktree(root, &config, &branch, &mut warnings)?;

    let ref_to_merge = if crate::git_util::remote_branch_tip(&wt_display, &merge_base).is_some() {
        format!("origin/{merge_base}")
    } else {
        merge_base.to_string()
    };
    let merge_message = crate::git_util::merge_ref(&wt_display, &ref_to_merge, &mut warnings);

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

    let profile_name = triggering_transition
        .and_then(|tr| tr.profile.as_deref())
        .unwrap_or("")
        .to_string();
    let profile = triggering_transition.and_then(|tr| resolve_profile(tr, &config, &mut warnings));
    let state_instructions = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_deref());
    let worker_system = resolve_system_prompt(root, profile, state_instructions);
    let raw_prompt = format!("{}\n\n{content}", agent_role_prefix(profile, &id));
    let with_epic = with_epic_bundle(root, ticket_epic_id.as_deref(), &id, &config, raw_prompt);
    let ticket_content = with_dependency_bundle(root, &ticket_depends_on, &config, with_epic);
    let mut params = effective_spawn_params(profile, &config.workers);
    apply_frontmatter_agent(&mut params.agent, &t.frontmatter, &profile_name);
    let role_prefix = profile.and_then(|p| p.role_prefix.clone());

    let log_path = wt_display.join(".apm-worker.log");

    let sys_file = write_temp_file("sys", &worker_system)?;
    let msg_file = write_temp_file("msg", &ticket_content)?;
    let ctx = WrapperContext {
        worker_name: worker_name.clone(),
        ticket_id: id.clone(),
        ticket_branch: branch.clone(),
        worktree_path: wt_display.clone(),
        system_prompt_file: sys_file.clone(),
        user_message_file: msg_file.clone(),
        skip_permissions,
        profile: profile_name,
        role_prefix,
        options: params.options.clone(),
        model: params.model.clone(),
        log_path: log_path.clone(),
        container: params.container.clone(),
        extra_env: params.env.clone(),
        root: root.to_path_buf(),
        keychain: config.workers.keychain.clone(),
    };
    check_output_format_supported(&params.command)?;
    let mut child = spawn_worker(&ctx, &params.agent, root)?;
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;

    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = std::fs::remove_file(&sys_file);
        let _ = std::fs::remove_file(&msg_file);
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
    let all_tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let agent_name = crate::config::resolve_caller_name();
    let current_user = crate::config::resolve_identity(root);

    // Filter out tickets whose epic already has the max number of active workers.
    let active_epic_ids: Vec<Option<String>> = all_tickets.iter()
        .filter(|t| {
            let s = t.frontmatter.state.as_str();
            actionable.contains(&s) && !startable.contains(&s)
        })
        .map(|t| t.frontmatter.epic.clone())
        .collect();
    let blocked = config.blocked_epics(&active_epic_ids);
    let default_blocked = config.is_default_branch_blocked(&active_epic_ids);
    let tickets: Vec<_> = all_tickets.into_iter()
        .filter(|t| match t.frontmatter.epic.as_deref() {
            Some(eid) => !blocked.iter().any(|b| b == eid),
            None => !default_blocked,
        })
        .collect();

    let Some(candidate) = ticket::pick_next(&tickets, &actionable, &startable, p.priority_weight, p.effort_weight, p.risk_weight, &config, Some(&agent_name), Some(&current_user)) else {
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
            .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
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

    let profile_name2 = triggering_transition_owned.as_ref()
        .and_then(|tr| tr.profile.as_deref())
        .unwrap_or("")
        .to_string();
    let profile2 = triggering_transition_owned.as_ref().and_then(|tr| resolve_profile(tr, &config, &mut warnings));
    let state_instr2 = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_deref());
    let worker_system = resolve_system_prompt(root, profile2, state_instr2);

    let raw = t.serialize()?;
    let dep_ids_next = t.frontmatter.depends_on.clone().unwrap_or_default();
    let raw_prompt_next = format!("{}\n\n{raw}", agent_role_prefix(profile2, &id));
    let with_epic_next = with_epic_bundle(root, t.frontmatter.epic.as_deref(), &id, &config, raw_prompt_next);
    let ticket_content = with_dependency_bundle(root, &dep_ids_next, &config, with_epic_next);
    let mut params = effective_spawn_params(profile2, &config.workers);
    apply_frontmatter_agent(&mut params.agent, &t.frontmatter, &profile_name2);
    let role_prefix2 = profile2.and_then(|p| p.role_prefix.clone());

    let branch = t.frontmatter.branch.clone()
        .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt_name = branch.replace('/', "-");
    let main_root = crate::git_util::main_worktree_root(root).unwrap_or_else(|| root.to_path_buf());
    let wt_path = main_root.join(&config.worktrees.dir).join(&wt_name);
    let wt_display = crate::worktree::find_worktree_for_branch(root, &branch).unwrap_or(wt_path);

    let log_path = wt_display.join(".apm-worker.log");

    let sys_file = write_temp_file("sys", &worker_system)?;
    let msg_file = write_temp_file("msg", &ticket_content)?;
    let ctx = WrapperContext {
        worker_name: worker_name.clone(),
        ticket_id: id.clone(),
        ticket_branch: branch.clone(),
        worktree_path: wt_display.clone(),
        system_prompt_file: sys_file.clone(),
        user_message_file: msg_file.clone(),
        skip_permissions,
        profile: profile_name2,
        role_prefix: role_prefix2,
        options: params.options.clone(),
        model: params.model.clone(),
        log_path: log_path.clone(),
        container: params.container.clone(),
        extra_env: params.env.clone(),
        root: root.to_path_buf(),
        keychain: config.workers.keychain.clone(),
    };
    check_output_format_supported(&params.command)?;
    let mut child = spawn_worker(&ctx, &params.agent, root)?;
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = std::fs::remove_file(&sys_file);
        let _ = std::fs::remove_file(&msg_file);
    });

    messages.push(format!("Worker spawned: PID={pid}, log={}", log_path.display()));
    messages.push(format!("Agent name: {worker_name}"));

    Ok(RunNextOutput { ticket_id: Some(id), messages, warnings, worker_pid: Some(pid), log_path: Some(log_path) })
}

#[allow(clippy::type_complexity)]
pub fn spawn_next_worker(
    root: &Path,
    no_aggressive: bool,
    skip_permissions: bool,
    epic_filter: Option<&str>,
    blocked_epics: &[String],
    default_blocked: bool,
    messages: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<Option<(String, Option<String>, ManagedChild, PathBuf)>> {
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
    let tickets: Vec<ticket::Ticket> = {
        let epic_filtered: Vec<ticket::Ticket> = match epic_filter {
            Some(epic_id) => all_tickets.into_iter()
                .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
                .collect(),
            None => all_tickets,
        };
        epic_filtered.into_iter()
            .filter(|t| match t.frontmatter.epic.as_deref() {
                Some(eid) => !blocked_epics.iter().any(|b| b == eid),
                None => !default_blocked,
            })
            .collect()
    };
    let agent_name = crate::config::resolve_caller_name();
    let current_user = crate::config::resolve_identity(root);

    let Some(candidate) = ticket::pick_next(&tickets, &actionable, &startable, p.priority_weight, p.effort_weight, p.risk_weight, &config, Some(&agent_name), Some(&current_user)) else {
        return Ok(None);
    };

    let id = candidate.frontmatter.id.clone();
    let epic_id = candidate.frontmatter.epic.clone();
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
            .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
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

    let profile_name2 = triggering_transition_owned.as_ref()
        .and_then(|tr| tr.profile.as_deref())
        .unwrap_or("")
        .to_string();
    let profile2 = triggering_transition_owned.as_ref().and_then(|tr| resolve_profile(tr, &config, warnings));
    let state_instr2 = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|sc| sc.instructions.as_deref());
    let worker_system = resolve_system_prompt(root, profile2, state_instr2);

    let raw = t.serialize()?;
    let dep_ids_snw = t.frontmatter.depends_on.clone().unwrap_or_default();
    let raw_prompt_snw = format!("{}\n\n{raw}", agent_role_prefix(profile2, &id));
    let with_epic_snw = with_epic_bundle(root, t.frontmatter.epic.as_deref(), &id, &config, raw_prompt_snw);
    let ticket_content = with_dependency_bundle(root, &dep_ids_snw, &config, with_epic_snw);
    let mut params = effective_spawn_params(profile2, &config.workers);
    apply_frontmatter_agent(&mut params.agent, &t.frontmatter, &profile_name2);
    let role_prefix2 = profile2.and_then(|p| p.role_prefix.clone());

    let branch = t.frontmatter.branch.clone()
        .or_else(|| ticket_fmt::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));
    let wt_name = branch.replace('/', "-");
    let main_root = crate::git_util::main_worktree_root(root).unwrap_or_else(|| root.to_path_buf());
    let wt_path = main_root.join(&config.worktrees.dir).join(&wt_name);
    let wt_display = crate::worktree::find_worktree_for_branch(root, &branch).unwrap_or(wt_path);

    let log_path = wt_display.join(".apm-worker.log");

    let sys_file = write_temp_file("sys", &worker_system)?;
    let msg_file = write_temp_file("msg", &ticket_content)?;
    let ctx = WrapperContext {
        worker_name: worker_name.clone(),
        ticket_id: id.clone(),
        ticket_branch: branch.clone(),
        worktree_path: wt_display.clone(),
        system_prompt_file: sys_file.clone(),
        user_message_file: msg_file.clone(),
        skip_permissions,
        profile: profile_name2,
        role_prefix: role_prefix2,
        options: params.options.clone(),
        model: params.model.clone(),
        log_path: log_path.clone(),
        container: params.container.clone(),
        extra_env: params.env.clone(),
        root: root.to_path_buf(),
        keychain: config.workers.keychain.clone(),
    };
    check_output_format_supported(&params.command)?;
    let child = spawn_worker(&ctx, &params.agent, root)?;
    let pid = child.id();

    let managed = ManagedChild {
        inner: child,
        temp_files: vec![sys_file, msg_file],
    };

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;

    messages.push(format!("Worker spawned: PID={pid}, log={}", log_path.display()));
    messages.push(format!("Agent name: {worker_name}"));

    Ok(Some((id, epic_id, managed, pid_path)))
}

/// If the ticket has dependencies, prepend a dependency context bundle to the
/// worker prompt content.  Tickets with no dependencies are unchanged.
fn with_dependency_bundle(root: &Path, depends_on: &[String], config: &Config, content: String) -> String {
    if depends_on.is_empty() {
        return content;
    }
    let bundle = crate::context::build_dependency_bundle(root, depends_on, config);
    if bundle.is_empty() {
        return content;
    }
    format!("{bundle}\n{content}")
}

/// If the ticket belongs to an epic, prepend an epic context bundle to the
/// worker prompt content.  Tickets without an epic are unchanged.
fn with_epic_bundle(root: &Path, epic_id: Option<&str>, ticket_id: &str, config: &Config, content: String) -> String {
    match epic_id {
        Some(eid) => {
            let bundle = crate::context::build_epic_bundle(root, eid, ticket_id, config);
            format!("{bundle}\n{content}")
        }
        None => content,
    }
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
    use super::{resolve_system_prompt, agent_role_prefix, resolve_profile, effective_spawn_params, check_output_format_supported, apply_frontmatter_agent, ManagedChild, DEPRECATION_WARNED, DEPRECATION_TEST_LOG, DEPRECATION_TEST_LOCK};
    use crate::config::{WorkerProfileConfig, WorkersConfig, TransitionConfig, CompletionStrategy};
    use std::collections::HashMap;

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
            on_failure: None,
            outcome: None,
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
            command: Some(command.to_string()),
            args: None,
            model: model.map(|s| s.to_string()),
            env: HashMap::new(),
            container: None,
            keychain: HashMap::new(),
            agent: None,
            options: HashMap::new(),
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
            context: Default::default(),
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
            context: Default::default(),
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
            context: Default::default(),
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

    // --- spawn worker cwd ---

    #[test]
    fn spawn_worker_cwd_is_ticket_worktree() {
        use std::os::unix::fs::PermissionsExt;

        let wt = tempfile::tempdir().unwrap();
        let log_dir = tempfile::tempdir().unwrap();
        let mock_dir = tempfile::tempdir().unwrap();

        // Write mock 'claude' script — reports pwd to a file
        let mock_claude = mock_dir.path().join("claude");
        let cwd_file = wt.path().join("cwd-output.txt");
        let script = format!(concat!(
            "#!/bin/sh\n",
            "pwd > \"{}\"\n",
        ), cwd_file.display());
        std::fs::write(&mock_claude, &script).unwrap();
        std::fs::set_permissions(&mock_claude, std::fs::Permissions::from_mode(0o755)).unwrap();

        let sys_file = crate::wrapper::write_temp_file("sys", "system").unwrap();
        let msg_file = crate::wrapper::write_temp_file("msg", "ticket content").unwrap();

        let mut extra_env = HashMap::new();
        extra_env.insert(
            "PATH".to_string(),
            format!("{}:{}", mock_dir.path().display(), std::env::var("PATH").unwrap_or_default()),
        );

        let ctx = crate::wrapper::WrapperContext {
            worker_name: "test-worker".to_string(),
            ticket_id: "test-id".to_string(),
            ticket_branch: "ticket/test-id".to_string(),
            worktree_path: wt.path().to_path_buf(),
            system_prompt_file: sys_file.clone(),
            user_message_file: msg_file.clone(),
            skip_permissions: false,
            profile: "default".to_string(),
            role_prefix: None,
            options: HashMap::new(),
            model: None,
            log_path: log_dir.path().join("worker.log"),
            container: None,
            extra_env,
            root: wt.path().to_path_buf(),
            keychain: HashMap::new(),
        };

        let wrapper = crate::wrapper::resolve_builtin("claude").unwrap();
        let mut child = wrapper.spawn(&ctx).unwrap();
        child.wait().unwrap();
        let _ = std::fs::remove_file(&sys_file);
        let _ = std::fs::remove_file(&msg_file);

        let cwd_out = std::fs::read_to_string(&cwd_file)
            .expect("cwd-output.txt not written — mock claude did not run in expected cwd");
        let expected = wt.path().canonicalize().unwrap();
        assert_eq!(
            cwd_out.trim(),
            expected.to_str().unwrap(),
            "spawned worker CWD must equal the ticket worktree path"
        );
    }

    // --- check_output_format_supported ---

    #[test]
    fn check_output_format_supported_passes_when_flag_present() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("fake-claude");
        std::fs::write(&bin, "#!/bin/sh\necho '--output-format stream-json'\n").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(check_output_format_supported(bin.to_str().unwrap()).is_ok());
    }

    #[test]
    fn check_output_format_supported_errors_when_flag_absent() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("old-claude");
        std::fs::write(&bin, "#!/bin/sh\necho 'Usage: old-claude [options]'\n").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        let err = check_output_format_supported(bin.to_str().unwrap()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("--output-format"),
            "error message must name the missing flag: {msg}"
        );
        assert!(
            msg.contains(bin.to_str().unwrap()),
            "error message must include binary path: {msg}"
        );
    }

    // --- APM env vars on spawned process ---

    #[test]
    fn claude_wrapper_sets_apm_env_vars() {
        use std::os::unix::fs::PermissionsExt;

        let wt = tempfile::tempdir().unwrap();
        let log_dir = tempfile::tempdir().unwrap();
        let mock_dir = tempfile::tempdir().unwrap();
        let env_output = wt.path().join("env-output.txt");

        // Mock 'claude' writes all env vars to a file then exits
        let mock_claude = mock_dir.path().join("claude");
        let script = format!(
            "#!/bin/sh\nprintenv > \"{}\"\n",
            env_output.display()
        );
        std::fs::write(&mock_claude, &script).unwrap();
        std::fs::set_permissions(&mock_claude, std::fs::Permissions::from_mode(0o755)).unwrap();

        let sys_file = crate::wrapper::write_temp_file("sys", "system prompt").unwrap();
        let msg_file = crate::wrapper::write_temp_file("msg", "ticket content").unwrap();

        let mut extra_env = HashMap::new();
        extra_env.insert(
            "PATH".to_string(),
            format!("{}:{}", mock_dir.path().display(), std::env::var("PATH").unwrap_or_default()),
        );

        let ctx = crate::wrapper::WrapperContext {
            worker_name: "test-worker".to_string(),
            ticket_id: "abc123".to_string(),
            ticket_branch: "ticket/abc123-some-feature".to_string(),
            worktree_path: wt.path().to_path_buf(),
            system_prompt_file: sys_file.clone(),
            user_message_file: msg_file.clone(),
            skip_permissions: false,
            profile: "my-profile".to_string(),
            role_prefix: None,
            options: HashMap::new(),
            model: None,
            log_path: log_dir.path().join("worker.log"),
            container: None,
            extra_env,
            root: wt.path().to_path_buf(),
            keychain: HashMap::new(),
        };

        let wrapper = crate::wrapper::resolve_builtin("claude").unwrap();
        let mut child = wrapper.spawn(&ctx).unwrap();
        child.wait().unwrap();
        let _ = std::fs::remove_file(&sys_file);
        let _ = std::fs::remove_file(&msg_file);

        let env_content = std::fs::read_to_string(&env_output)
            .expect("env-output.txt not written — mock claude did not run");

        assert!(env_content.contains("APM_AGENT_NAME=test-worker"), "missing APM_AGENT_NAME\n{env_content}");
        assert!(env_content.contains("APM_TICKET_ID=abc123"), "missing APM_TICKET_ID\n{env_content}");
        assert!(env_content.contains("APM_TICKET_BRANCH=ticket/abc123-some-feature"), "missing APM_TICKET_BRANCH\n{env_content}");
        assert!(env_content.contains("APM_TICKET_WORKTREE="), "missing APM_TICKET_WORKTREE\n{env_content}");
        assert!(env_content.contains("APM_SYSTEM_PROMPT_FILE="), "missing APM_SYSTEM_PROMPT_FILE\n{env_content}");
        assert!(env_content.contains("APM_USER_MESSAGE_FILE="), "missing APM_USER_MESSAGE_FILE\n{env_content}");
        assert!(env_content.contains("APM_SKIP_PERMISSIONS=0"), "missing APM_SKIP_PERMISSIONS\n{env_content}");
        assert!(env_content.contains("APM_PROFILE=my-profile"), "missing APM_PROFILE\n{env_content}");
        assert!(env_content.contains("APM_WRAPPER_VERSION=1"), "missing APM_WRAPPER_VERSION\n{env_content}");
        assert!(env_content.contains("APM_BIN="), "missing APM_BIN\n{env_content}");

        // APM_BIN must point to an existing file
        if let Some(line) = env_content.lines().find(|l| l.starts_with("APM_BIN=")) {
            let path = line.trim_start_matches("APM_BIN=");
            assert!(std::path::Path::new(path).exists(), "APM_BIN path does not exist: {path}");
        }
    }

    // --- temp file cleanup ---

    #[test]
    fn temp_files_removed_after_child_exits() {
        use std::os::unix::fs::PermissionsExt;

        let wt = tempfile::tempdir().unwrap();
        let log_dir = tempfile::tempdir().unwrap();
        let mock_dir = tempfile::tempdir().unwrap();

        // Mock 'claude' that just exits immediately
        let mock_claude = mock_dir.path().join("claude");
        std::fs::write(&mock_claude, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&mock_claude, std::fs::Permissions::from_mode(0o755)).unwrap();

        let sys_file = crate::wrapper::write_temp_file("sys", "system").unwrap();
        let msg_file = crate::wrapper::write_temp_file("msg", "message").unwrap();

        assert!(sys_file.exists(), "sys_file should exist before spawn");
        assert!(msg_file.exists(), "msg_file should exist before spawn");

        let mut extra_env = HashMap::new();
        extra_env.insert(
            "PATH".to_string(),
            format!("{}:{}", mock_dir.path().display(), std::env::var("PATH").unwrap_or_default()),
        );

        let ctx = crate::wrapper::WrapperContext {
            worker_name: "test".to_string(),
            ticket_id: "test123".to_string(),
            ticket_branch: "ticket/test123".to_string(),
            worktree_path: wt.path().to_path_buf(),
            system_prompt_file: sys_file.clone(),
            user_message_file: msg_file.clone(),
            skip_permissions: false,
            profile: "default".to_string(),
            role_prefix: None,
            options: HashMap::new(),
            model: None,
            log_path: log_dir.path().join("worker.log"),
            container: None,
            extra_env,
            root: wt.path().to_path_buf(),
            keychain: HashMap::new(),
        };

        let wrapper = crate::wrapper::resolve_builtin("claude").unwrap();
        let child = wrapper.spawn(&ctx).unwrap();

        let mut managed = ManagedChild {
            inner: child,
            temp_files: vec![sys_file.clone(), msg_file.clone()],
        };
        managed.inner.wait().unwrap();
        drop(managed);

        assert!(!sys_file.exists(), "sys_file should be removed after ManagedChild is dropped");
        assert!(!msg_file.exists(), "msg_file should be removed after ManagedChild is dropped");
    }

    // --- agent/options resolution ---

    #[test]
    fn resolution_agent_profile_overrides_global() {
        let workers = WorkersConfig { agent: Some("codex".into()), ..Default::default() };
        let profile = WorkerProfileConfig { agent: Some("mock-happy".into()), ..Default::default() };
        let params = effective_spawn_params(Some(&profile), &workers);
        assert_eq!(params.agent, "mock-happy");
    }

    #[test]
    fn resolution_agent_falls_back_to_claude() {
        let params = effective_spawn_params(None, &WorkersConfig::default());
        assert_eq!(params.agent, "claude");
    }

    #[test]
    fn resolution_options_merge() {
        let mut workers = WorkersConfig { agent: Some("claude".into()), ..Default::default() };
        workers.options.insert("model".into(), "opus".into());
        workers.options.insert("timeout".into(), "30".into());
        let mut profile_opts = HashMap::new();
        profile_opts.insert("model".into(), "sonnet".into());
        let profile = WorkerProfileConfig { options: profile_opts, ..Default::default() };
        let params = effective_spawn_params(Some(&profile), &workers);
        assert_eq!(params.options.get("model").map(|s| s.as_str()), Some("sonnet"), "profile model should override workers model");
        assert_eq!(params.options.get("timeout").map(|s| s.as_str()), Some("30"), "non-overlapping key should survive");
    }

    #[test]
    fn deprecation_warning_emitted_once() {
        let _guard = DEPRECATION_TEST_LOCK.lock().unwrap();
        DEPRECATION_WARNED.store(false, std::sync::atomic::Ordering::SeqCst);
        DEPRECATION_TEST_LOG.lock().unwrap().clear();

        let workers = WorkersConfig { command: Some("claude".into()), ..Default::default() };
        effective_spawn_params(None, &workers);
        effective_spawn_params(None, &workers);

        let log = DEPRECATION_TEST_LOG.lock().unwrap();
        let count = log.iter().filter(|m: &&String| m.contains("deprecated")).count();
        assert_eq!(count, 1, "deprecated message should appear exactly once, found {count}");
    }

    #[test]
    fn legacy_model_forwarded_to_ctx() {
        let workers = WorkersConfig { model: Some("opus".into()), ..Default::default() };
        let params = effective_spawn_params(None, &workers);
        assert_eq!(params.model.as_deref(), Some("opus"));
    }

    #[test]
    fn options_model_takes_precedence_over_legacy() {
        let mut workers = WorkersConfig { model: Some("opus".into()), agent: Some("claude".into()), ..Default::default() };
        workers.options.insert("model".into(), "sonnet".into());
        let params = effective_spawn_params(None, &workers);
        assert_eq!(params.model.as_deref(), Some("sonnet"));
    }

    // --- APM_OPT_ env vars ---

    #[test]
    fn apm_opt_env_vars_set() {
        use std::os::unix::fs::PermissionsExt;

        let wt = tempfile::tempdir().unwrap();
        let log_dir = tempfile::tempdir().unwrap();
        let mock_dir = tempfile::tempdir().unwrap();
        let env_output = wt.path().join("env-output.txt");

        let mock_claude = mock_dir.path().join("claude");
        let script = format!("#!/bin/sh\nprintenv > \"{}\"\n", env_output.display());
        std::fs::write(&mock_claude, &script).unwrap();
        std::fs::set_permissions(&mock_claude, std::fs::Permissions::from_mode(0o755)).unwrap();

        let sys_file = crate::wrapper::write_temp_file("sys", "system prompt").unwrap();
        let msg_file = crate::wrapper::write_temp_file("msg", "ticket content").unwrap();

        let mut extra_env = HashMap::new();
        extra_env.insert(
            "PATH".to_string(),
            format!("{}:{}", mock_dir.path().display(), std::env::var("PATH").unwrap_or_default()),
        );

        let mut options = HashMap::new();
        options.insert("model".to_string(), "sonnet".to_string());

        let ctx = crate::wrapper::WrapperContext {
            worker_name: "test-worker".to_string(),
            ticket_id: "abc123".to_string(),
            ticket_branch: "ticket/abc123".to_string(),
            worktree_path: wt.path().to_path_buf(),
            system_prompt_file: sys_file.clone(),
            user_message_file: msg_file.clone(),
            skip_permissions: false,
            profile: "default".to_string(),
            role_prefix: None,
            options,
            model: None,
            log_path: log_dir.path().join("worker.log"),
            container: None,
            extra_env,
            root: wt.path().to_path_buf(),
            keychain: HashMap::new(),
        };

        let wrapper = crate::wrapper::resolve_builtin("claude").unwrap();
        let mut child = wrapper.spawn(&ctx).unwrap();
        child.wait().unwrap();
        let _ = std::fs::remove_file(&sys_file);
        let _ = std::fs::remove_file(&msg_file);

        let env_content = std::fs::read_to_string(&env_output)
            .expect("env-output.txt not written");

        assert!(env_content.contains("APM_OPT_MODEL=sonnet"), "APM_OPT_MODEL=sonnet must be set\n{env_content}");
    }

    // --- apply_frontmatter_agent ---

    fn make_frontmatter_with_agent(agent: Option<&str>, overrides: &[(&str, &str)]) -> crate::ticket_fmt::Frontmatter {
        let agent_line = agent.map(|a| format!("agent = \"{a}\"\n")).unwrap_or_default();
        let overrides_section = if overrides.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = overrides.iter()
                .map(|(k, v)| format!("{k} = \"{v}\""))
                .collect();
            format!("[agent_overrides]\n{}\n", pairs.join("\n"))
        };
        let toml_str = format!("id = \"t\"\ntitle = \"T\"\nstate = \"new\"\n{agent_line}{overrides_section}");
        toml::from_str(&toml_str).unwrap()
    }

    #[test]
    fn apply_fm_profile_override_wins() {
        let fm = make_frontmatter_with_agent(Some("mock-sad"), &[("impl_agent", "mock-happy")]);
        let mut agent = "claude".to_string();
        apply_frontmatter_agent(&mut agent, &fm, "impl_agent");
        assert_eq!(agent, "mock-happy");
    }

    #[test]
    fn apply_fm_agent_field_wins_when_no_profile_match() {
        let fm = make_frontmatter_with_agent(Some("mock-sad"), &[]);
        let mut agent = "claude".to_string();
        apply_frontmatter_agent(&mut agent, &fm, "impl_agent");
        assert_eq!(agent, "mock-sad");
    }

    #[test]
    fn apply_fm_profile_override_beats_agent_field() {
        let fm = make_frontmatter_with_agent(Some("mock-random"), &[("impl_agent", "claude")]);
        let mut agent = "other".to_string();
        apply_frontmatter_agent(&mut agent, &fm, "impl_agent");
        assert_eq!(agent, "claude");
    }

    #[test]
    fn apply_fm_no_fields_unchanged() {
        let fm = make_frontmatter_with_agent(None, &[]);
        let mut agent = "claude".to_string();
        apply_frontmatter_agent(&mut agent, &fm, "impl_agent");
        assert_eq!(agent, "claude");
    }
}
