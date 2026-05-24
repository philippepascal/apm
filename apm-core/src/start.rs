use anyhow::{bail, Context, Result};
use crate::{config::{Config, WorkersConfig}, git, ticket, ticket_fmt};
use crate::wrapper::{WrapperContext, write_temp_file};
use chrono::Utc;
use std::path::{Path, PathBuf};

const DEFAULT_WORKER_DEFAULT: &str = include_str!("default/agents/claude/apm.worker.md");
const DEFAULT_SPEC_WRITER_DEFAULT: &str = include_str!("default/agents/claude/apm.spec-writer.md");
const MOCK_HAPPY_WORKER_DEFAULT: &str = include_str!("default/agents/mock-happy/apm.worker.md");
const MOCK_HAPPY_SPEC_WRITER_DEFAULT: &str = include_str!("default/agents/mock-happy/apm.spec-writer.md");
const MOCK_SAD_WORKER_DEFAULT: &str = include_str!("default/agents/mock-sad/apm.worker.md");
const MOCK_SAD_SPEC_WRITER_DEFAULT: &str = include_str!("default/agents/mock-sad/apm.spec-writer.md");
const MOCK_RANDOM_WORKER_DEFAULT: &str = include_str!("default/agents/mock-random/apm.worker.md");
const MOCK_RANDOM_SPEC_WRITER_DEFAULT: &str = include_str!("default/agents/mock-random/apm.spec-writer.md");
const DEBUG_WORKER_DEFAULT: &str = include_str!("default/agents/debug/apm.worker.md");
const DEBUG_SPEC_WRITER_DEFAULT: &str = include_str!("default/agents/debug/apm.spec-writer.md");
const DEFAULT_MAIN_AGENT_MD: &str = include_str!("default/agents/claude/apm.main-agent.md");

/// Delay inserted between `git fetch` and `git merge` in aggressive mode to let
/// remote-propagation settle and reduce the fetch-race window.
const POST_FETCH_SETTLE_MS: u64 = 1_000;

pub struct ResolvedWorkerProfile {
    pub agent: String,
    pub role: String,
    pub env: std::collections::HashMap<String, String>,
    pub container: Option<String>,
    pub model: Option<String>,
}

fn parse_worker_profile(s: &str) -> Result<(String, String)> {
    match s.split_once('/') {
        Some((agent, role)) if !agent.is_empty() && !role.is_empty() =>
            Ok((agent.to_string(), role.to_string())),
        _ => bail!("invalid worker_profile {:?}: expected format \"agent/role\"", s),
    }
}

pub fn resolve_worker_profile(worker_profile_str: &str, workers: &WorkersConfig) -> Result<ResolvedWorkerProfile> {
    let (agent, role) = parse_worker_profile(worker_profile_str)?;
    Ok(ResolvedWorkerProfile {
        agent,
        role,
        env: workers.env.clone(),
        container: workers.container.clone(),
        model: workers.model.clone(),
    })
}

pub(crate) fn apply_frontmatter_agent(
    agent: &mut String,
    frontmatter: &ticket_fmt::Frontmatter,
    worker_profile: &str,
) {
    if let Some(ov) = frontmatter.agent_overrides.get(worker_profile) {
        *agent = ov.clone();
    } else if let Some(a) = &frontmatter.agent {
        *agent = a.clone();
    }
}

pub struct StartOutput {
    pub id: String,
    pub old_state: String,
    pub new_state: String,
    pub caller_name: String,
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

/// True when `agent` resolves to the built-in claude wrapper (no custom shadow).
/// The compatibility probe is only meaningful in that case.
pub(crate) fn should_check_claude_compat(root: &Path, agent: &str) -> bool {
    if agent != "claude" { return false; }
    matches!(
        crate::wrapper::resolve_wrapper(root, "claude"),
        Ok(Some(crate::wrapper::WrapperKind::Builtin(_)))
    )
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
             an alternative worker command in your .apm/config.toml [workers] section."
        )
    }
}

pub struct ManagedChild {
    pub inner: std::process::Child,
    temp_files: Vec<PathBuf>,
    /// When set, denial scanning is run on drop (claude wrapper only).
    /// Tuple: (log_path, worktree_path, ticket_id).
    denial_ctx: Option<(PathBuf, PathBuf, String)>,
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
        if let Some((log_path, worktree_path, ticket_id)) = &self.denial_ctx {
            run_denial_scan(log_path, worktree_path, ticket_id);
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
            "agent {:?} not found: checked built-ins {{{}}} and '.apm/agents/{agent}/'",
            agent,
            crate::wrapper::list_builtin_names().join(", ")
        ),
    }
}

/// Scan the worker transcript for permission denials, write the summary file,
/// and emit a warning to the APM log when apm-command denials are found.
fn run_denial_scan(log_path: &Path, worktree: &Path, ticket_id: &str) {
    let summary = crate::denial::scan_transcript(log_path, worktree, ticket_id);
    let summary_path = crate::denial::summary_path_for(log_path);
    crate::denial::write_summary(&summary_path, &summary);
    let unique_cmds = crate::denial::collect_unique_apm_commands(&summary);
    if !unique_cmds.is_empty() {
        crate::logger::log(
            "worker-diag",
            &format!(
                "apm_command_denial ticket {} denied apm commands: {}",
                ticket_id,
                unique_cmds.join(", ")
            ),
        );
    }
}

pub fn run(root: &Path, id_arg: &str, no_aggressive: bool, spawn: bool, skip_permissions: bool, caller_name: &str) -> Result<StartOutput> {
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
    crate::state::append_history(&mut t.body, &old_state, &new_state, &when, caller_name);

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
        std::thread::sleep(std::time::Duration::from_millis(POST_FETCH_SETTLE_MS));
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
            caller_name: caller_name.to_string(),
            branch,
            worktree_path: wt_display,
            merge_message,
            worker_pid: None,
            log_path: None,
            worker_name: None,
            warnings,
        });
    }

    let worker_profile_str = triggering_transition
        .and_then(|tr| tr.worker_profile.as_deref())
        .or_else(|| config.workers.default.as_deref())
        .unwrap_or("claude/worker")
        .to_string();
    let mut wp = resolve_worker_profile(&worker_profile_str, &config.workers)?;
    apply_frontmatter_agent(&mut wp.agent, &t.frontmatter, &worker_profile_str);

    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("{}-{}-{:04x}", wp.agent, now_str, rand_u16());
    let worker_system = build_system_prompt(root, config.agents.project.as_deref(), &wp.agent, &wp.role)?;
    let raw_prompt = format!("{}\n\n{content}", agent_role_prefix(&wp.role, &id));
    let ticket_content = with_dependency_bundle(root, &ticket_depends_on, &config, raw_prompt);
    let role_prefix = Some(agent_role_prefix(&wp.role, &id));

    let log_path = wt_display.join(".apm-worker.log");

    let sys_file = write_temp_file("sys", &worker_system)?;
    let msg_file = write_temp_file("msg", &ticket_content)?;
    let ctx = WrapperContext {
        worker_name: worker_name.clone(),
        agent_type: wp.agent.clone(),
        ticket_id: id.clone(),
        ticket_branch: branch.clone(),
        worktree_path: wt_display.clone(),
        system_prompt_file: sys_file.clone(),
        user_message_file: msg_file.clone(),
        skip_permissions,
        profile: worker_profile_str,
        role_prefix,
        options: std::collections::HashMap::new(),
        model: wp.model.clone(),
        log_path: log_path.clone(),
        container: wp.container.clone(),
        extra_env: wp.env.clone(),
        root: root.to_path_buf(),
        keychain: config.workers.keychain.clone(),
        current_state: new_state.clone(),
        command: Some(wp.agent.clone()),
    };
    if should_check_claude_compat(root, &wp.agent) {
        check_output_format_supported(&wp.agent)?;
    }
    let mut child = spawn_worker(&ctx, &wp.agent, root)?;
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;

    let enforce_isolation = skip_permissions || config.isolation.enforce_worktree_isolation;
    let wt_for_cleanup = wt_display.clone();
    let denial_log_path = log_path.clone();
    let denial_worktree = wt_display.clone();
    let denial_ticket_id = id.clone();
    let agent_for_diag = wp.agent.clone();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = std::fs::remove_file(&sys_file);
        let _ = std::fs::remove_file(&msg_file);
        if agent_for_diag == "claude" {
            run_denial_scan(&denial_log_path, &denial_worktree, &denial_ticket_id);
        }
        if enforce_isolation {
            let _ = crate::wrapper::hook_config::remove_hook_config(&wt_for_cleanup);
        }
    });

    Ok(StartOutput {
        id,
        old_state,
        new_state,
        caller_name: caller_name.to_string(),
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
    let caller_name = crate::config::resolve_caller_name();
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

    let Some(candidate) = ticket::pick_next(&tickets, &actionable, &startable, p.priority_weight, p.effort_weight, p.risk_weight, &config, Some(&caller_name), Some(&current_user)) else {
        messages.push("No actionable tickets.".to_string());
        return Ok(RunNextOutput { ticket_id: None, messages, warnings, worker_pid: None, log_path: None });
    };

    let id = candidate.frontmatter.id.clone();
    let old_state = candidate.frontmatter.state.clone();

    let triggering_transition_owned = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|s| s.transitions.iter().find(|tr| tr.trigger == "command:start"))
        .cloned();
    let worker_profile_str = triggering_transition_owned.as_ref()
        .and_then(|tr| tr.worker_profile.as_deref())
        .or_else(|| config.workers.default.as_deref())
        .unwrap_or("claude/worker")
        .to_string();
    let start_out = run(root, &id, no_aggressive, false, false, &caller_name)?;
    warnings.extend(start_out.warnings);

    if let Some(ref msg) = start_out.merge_message {
        messages.push(msg.clone());
    }
    messages.push(format!("{}: {} → {} (caller: {}, branch: {})", start_out.id, start_out.old_state, start_out.new_state, start_out.caller_name, start_out.branch));
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

    if !spawn {
        if let Some(ref hint) = focus_hint {
            messages.push(format!("Focus hint: {hint}"));
        }
        return Ok(RunNextOutput { ticket_id: Some(id), messages, warnings, worker_pid: None, log_path: None });
    }

    let mut wp = resolve_worker_profile(&worker_profile_str, &config.workers)?;
    apply_frontmatter_agent(&mut wp.agent, &t.frontmatter, &worker_profile_str);

    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("{}-{}-{:04x}", wp.agent, now_str, rand_u16());
    let worker_system = build_system_prompt(root, config.agents.project.as_deref(), &wp.agent, &wp.role)?;

    let raw = t.serialize()?;
    let dep_ids_next = t.frontmatter.depends_on.clone().unwrap_or_default();
    let mut raw_prompt_next = format!("{}\n\n{raw}", agent_role_prefix(&wp.role, &id));
    if let Some(ref hint) = focus_hint {
        raw_prompt_next.push_str(&format!("\n\n{hint}"));
    }
    let ticket_content = with_dependency_bundle(root, &dep_ids_next, &config, raw_prompt_next);
    let role_prefix = Some(agent_role_prefix(&wp.role, &id));

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
        agent_type: wp.agent.clone(),
        ticket_id: id.clone(),
        ticket_branch: branch.clone(),
        worktree_path: wt_display.clone(),
        system_prompt_file: sys_file.clone(),
        user_message_file: msg_file.clone(),
        skip_permissions,
        profile: worker_profile_str,
        role_prefix,
        options: std::collections::HashMap::new(),
        model: wp.model.clone(),
        log_path: log_path.clone(),
        container: wp.container.clone(),
        extra_env: wp.env.clone(),
        root: root.to_path_buf(),
        keychain: config.workers.keychain.clone(),
        current_state: t.frontmatter.state.clone(),
        command: Some(wp.agent.clone()),
    };
    if should_check_claude_compat(root, &wp.agent) {
        check_output_format_supported(&wp.agent)?;
    }
    let mut child = spawn_worker(&ctx, &wp.agent, root)?;
    let pid = child.id();

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;
    let enforce_isolation_next = skip_permissions || config.isolation.enforce_worktree_isolation;
    let wt_for_cleanup_next = wt_display.clone();
    let denial_log_path2 = log_path.clone();
    let denial_worktree2 = wt_display.clone();
    let denial_ticket_id2 = id.clone();
    let agent_for_diag2 = wp.agent.clone();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = std::fs::remove_file(&sys_file);
        let _ = std::fs::remove_file(&msg_file);
        if agent_for_diag2 == "claude" {
            run_denial_scan(&denial_log_path2, &denial_worktree2, &denial_ticket_id2);
        }
        if enforce_isolation_next {
            let _ = crate::wrapper::hook_config::remove_hook_config(&wt_for_cleanup_next);
        }
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
    let caller_name = crate::config::resolve_caller_name();
    let current_user = crate::config::resolve_identity(root);

    let Some(candidate) = ticket::pick_next(&tickets, &actionable, &startable, p.priority_weight, p.effort_weight, p.risk_weight, &config, Some(&caller_name), Some(&current_user)) else {
        return Ok(None);
    };

    let id = candidate.frontmatter.id.clone();
    let epic_id = candidate.frontmatter.epic.clone();
    let old_state = candidate.frontmatter.state.clone();

    let triggering_transition_owned = config.workflow.states.iter()
        .find(|s| s.id == old_state)
        .and_then(|s| s.transitions.iter().find(|tr| tr.trigger == "command:start"))
        .cloned();
    let worker_profile_str = triggering_transition_owned.as_ref()
        .and_then(|tr| tr.worker_profile.as_deref())
        .or_else(|| config.workers.default.as_deref())
        .unwrap_or("claude/worker")
        .to_string();
    let start_out = run(root, &id, no_aggressive, false, false, &caller_name)?;
    warnings.extend(start_out.warnings);

    if let Some(ref msg) = start_out.merge_message {
        messages.push(msg.clone());
    }
    messages.push(format!("{}: {} → {} (caller: {}, branch: {})", start_out.id, start_out.old_state, start_out.new_state, start_out.caller_name, start_out.branch));
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

    let mut wp = resolve_worker_profile(&worker_profile_str, &config.workers)?;
    apply_frontmatter_agent(&mut wp.agent, &t.frontmatter, &worker_profile_str);

    let now_str = chrono::Utc::now().format("%m%d-%H%M").to_string();
    let worker_name = format!("{}-{}-{:04x}", wp.agent, now_str, rand_u16());
    let worker_system = build_system_prompt(root, config.agents.project.as_deref(), &wp.agent, &wp.role)?;

    let raw = t.serialize()?;
    let dep_ids_snw = t.frontmatter.depends_on.clone().unwrap_or_default();
    let mut raw_prompt_snw = format!("{}\n\n{raw}", agent_role_prefix(&wp.role, &id));
    if let Some(ref hint) = focus_hint {
        raw_prompt_snw.push_str(&format!("\n\n{hint}"));
    }
    let ticket_content = with_dependency_bundle(root, &dep_ids_snw, &config, raw_prompt_snw);
    let role_prefix = Some(agent_role_prefix(&wp.role, &id));

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
        agent_type: wp.agent.clone(),
        ticket_id: id.clone(),
        ticket_branch: branch.clone(),
        worktree_path: wt_display.clone(),
        system_prompt_file: sys_file.clone(),
        user_message_file: msg_file.clone(),
        skip_permissions,
        profile: worker_profile_str,
        role_prefix,
        options: std::collections::HashMap::new(),
        model: wp.model.clone(),
        log_path: log_path.clone(),
        container: wp.container.clone(),
        extra_env: wp.env.clone(),
        root: root.to_path_buf(),
        keychain: config.workers.keychain.clone(),
        current_state: t.frontmatter.state.clone(),
        command: Some(wp.agent.clone()),
    };
    if should_check_claude_compat(root, &wp.agent) {
        check_output_format_supported(&wp.agent)?;
    }
    let child = spawn_worker(&ctx, &wp.agent, root)?;
    let pid = child.id();

    let denial_ctx = if wp.agent == "claude" {
        Some((log_path.clone(), wt_display.clone(), id.clone()))
    } else {
        None
    };
    let managed = ManagedChild {
        inner: child,
        temp_files: vec![sys_file, msg_file],
        denial_ctx,
    };

    let pid_path = wt_display.join(".apm-worker.pid");
    write_pid_file(&pid_path, pid, &id)?;

    messages.push(format!("Worker spawned: PID={pid}, log={}", log_path.display()));
    messages.push(format!("Agent name: {worker_name}"));

    Ok(Some((id, epic_id, managed, pid_path)))
}

/// If the ticket has dependencies, prepend a dependency context bundle to the
/// worker prompt content.  Tickets with no dependencies are unchanged.
pub(crate) fn with_dependency_bundle(root: &Path, depends_on: &[String], config: &Config, content: String) -> String {
    if depends_on.is_empty() {
        return content;
    }
    let bundle = crate::context::build_dependency_bundle(root, depends_on, config);
    if bundle.is_empty() {
        return content;
    }
    format!("{bundle}\n{content}")
}

pub fn build_user_message(
    root: &Path,
    ticket: &crate::ticket::Ticket,
    depends_on: &[String],
    role: &str,
    config: &Config,
) -> Result<String> {
    let content = ticket.serialize()?;
    let id = &ticket.frontmatter.id;
    let raw = format!("{}\n\n{content}", agent_role_prefix(role, id));
    Ok(with_dependency_bundle(root, depends_on, config, raw))
}


pub(crate) fn resolve_builtin_instructions(agent: &str, role: &str) -> Option<&'static str> {
    match (agent, role) {
        ("claude", "worker") => Some(DEFAULT_WORKER_DEFAULT),
        ("default", "worker") => Some(DEFAULT_WORKER_DEFAULT),
        ("claude", "spec-writer") => Some(DEFAULT_SPEC_WRITER_DEFAULT),
        ("mock-happy", "worker") => Some(MOCK_HAPPY_WORKER_DEFAULT),
        ("mock-happy", "spec-writer") => Some(MOCK_HAPPY_SPEC_WRITER_DEFAULT),
        ("mock-sad", "worker") => Some(MOCK_SAD_WORKER_DEFAULT),
        ("mock-sad", "spec-writer") => Some(MOCK_SAD_SPEC_WRITER_DEFAULT),
        ("mock-random", "worker") => Some(MOCK_RANDOM_WORKER_DEFAULT),
        ("mock-random", "spec-writer") => Some(MOCK_RANDOM_SPEC_WRITER_DEFAULT),
        ("debug", "worker") => Some(DEBUG_WORKER_DEFAULT),
        ("debug", "spec-writer") => Some(DEBUG_SPEC_WRITER_DEFAULT),
        (_, "main-agent") => Some(DEFAULT_MAIN_AGENT_MD),
        _ => None,
    }
}

pub(crate) struct PromptProvenance {
    pub layer1_role: Option<String>,
    pub layer2_path: Option<String>,
    pub winner: ProvenanceEntry,
    pub skipped: Vec<ProvenanceEntry>,
}

pub(crate) struct ProvenanceEntry {
    pub level: u8,
    pub label: &'static str,
    pub source: String,
}

const LEVEL_LABELS: [&str; 3] = [
    "per-agent file",
    "claude-fallback file",
    "built-in default",
];

pub(crate) fn build_system_prompt(
    root: &Path,
    project_file: Option<&Path>,
    agent: &str,
    role: &str,
) -> Result<String> {
    // Layer 1: APM system knowledge (always present, scoped to role)
    let layer1 = crate::instructions::generate(root, Some(role), &[])?;

    // Layer 2: project context file (absent when not configured or path is empty)
    let layer2: Option<String> = if let Some(path) = project_file {
        if path.as_os_str().is_empty() {
            None
        } else {
            let content = std::fs::read_to_string(root.join(path))
                .map_err(|_| anyhow::anyhow!("agents.project: file not found: {}", path.display()))?;
            Some(content)
        }
    } else {
        None
    };

    // Layer 3: role-file cascade
    let layer3 = build_system_prompt_body(root, agent, role)?;

    // Compose layers joined by a single blank line
    let mut result = layer1.trim_end().to_owned();
    if let Some(ref l2) = layer2 {
        result.push_str("\n\n");
        result.push_str(l2.trim_end());
    }
    result.push_str("\n\n");
    result.push_str(layer3.trim_end());

    Ok(result)
}

fn build_system_prompt_body(root: &Path, agent: &str, role: &str) -> Result<String> {
    // Level 0: .apm/agents/<agent>/apm.<role>.md
    let per_agent = root.join(format!(".apm/agents/{agent}/apm.{role}.md"));
    if per_agent.exists() {
        if let Ok(content) = std::fs::read_to_string(&per_agent) {
            return Ok(content);
        }
    }
    // Level 1: .apm/agents/claude/apm.<role>.md (fallback for non-claude agents)
    //          .apm/agents/default/apm.<role>.md (backward compat — pre-rename)
    if agent != "claude" {
        let claude_file = root.join(format!(".apm/agents/claude/apm.{role}.md"));
        if claude_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&claude_file) {
                return Ok(content);
            }
        }
    }
    if agent != "default" {
        let default_file = root.join(format!(".apm/agents/default/apm.{role}.md"));
        if default_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&default_file) {
                return Ok(content);
            }
        }
    }
    // Level 2: built-in default
    if let Some(s) = resolve_builtin_instructions(agent, role) {
        return Ok(s.to_string());
    }
    // Level 3: hard error
    bail!(
        "no instructions found for agent '{agent}' role '{role}': \
         add .apm/agents/{agent}/apm.{role}.md or .apm/agents/claude/apm.{role}.md"
    )
}

pub(crate) fn explain_system_prompt(
    root: &Path,
    project_file: Option<&Path>,
    agent: &str,
    role: &str,
) -> Result<PromptProvenance> {
    let layer1_role = Some(role.to_string());
    let layer2_path = project_file
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.display().to_string());

    let mut skipped: Vec<ProvenanceEntry> = Vec::new();

    // Level 0: per-agent file
    let per_agent_rel = format!(".apm/agents/{agent}/apm.{role}.md");
    let per_agent = root.join(&per_agent_rel);
    if per_agent.exists() {
        let winner = ProvenanceEntry { level: 0, label: LEVEL_LABELS[0], source: per_agent_rel };
        for i in 1usize..=2 {
            skipped.push(ProvenanceEntry { level: i as u8, label: LEVEL_LABELS[i], source: "not reached".to_string() });
        }
        return Ok(PromptProvenance { layer1_role, layer2_path, winner, skipped });
    }
    skipped.push(ProvenanceEntry {
        level: 0,
        label: LEVEL_LABELS[0],
        source: format!("file absent: {per_agent_rel}"),
    });

    // Level 1: .apm/agents/claude/apm.<role>.md fallback (for non-claude agents)
    if agent != "claude" {
        let claude_rel = format!(".apm/agents/claude/apm.{role}.md");
        let claude_file = root.join(&claude_rel);
        if claude_file.exists() {
            let winner = ProvenanceEntry {
                level: 1,
                label: LEVEL_LABELS[1],
                source: format!("{claude_rel} (claude fallback — {per_agent_rel} absent)"),
            };
            skipped.push(ProvenanceEntry { level: 2, label: LEVEL_LABELS[2], source: "not reached".to_string() });
            return Ok(PromptProvenance { layer1_role, layer2_path, winner, skipped });
        }
    }
    skipped.push(ProvenanceEntry { level: 1, label: LEVEL_LABELS[1], source: "none found".to_string() });

    // Level 2: built-in default
    if resolve_builtin_instructions(agent, role).is_some() {
        let winner = ProvenanceEntry {
            level: 2,
            label: LEVEL_LABELS[2],
            source: format!("built-in default ({agent}/{role})"),
        };
        return Ok(PromptProvenance { layer1_role, layer2_path, winner, skipped });
    }

    // Level 3: hard error
    bail!(
        "no instructions found for agent '{agent}' role '{role}': \
         add .apm/agents/{agent}/apm.{role}.md or .apm/agents/claude/apm.{role}.md"
    )
}

pub(crate) fn agent_role_prefix(role: &str, id: &str) -> String {
    let title: String = role.split('-')
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("-");
    format!("You are a {title} agent assigned to ticket #{id}.")
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
    use super::{build_system_prompt, agent_role_prefix, check_output_format_supported, apply_frontmatter_agent, ManagedChild};
    use crate::config::WorkersConfig;
    use std::collections::HashMap;

    // --- resolve_worker_profile ---

    #[test]
    fn parse_worker_profile_valid() {
        let (agent, role) = super::parse_worker_profile("claude/spec-writer").unwrap();
        assert_eq!(agent, "claude");
        assert_eq!(role, "spec-writer");
    }

    #[test]
    fn parse_worker_profile_invalid_no_slash() {
        assert!(super::parse_worker_profile("claude").is_err());
    }

    #[test]
    fn parse_worker_profile_invalid_empty_parts() {
        assert!(super::parse_worker_profile("/worker").is_err());
        assert!(super::parse_worker_profile("claude/").is_err());
    }

    #[test]
    fn resolve_worker_profile_inherits_workers_env() {
        let mut workers = WorkersConfig::default();
        workers.env.insert("FOO".into(), "bar".into());
        let wp = super::resolve_worker_profile("claude/worker", &workers).unwrap();
        assert_eq!(wp.env.get("FOO").map(|s| s.as_str()), Some("bar"));
    }

    #[test]
    fn resolve_worker_profile_inherits_model() {
        let mut workers = WorkersConfig::default();
        workers.model = Some("sonnet".into());
        let wp = super::resolve_worker_profile("claude/worker", &workers).unwrap();
        assert_eq!(wp.model.as_deref(), Some("sonnet"));
    }

    // --- build_system_prompt ---

    #[test]
    fn build_system_prompt_uses_per_agent_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm/agents/claude")).unwrap();
        std::fs::write(p.join(".apm/agents/claude/apm.worker.md"), "PER AGENT WORKER").unwrap();
        let result = build_system_prompt(p, None, "claude", "worker").unwrap();
        assert!(result.contains("PER AGENT WORKER"), "layer 3 content missing: {result}");
    }

    #[test]
    fn build_system_prompt_falls_back_to_builtin_default() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        let result = build_system_prompt(p, None, "claude", "worker").unwrap();
        assert!(result.contains(super::DEFAULT_WORKER_DEFAULT.trim()), "built-in default not found in output");
    }

    #[test]
    fn build_system_prompt_falls_back_to_builtin_spec_writer() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        let result = build_system_prompt(p, None, "claude", "spec-writer").unwrap();
        assert!(result.contains(super::DEFAULT_SPEC_WRITER_DEFAULT.trim()), "built-in spec-writer default not found in output");
    }

    #[test]
    fn build_system_prompt_falls_back_to_claude_agent_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm/agents/claude")).unwrap();
        std::fs::write(p.join(".apm/agents/claude/apm.worker.md"), "CLAUDE WORKER CONTENT").unwrap();
        // "my-bot" has no per-agent file; should fall back to claude/
        let result = build_system_prompt(p, None, "my-bot", "worker").unwrap();
        assert!(result.contains("CLAUDE WORKER CONTENT"), "claude fallback content missing: {result}");
    }

    #[test]
    fn build_system_prompt_agent_file_takes_precedence_over_claude_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join(".apm/agents/my-bot")).unwrap();
        std::fs::create_dir_all(p.join(".apm/agents/claude")).unwrap();
        std::fs::write(p.join(".apm/agents/my-bot/apm.worker.md"), "AGENT SPECIFIC").unwrap();
        std::fs::write(p.join(".apm/agents/claude/apm.worker.md"), "CLAUDE CONTENT").unwrap();
        let result = build_system_prompt(p, None, "my-bot", "worker").unwrap();
        assert!(result.contains("AGENT SPECIFIC"), "agent-specific file should win: {result}");
        assert!(!result.contains("CLAUDE CONTENT"), "claude fallback should be skipped: {result}");
    }

    #[test]
    fn build_system_prompt_errors_for_unknown_agent() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        let result = build_system_prompt(p, None, "custom-bot", "worker");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("custom-bot"), "error should name the agent: {msg}");
        assert!(msg.contains("worker"), "error should name the role: {msg}");
    }

    // --- layer 2 (project file) tests ---

    #[test]
    fn agents_instructions_prepended_with_blank_line() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::write(p.join("prefix.md"), "PREFIX CONTENT\n").unwrap();
        let result = build_system_prompt(
            p,
            Some(std::path::Path::new("prefix.md")),
            "claude", "worker",
        ).unwrap();
        let layer1 = crate::instructions::generate(p, Some("worker"), &[]).unwrap();
        let expected = format!(
            "{}\n\nPREFIX CONTENT\n\n{}",
            layer1.trim_end(),
            super::DEFAULT_WORKER_DEFAULT.trim_end()
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn agents_instructions_none_is_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        let result = build_system_prompt(p, None, "claude", "worker").unwrap();
        let layer1 = crate::instructions::generate(p, Some("worker"), &[]).unwrap();
        let expected = format!("{}\n\n{}", layer1.trim_end(), super::DEFAULT_WORKER_DEFAULT.trim_end());
        assert_eq!(result, expected);
    }

    #[test]
    fn agents_instructions_empty_path_is_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        let result = build_system_prompt(
            p,
            Some(std::path::Path::new("")),
            "claude", "worker",
        ).unwrap();
        let layer1 = crate::instructions::generate(p, Some("worker"), &[]).unwrap();
        let expected = format!("{}\n\n{}", layer1.trim_end(), super::DEFAULT_WORKER_DEFAULT.trim_end());
        assert_eq!(result, expected);
    }

    #[test]
    fn agents_instructions_missing_file_is_hard_error() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        let result = build_system_prompt(
            p,
            Some(std::path::Path::new("no-such-file.md")),
            "claude", "worker",
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("agents.project"), "error should mention agents.project: {msg}");
        assert!(msg.contains("no-such-file.md"), "error should name the file: {msg}");
    }

    #[test]
    fn agents_instructions_trailing_whitespace_trimmed() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::write(p.join("prefix.md"), "PREFIX\n\n\n").unwrap();
        let result = build_system_prompt(
            p,
            Some(std::path::Path::new("prefix.md")),
            "claude", "worker",
        ).unwrap();
        let layer1 = crate::instructions::generate(p, Some("worker"), &[]).unwrap();
        let expected = format!(
            "{}\n\nPREFIX\n\n{}",
            layer1.trim_end(),
            super::DEFAULT_WORKER_DEFAULT.trim_end()
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn project_file_in_layer2() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::write(p.join("project.md"), "PROJECT CONTEXT\n").unwrap();
        let result = build_system_prompt(
            p,
            Some(std::path::Path::new("project.md")),
            "claude", "worker",
        ).unwrap();
        let layer1 = crate::instructions::generate(p, Some("worker"), &[]).unwrap();
        let expected = format!(
            "{}\n\nPROJECT CONTEXT\n\n{}",
            layer1.trim_end(),
            super::DEFAULT_WORKER_DEFAULT.trim_end()
        );
        assert_eq!(result, expected);
    }

    // --- agent_role_prefix ---

    #[test]
    fn agent_role_prefix_worker() {
        assert_eq!(
            agent_role_prefix("worker", "abc123"),
            "You are a Worker agent assigned to ticket #abc123."
        );
    }

    #[test]
    fn agent_role_prefix_spec_writer() {
        assert_eq!(
            agent_role_prefix("spec-writer", "abc123"),
            "You are a Spec-Writer agent assigned to ticket #abc123."
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
            agent_type: "test".to_string(),
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
            current_state: "in_progress".to_string(),
            command: None,
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
            agent_type: "test".to_string(),
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
            current_state: "in_progress".to_string(),
            command: None,
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
            agent_type: "test".to_string(),
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
            current_state: "in_progress".to_string(),
            command: None,
        };

        let wrapper = crate::wrapper::resolve_builtin("claude").unwrap();
        let child = wrapper.spawn(&ctx).unwrap();

        let mut managed = ManagedChild {
            inner: child,
            temp_files: vec![sys_file.clone(), msg_file.clone()],
            denial_ctx: None,
        };
        managed.inner.wait().unwrap();
        drop(managed);

        assert!(!sys_file.exists(), "sys_file should be removed after ManagedChild is dropped");
        assert!(!msg_file.exists(), "msg_file should be removed after ManagedChild is dropped");
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
            agent_type: "test".to_string(),
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
            current_state: "in_progress".to_string(),
            command: None,
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

    // --- mock wrapper integration tests ---

    fn find_apm_bin() -> Option<String> {
        // 1. Explicit override wins
        if let Ok(v) = std::env::var("APM_BIN") {
            if !v.is_empty() && std::path::Path::new(&v).exists() {
                return Some(v);
            }
        }
        // 2. Derive from the test binary path.
        //    current_exe() -> <workspace>/target/{profile}/deps/apm_core-<hash>
        //    two parents up -> <workspace>/target/{profile}/
        //    sibling "apm"  -> <workspace>/target/{profile}/apm
        if let Ok(exe) = std::env::current_exe() {
            if let Some(target_dir) = exe.parent().and_then(|p| p.parent()) {
                let candidate = target_dir.join("apm");
                if candidate.is_file() {
                    return Some(candidate.to_string_lossy().into_owned());
                }
            }
        }
        None
    }

    fn make_mock_project(root: &std::path::Path, ticket_state: &str, ticket_id: &str) {
        use std::fs;

        fs::create_dir_all(root.join(".apm/agents/claude")).unwrap();
        fs::create_dir_all(root.join("tickets")).unwrap();

        fs::write(root.join(".apm/config.toml"), r#"
[project]
name = "test-project"
default_branch = "main"

[workers]
agent = "mock-happy"

[tickets]
dir = "tickets"
"#).unwrap();

        fs::write(root.join(".apm/workflow.toml"), r#"
[[workflow.states]]
id = "in_design"
label = "In Design"
actionable = ["agent"]
instructions = ".apm/apm.spec-writer.md"

  [[workflow.states.transitions]]
  to = "specd"
  trigger = "manual"
  outcome = "success"

  [[workflow.states.transitions]]
  to = "closed"
  trigger = "manual"
  outcome = "cancelled"

[[workflow.states]]
id = "specd"
label = "Specd"
actionable = ["supervisor"]
satisfies_deps = true
worker_end = true

  [[workflow.states.transitions]]
  to = "in_progress"
  trigger = "manual"
  outcome = "success"

  [[workflow.states.transitions]]
  to = "closed"
  trigger = "manual"
  outcome = "cancelled"

[[workflow.states]]
id = "in_progress"
label = "In Progress"
instructions = ".apm/apm.worker.md"

  [[workflow.states.transitions]]
  to = "implemented"
  trigger = "manual"
  outcome = "success"

  [[workflow.states.transitions]]
  to = "closed"
  trigger = "manual"
  outcome = "cancelled"

[[workflow.states]]
id = "implemented"
label = "Implemented"
actionable = ["supervisor"]
satisfies_deps = true
worker_end = true
terminal = false

  [[workflow.states.transitions]]
  to = "closed"
  trigger = "manual"
  outcome = "cancelled"

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#).unwrap();

        fs::write(root.join(".apm/apm.worker.md"), "Worker instructions.").unwrap();
        fs::write(root.join(".apm/apm.spec-writer.md"), "Spec writer instructions.").unwrap();

        let ticket_content = format!(r#"+++
id = "{ticket_id}"
title = "Test Ticket"
state = "{ticket_state}"
priority = 0
effort = 5
risk = 3
author = "test"
owner = "test"
branch = "ticket/{ticket_id}-test"
created_at = "2026-01-01T00:00:00Z"
updated_at = "2026-01-01T00:00:00Z"
+++

## Spec

### Problem

Original problem.

### Acceptance criteria

- [ ] Some criterion

### Out of scope

Nothing.

### Approach

Some approach.

### Open questions

### Amendment requests

### Code review

## History

| When | From | To | By |
|------|------|----|----|
"#);
        fs::write(root.join(format!("tickets/{ticket_id}-test.md")), ticket_content).unwrap();

        std::process::Command::new("git")
            .arg("init")
            .current_dir(root)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(root)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(root)
            .output()
            .unwrap();
        // Create main branch with config files
        std::process::Command::new("git")
            .args(["add", ".apm"])
            .current_dir(root)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "initial commit", "--allow-empty"])
            .current_dir(root)
            .output()
            .unwrap();
        // Create the ticket branch and commit the ticket there
        let branch_name = format!("ticket/{ticket_id}-test");
        std::process::Command::new("git")
            .args(["checkout", "-b", &branch_name])
            .current_dir(root)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["add", &format!("tickets/{ticket_id}-test.md")])
            .current_dir(root)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", &format!("ticket({ticket_id}): created")])
            .current_dir(root)
            .output()
            .unwrap();
        // Switch back to main
        std::process::Command::new("git")
            .args(["checkout", "main"])
            .current_dir(root)
            .output()
            .unwrap();
    }

    fn make_wrapper_ctx_for_mock(
        project_root: &std::path::Path,
        ticket_id: &str,
        ticket_state: &str,
        apm_bin: &str,
        log_path: std::path::PathBuf,
    ) -> crate::wrapper::WrapperContext {
        let sys_file = crate::wrapper::write_temp_file("sys", "system prompt").unwrap();
        let msg_file = crate::wrapper::write_temp_file("msg", "ticket content").unwrap();
        let mut options = HashMap::new();
        options.insert("apm_bin".to_string(), apm_bin.to_string());
        crate::wrapper::WrapperContext {
            worker_name: "test-worker".to_string(),
            agent_type: "test".to_string(),
            ticket_id: ticket_id.to_string(),
            ticket_branch: format!("ticket/{ticket_id}-test"),
            worktree_path: project_root.to_path_buf(),
            system_prompt_file: sys_file,
            user_message_file: msg_file,
            skip_permissions: false,
            profile: "default".to_string(),
            role_prefix: None,
            options,
            model: None,
            log_path,
            container: None,
            extra_env: HashMap::new(),
            root: project_root.to_path_buf(),
            keychain: HashMap::new(),
            current_state: ticket_state.to_string(),
            command: None,
        }
    }

    #[test]
    fn mock_happy_spec_mode_transitions_to_specd() {
        let apm_bin = match find_apm_bin() { Some(b) => b, None => return };
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_mock_project(root, "in_design", "aaaa0001");
        let log_path = root.join("test-worker.log");
        let ctx = make_wrapper_ctx_for_mock(root, "aaaa0001", "in_design", &apm_bin, log_path.clone());
        let wrapper = crate::wrapper::resolve_builtin("mock-happy").unwrap();
        let mut child = wrapper.spawn(&ctx).unwrap();
        child.wait().unwrap();

        let log_content = std::fs::read_to_string(&log_path).unwrap_or_default();
        // Read ticket from the ticket branch (where apm commits changes)
        let ticket_from_branch = {
            let out = std::process::Command::new("git")
                .args(["show", "ticket/aaaa0001-test:tickets/aaaa0001-test.md"])
                .current_dir(root)
                .output()
                .unwrap();
            String::from_utf8_lossy(&out.stdout).to_string()
        };
        assert!(ticket_from_branch.contains("state = \"specd\""),
            "ticket should be in specd state\nticket_from_branch: {ticket_from_branch}\nlog: {log_content}");
        assert!(ticket_from_branch.contains("### Problem"),
            "ticket should have Problem section\n{ticket_from_branch}");
        assert!(ticket_from_branch.contains("effort = 1"),
            "effort should be 1\n{ticket_from_branch}");
        assert!(ticket_from_branch.contains("risk = 1"),
            "risk should be 1\n{ticket_from_branch}");
    }

    #[test]
    fn mock_happy_zero_success_transitions_returns_err() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        fs::create_dir_all(root.join(".apm/agents/claude")).unwrap();
        fs::create_dir_all(root.join("tickets")).unwrap();
        fs::write(root.join(".apm/config.toml"), r#"
[project]
name = "test"
default_branch = "main"
[workers]
agent = "mock-happy"
[tickets]
dir = "tickets"
"#).unwrap();
        fs::write(root.join(".apm/workflow.toml"), r#"
[[workflow.states]]
id = "in_design"
label = "In Design"
actionable = ["agent"]

  [[workflow.states.transitions]]
  to = "closed"
  trigger = "manual"
  outcome = "needs_input"

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#).unwrap();
        fs::write(root.join(".apm/apm.worker.md"), "instructions").unwrap();
        fs::write(root.join(".apm/apm.spec-writer.md"), "instructions").unwrap();
        let ticket_content = r#"+++
id = "aaaa0002"
title = "Test"
state = "in_design"
priority = 0
effort = 5
risk = 3
author = "test"
owner = "test"
branch = "ticket/aaaa0002-test"
created_at = "2026-01-01T00:00:00Z"
updated_at = "2026-01-01T00:00:00Z"
+++

## Spec

### Problem

### Acceptance criteria

### Out of scope

### Approach

## History

| When | From | To | By |
|------|------|----|----|
"#;
        fs::write(root.join("tickets/aaaa0002-test.md"), ticket_content).unwrap();
        std::process::Command::new("git").args(["init"]).current_dir(root).output().unwrap();
        std::process::Command::new("git").args(["config", "user.email", "t@t.com"]).current_dir(root).output().unwrap();
        std::process::Command::new("git").args(["config", "user.name", "T"]).current_dir(root).output().unwrap();
        std::process::Command::new("git").args(["add", "."]).current_dir(root).output().unwrap();
        std::process::Command::new("git").args(["commit", "-m", "init"]).current_dir(root).output().unwrap();

        let log_path = root.join("test.log");
        let sys_file = crate::wrapper::write_temp_file("sys", "sys").unwrap();
        let msg_file = crate::wrapper::write_temp_file("msg", "msg").unwrap();
        let ctx = crate::wrapper::WrapperContext {
            worker_name: "test".to_string(),
            agent_type: "test".to_string(),
            ticket_id: "aaaa0002".to_string(),
            ticket_branch: "ticket/aaaa0002-test".to_string(),
            worktree_path: root.to_path_buf(),
            system_prompt_file: sys_file,
            user_message_file: msg_file,
            skip_permissions: false,
            profile: "default".to_string(),
            role_prefix: None,
            options: HashMap::new(),
            model: None,
            log_path,
            container: None,
            extra_env: HashMap::new(),
            root: root.to_path_buf(),
            keychain: HashMap::new(),
            current_state: "in_design".to_string(),
            command: None,
        };
        let wrapper = crate::wrapper::resolve_builtin("mock-happy").unwrap();
        let result = wrapper.spawn(&ctx);
        assert!(result.is_err(), "mock-happy should return Err when no success transitions");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("no success-outcome transition"), "error should mention no success transition: {msg}");
    }

    #[test]
    fn mock_sad_transitions_to_non_success_state() {
        let apm_bin = match find_apm_bin() { Some(b) => b, None => return };
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_mock_project(root, "in_design", "aaaa0003");
        let log_path = root.join("test.log");
        let ctx = make_wrapper_ctx_for_mock(root, "aaaa0003", "in_design", &apm_bin, log_path.clone());
        let wrapper = crate::wrapper::resolve_builtin("mock-sad").unwrap();
        let mut child = wrapper.spawn(&ctx).unwrap();
        child.wait().unwrap();

        let log_content = std::fs::read_to_string(&log_path).unwrap_or_default();
        let out = std::process::Command::new("git")
            .args(["show", "ticket/aaaa0003-test:tickets/aaaa0003-test.md"])
            .current_dir(root)
            .output()
            .unwrap();
        let ticket_from_branch = String::from_utf8_lossy(&out.stdout).to_string();
        assert!(!ticket_from_branch.contains("state = \"specd\""),
            "mock-sad should NOT transition to specd\n{ticket_from_branch}\nlog: {log_content}");
        // Should have transitioned to some other state
        assert!(ticket_from_branch.contains("state = \"closed\"") || ticket_from_branch.contains("state = \"in_design\""),
            "mock-sad should transition to a non-success state\n{ticket_from_branch}\nlog: {log_content}");
    }

    #[test]
    fn mock_sad_seed_reproducibility() {
        let apm_bin = match find_apm_bin() { Some(b) => b, None => return };

        let run_mock_sad = |ticket_id: &str, seed: &str| -> String {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            make_mock_project(root, "in_design", ticket_id);
            let log_path = root.join("test.log");
            let mut options = HashMap::new();
            options.insert("apm_bin".to_string(), apm_bin.clone());
            options.insert("seed".to_string(), seed.to_string());
            let sys_file = crate::wrapper::write_temp_file("sys", "sys").unwrap();
            let msg_file = crate::wrapper::write_temp_file("msg", "msg").unwrap();
            let ctx = crate::wrapper::WrapperContext {
                worker_name: "test".to_string(),
                agent_type: "test".to_string(),
                ticket_id: ticket_id.to_string(),
                ticket_branch: format!("ticket/{ticket_id}-test"),
                worktree_path: root.to_path_buf(),
                system_prompt_file: sys_file,
                user_message_file: msg_file,
                skip_permissions: false,
                profile: "default".to_string(),
                role_prefix: None,
                options,
                model: None,
                log_path,
                container: None,
                extra_env: HashMap::new(),
                root: root.to_path_buf(),
                keychain: HashMap::new(),
                current_state: "in_design".to_string(),
            command: None,
            };
            let wrapper = crate::wrapper::resolve_builtin("mock-sad").unwrap();
            let mut child = wrapper.spawn(&ctx).unwrap();
            child.wait().unwrap();

            // Read state from ticket branch (where apm commits changes)
            let git_content = {
                let o = std::process::Command::new("git")
                    .args(["show", &format!("ticket/{ticket_id}-test:tickets/{ticket_id}-test.md")])
                    .current_dir(root)
                    .output()
                    .unwrap();
                String::from_utf8_lossy(&o.stdout).to_string()
            };
            for line in git_content.lines() {
                if line.starts_with("state = ") {
                    return line.to_string();
                }
            }
            "unknown".to_string()
        };

        let state1 = run_mock_sad("aaaa000a", "42");
        let state2 = run_mock_sad("aaaa000b", "42");
        assert_eq!(state1, state2, "mock-sad with same seed should pick same target state");
    }

    #[test]
    fn debug_does_not_change_state() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_mock_project(root, "in_design", "aaaa0005");
        let log_path = root.join("test.log");
        let sys_file = crate::wrapper::write_temp_file("sys", "debug-system-prompt-unique-text").unwrap();
        let msg_file = crate::wrapper::write_temp_file("msg", "debug-message").unwrap();
        let ctx = crate::wrapper::WrapperContext {
            worker_name: "test-worker".to_string(),
            agent_type: "test".to_string(),
            ticket_id: "aaaa0005".to_string(),
            ticket_branch: "ticket/aaaa0005-test".to_string(),
            worktree_path: root.to_path_buf(),
            system_prompt_file: sys_file,
            user_message_file: msg_file,
            skip_permissions: false,
            profile: "default".to_string(),
            role_prefix: None,
            options: HashMap::new(),
            model: None,
            log_path: log_path.clone(),
            container: None,
            extra_env: HashMap::new(),
            root: root.to_path_buf(),
            keychain: HashMap::new(),
            current_state: "in_design".to_string(),
            command: None,
        };
        let wrapper = crate::wrapper::resolve_builtin("debug").unwrap();
        let mut child = wrapper.spawn(&ctx).unwrap();
        child.wait().unwrap();

        // State should still be in_design (debug doesn't commit or transition)
        // Read from the ticket branch (HEAD of main won't have the ticket)
        let git_content = {
            let o = std::process::Command::new("git")
                .args(["show", "ticket/aaaa0005-test:tickets/aaaa0005-test.md"])
                .current_dir(root)
                .output()
                .unwrap();
            String::from_utf8_lossy(&o.stdout).to_string()
        };
        assert!(git_content.contains("state = \"in_design\""),
            "debug should not change ticket state\n{git_content}");

        // Log file should contain APM env vars and system prompt text
        let log_content = std::fs::read_to_string(&log_path).unwrap_or_default();
        assert!(log_content.contains("APM_TICKET_ID"),
            "log should contain APM_TICKET_ID\n{log_content}");
        assert!(log_content.contains("debug-system-prompt-unique-text"),
            "log should contain system prompt text\n{log_content}");
        assert!(log_content.contains("\"type\":\"tool_use\""),
            "log should contain tool_use JSONL\n{log_content}");
    }
}
