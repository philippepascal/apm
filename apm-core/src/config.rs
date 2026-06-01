use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// `free` — free-form prose. `tasks` — checkbox list (`- [ ] item`); supports `apm spec --mark` and `apm spec --add-task`. `qa` — question/answer pairs.
#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SectionType {
    Free,
    Tasks,
    Qa,
}

/// A single section in the ticket template.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TicketSection {
    /// Display name of the section (e.g. "Problem", "Approach").
    pub name: String,
    /// Rendering mode — `tasks` sections support `apm spec --mark` and `apm spec --add-task`; `free` is prose; `qa` is question/answer pairs.
    #[serde(rename = "type")]
    pub type_: SectionType,
    /// Whether the section must be non-empty before the ticket can transition out of in_design.
    #[serde(default)]
    pub required: bool,
    /// Hint text pre-filled into an empty section when a new ticket is created.
    #[serde(default)]
    pub placeholder: Option<String>,
}

/// Configuration for the sections that appear on every ticket, in order.
/// Defined in `.apm/ticket.toml` as `[[ticket.sections]]` blocks.
#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct TicketConfig {
    #[serde(default)]
    pub sections: Vec<TicketSection>,
}

/// Determines how a worker's branch is integrated as part of a state transition.
/// `pr`: open PR, fires on open not merge. `merge`: merge to target_branch directly.
/// `pull`: pull upstream into ticket branch. `pr_or_epic_merge`: recommended default — PR
/// on main, merge to epic branch when ticket belongs to an epic. `none`: no integration.
#[derive(Debug, Clone, PartialEq, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CompletionStrategy {
    Pr,
    Merge,
    Pull,
    #[serde(rename = "pr_or_epic_merge")]
    PrOrEpicMerge,
    #[default]
    None,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IsolationConfig {
    /// Glob patterns for paths that workers are allowed to read from outside the worktree.
    /// `~` is expanded to $HOME before matching. `**` matches any number of path components.
    /// Default: ["/etc/resolv.conf", "~/.gitconfig"]
    #[serde(default = "default_read_allow")]
    pub read_allow: Vec<String>,
    /// When true, a PreToolUse hook is installed in the worker's `.claude/settings.json`
    /// that blocks writes outside `APM_TICKET_WORKTREE`. Default: false.
    #[serde(default)]
    pub enforce_worktree_isolation: bool,
}

fn default_read_allow() -> Vec<String> {
    vec!["/etc/resolv.conf".to_string(), "~/.gitconfig".to_string()]
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            read_allow: default_read_allow(),
            enforce_worktree_isolation: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
pub struct LoggingConfig {
    /// When true, apm writes a debug log file for each run.
    #[serde(default)]
    pub enabled: bool,
    /// Path to the log file written when logging is enabled.
    pub file: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Default, JsonSchema)]
#[serde(default)]
pub struct GitHostConfig {
    /// Git host provider; currently only `github` is supported.
    pub provider: Option<String>,
    /// Repository path in `owner/name` form used for PR creation and collaborator lookup.
    pub repo: Option<String>,
    /// Environment variable name that holds the git host API token.
    pub token_env: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WorkersConfig {
    /// Docker image used to run worker agents; omit for local execution.
    pub container: Option<String>,
    /// Map of secret names to keychain item names resolved at worker launch time.
    #[serde(default)]
    pub keychain: std::collections::HashMap<String, String>,
    /// Environment variables injected into every worker process.
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    /// Default worker profile used when no state-level `worker_profile` is set.
    /// Format: `"agent/role"` (e.g. `"claude/coder"`).
    pub default: String,
    /// Model identifier passed to the worker agent (e.g. `"claude-sonnet-4-5"`).
    /// Can be overridden per-machine in `.apm/local.toml` under `[workers].model`.
    pub model: Option<String>,
}

impl Default for WorkersConfig {
    fn default() -> Self {
        Self {
            container: None,
            keychain: std::collections::HashMap::new(),
            env: std::collections::HashMap::new(),
            default: String::new(),
            model: None,
        }
    }
}


#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct WorkConfig {
    /// Default epic ID assigned when creating tickets with `apm new`.
    #[serde(default)]
    pub epic: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ServerConfig {
    /// Public-facing origin URL of the apm server, used in PR descriptions.
    #[serde(default = "default_server_origin")]
    pub origin: String,
    /// Internal URL the apm CLI uses to reach the apm server.
    #[serde(default = "default_server_url")]
    pub url: String,
}

fn default_server_origin() -> String {
    "http://localhost:3000".to_string()
}

fn default_server_url() -> String {
    "http://127.0.0.1:3000".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { origin: default_server_origin(), url: default_server_url() }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContextConfig {
    /// Maximum number of sibling tickets included in worker context bundles.
    #[serde(default = "default_epic_sibling_cap")]
    pub epic_sibling_cap: usize,
    /// Maximum byte size of the context bundle injected into worker prompts.
    #[serde(default = "default_epic_byte_cap")]
    pub epic_byte_cap: usize,
}

fn default_epic_sibling_cap() -> usize { 20 }
fn default_epic_byte_cap() -> usize { 8192 }

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            epic_sibling_cap: default_epic_sibling_cap(),
            epic_byte_cap: default_epic_byte_cap(),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Config {
    pub project: ProjectConfig,
    #[serde(default)]
    pub ticket: TicketConfig,
    #[serde(default)]
    pub tickets: TicketsConfig,
    #[serde(default)]
    pub workflow: WorkflowConfig,
    #[serde(default)]
    pub agents: AgentsConfig,
    #[serde(default)]
    pub worktrees: WorktreesConfig,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub workers: WorkersConfig,
    #[serde(default)]
    pub work: WorkConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub git_host: GitHostConfig,
    #[serde(default)]
    pub context: ContextConfig,
    #[serde(default)]
    pub isolation: IsolationConfig,
    /// Warnings generated during load (e.g. conflicting split/monolithic files).
    #[serde(skip)]
    pub load_warnings: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct WorkflowFile {
    pub(crate) workflow: WorkflowConfig,
}

#[derive(Deserialize)]
pub(crate) struct TicketFile {
    pub(crate) ticket: TicketConfig,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SyncConfig {
    /// When true, `apm sync` fetches all remote branches before checking state.
    #[serde(default = "default_true")]
    pub aggressive: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self { aggressive: true }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectConfig {
    /// Project name shown in prompts and the APM dashboard.
    pub name: String,
    /// Optional description of the project's purpose.
    #[serde(default)]
    pub description: String,
    /// Git branch used as the integration target for non-epic tickets.
    #[serde(default = "default_branch_main")]
    pub default_branch: String,
    /// Usernames allowed to own and work on tickets.
    #[serde(default)]
    pub collaborators: Vec<String>,
}

fn default_branch_main() -> String {
    "main".to_string()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TicketsConfig {
    /// Directory (relative to project root) where ticket files are stored.
    pub dir: PathBuf,
    #[serde(default)]
    pub sections: Vec<String>,
    /// Optional directory where closed tickets are moved on `apm close`.
    #[serde(default)]
    pub archive_dir: Option<PathBuf>,
}

impl Default for TicketsConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("tickets"),
            sections: Vec::new(),
            archive_dir: None,
        }
    }
}

/// Defines the ticket state machine and prioritization weights. Loaded from `.apm/workflow.toml` or the `[workflow]` section of `.apm/config.toml`.
#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct WorkflowConfig {
    /// Ordered list of ticket states. Users define their own state IDs and transition graph.
    #[serde(default)]
    pub states: Vec<StateConfig>,
    /// Weights used to rank tickets in `apm next` and `apm list`.
    #[serde(default)]
    pub prioritization: PrioritizationConfig,
}

/// Controls when reaching the parent state satisfies `depends_on` relationships on other tickets.
#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum SatisfiesDeps {
    /// `false` = this state never satisfies dependencies; `true` = it always does.
    Bool(bool),
    /// Satisfies only dependencies annotated with this string tag via `dep_requires`.
    Tag(String),
}

impl Default for SatisfiesDeps {
    fn default() -> Self { SatisfiesDeps::Bool(false) }
}

/// A single state in the workflow state machine.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StateConfig {
    /// Unique state identifier (e.g. `new`, `in_progress`). Used in ticket frontmatter and transition targets.
    pub id: String,
    /// Human-readable name shown in `apm list` and review prompts.
    pub label: String,
    /// Optional longer explanation of what this state means.
    #[serde(default)]
    pub description: String,
    /// When `true`, tickets in this state are considered done; no further transitions are expected.
    #[serde(default)]
    pub terminal: bool,
    /// When `true`, a worker finishing in this state is considered complete (used by the dispatcher to release the worker slot).
    #[serde(default)]
    pub worker_end: bool,
    /// Whether reaching this state satisfies `depends_on` relationships. `false` = never, `true` = always, a string tag = satisfies deps tagged with that string.
    #[serde(default)]
    pub satisfies_deps: SatisfiesDeps,
    /// Optional string tag that must appear in a dependency's `satisfies_deps` for it to count as satisfied.
    #[serde(default)]
    pub dep_requires: Option<String>,
    /// Worker profile for agents that operate in this state. Format: `"agent/role"`
    /// (e.g. `"claude/coder"`). Used by dispatch resolution before falling back to
    /// `[workers].default`.
    #[serde(default)]
    pub worker_profile: Option<String>,
    /// List of outgoing transitions from this state.
    #[serde(default)]
    pub transitions: Vec<TransitionConfig>,
}

/// A directed edge in the state machine: from the parent state to `to`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransitionConfig {
    /// Target state ID after this transition fires.
    pub to: String,
    /// Event or command that fires this transition (e.g. `close`, `approve`).
    #[serde(default)]
    pub trigger: String,
    /// Short label shown in the review prompt (e.g. `Approve for implementation`).
    #[serde(default)]
    pub label: String,
    /// Guidance shown in the editor header (e.g. `Add requests in ### Amendment requests`).
    #[serde(default)]
    pub hint: String,
    /// How the worker's branch is integrated before or after this transition. See `CompletionStrategy`.
    #[serde(default)]
    pub completion: CompletionStrategy,
    /// Markdown section heading the agent should focus on when acting on this transition.
    #[serde(default)]
    pub focus_section: Option<String>,
    /// Markdown section heading included as extra context for the agent.
    #[serde(default)]
    pub context_section: Option<String>,
    /// Optional warning message shown to the supervisor before the transition is confirmed.
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub on_failure: Option<String>,
    /// Semantic outcome of this transition from the worker's perspective.
    /// Recognised values: `success`, `needs_input`, `blocked`, `rejected`, `cancelled`.
    /// Custom values are accepted but treated as non-success by tooling.
    /// When omitted, `resolve_outcome` applies implicit defaults; see that function.
    #[serde(default)]
    pub outcome: Option<String>,
}

/// Weights used to compute the priority score for ticket selection in `apm next`.
#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct PrioritizationConfig {
    /// Multiplier applied to the ticket's `priority` field. Default: 10.0.
    #[serde(default = "default_priority_weight")]
    pub priority_weight: f64,
    /// Multiplier applied to the ticket's `effort` field (negative favours low-effort). Default: -2.0.
    #[serde(default = "default_effort_weight")]
    pub effort_weight: f64,
    /// Multiplier applied to the ticket's `risk` field (negative favours low-risk). Default: -1.0.
    #[serde(default = "default_risk_weight")]
    pub risk_weight: f64,
}

fn default_priority_weight() -> f64 { 10.0 }
fn default_effort_weight() -> f64 { -2.0 }
fn default_risk_weight() -> f64 { -1.0 }

/// Returns the effective outcome label for `transition`.
///
/// Uses the explicit `outcome` field when set; otherwise applies implicit defaults in order:
/// 1. `completion` strategy is set (non-`None`) → `"success"`
/// 2. `target_state.terminal` is true → `"cancelled"`
/// 3. Otherwise → `"needs_input"`
pub fn resolve_outcome<'a>(
    transition: &'a TransitionConfig,
    target_state: &StateConfig,
) -> &'a str {
    if let Some(ref o) = transition.outcome {
        return o.as_str();
    }
    if transition.completion != CompletionStrategy::None {
        return "success";
    }
    if target_state.terminal {
        return "cancelled";
    }
    "needs_input"
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AgentsConfig {
    /// Maximum number of worker agents allowed to run simultaneously.
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    /// Maximum workers allowed to work on the same epic at once.
    #[serde(default = "default_max_workers_per_epic")]
    pub max_workers_per_epic: usize,
    /// Maximum workers allowed to target the default branch simultaneously.
    #[serde(default = "default_max_workers_on_default")]
    pub max_workers_on_default: usize,
    /// Path to the project-context file injected as Layer 2 into every worker prompt.
    #[serde(default)]
    pub project: Option<PathBuf>,
    /// When true, workers may file side-note tickets during implementation.
    #[serde(default = "default_true")]
    pub side_tickets: bool,
    /// When true, workers skip Claude Code permission prompts.
    #[serde(default)]
    pub skip_permissions: bool,
}

fn default_max_concurrent() -> usize { 3 }
fn default_max_workers_per_epic() -> usize { 1 }
fn default_max_workers_on_default() -> usize { 1 }
fn default_true() -> bool { true }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorktreesConfig {
    /// Directory (relative to project root) where git worktrees are created.
    pub dir: PathBuf,
    /// Additional directories created inside each worker worktree.
    #[serde(default)]
    pub agent_dirs: Vec<String>,
}

impl Default for WorktreesConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("../worktrees"),
            agent_dirs: Vec::new(),
        }
    }
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            max_concurrent: default_max_concurrent(),
            max_workers_per_epic: default_max_workers_per_epic(),
            max_workers_on_default: default_max_workers_on_default(),
            project: None,
            side_tickets: true,
            skip_permissions: false,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct LocalConfig {
    #[serde(default)]
    pub workers: LocalWorkersOverride,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub github_token: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LocalWorkersOverride {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub model: Option<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

impl LocalConfig {
    pub fn load(root: &Path) -> Self {
        let local_path = root.join(".apm").join("local.toml");
        std::fs::read_to_string(&local_path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }
}

fn effective_github_token(local: &LocalConfig, git_host: &GitHostConfig) -> Option<String> {
    if let Some(ref t) = local.github_token {
        if !t.is_empty() {
            return Some(t.clone());
        }
    }
    if let Some(ref env_var) = git_host.token_env {
        if let Ok(t) = std::env::var(env_var) {
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.is_empty())
}

pub fn resolve_identity(repo_root: &Path) -> String {
    let local_path = repo_root.join(".apm").join("local.toml");
    let local: LocalConfig = std::fs::read_to_string(&local_path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();

    let config_path = repo_root.join(".apm").join("config.toml");
    let config: Option<Config> = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok());

    let git_host = config.as_ref().map(|c| &c.git_host).cloned().unwrap_or_default();
    if git_host.provider.is_some() {
        // git_host is the identity authority — do not fall back to local.toml
        if git_host.provider.as_deref() == Some("github") {
            if let Some(login) = crate::github::gh_username() {
                return login;
            }
            if let Some(token) = effective_github_token(&local, &git_host) {
                if let Ok(login) = crate::github::fetch_authenticated_user(&token) {
                    return login;
                }
            }
        }
        return "unassigned".to_string();
    }

    // No git_host — use local.toml username (local-only dev)
    if let Some(ref u) = local.username {
        if !u.is_empty() {
            return u.clone();
        }
    }
    "unassigned".to_string()
}

/// Returns the caller identity for this process.
///
/// This value is used in two places:
/// - Recorded as the acting party in ticket history entries.
/// - Compared against a ticket's `owner` field when filtering candidates
///   in `pick_next()` / `sorted_actionable()`. Tickets owned by another
///   identity are excluded from the pick set.
///
/// Resolution order: `APM_AGENT_TYPE` → `APM_AGENT_NAME` → `USER` →
/// `USERNAME` → `"apm"`.
///
/// `APM_AGENT_TYPE` is the agent type (e.g. "pi", "claude") and is preferred
/// because it produces clean, human-readable values in ticket history.
/// `APM_AGENT_NAME` is the unique per-run worker identifier
/// (e.g. "pi-0514-0628-7348") — kept as a fallback for backward compatibility
/// and for environments where only the legacy variable is set.
pub fn resolve_caller_name() -> String {
    std::env::var("APM_AGENT_TYPE")
        .or_else(|_| std::env::var("APM_AGENT_NAME"))
        .or_else(|_| std::env::var("USER"))
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "apm".to_string())
}

pub fn try_github_username(git_host: &GitHostConfig) -> Option<String> {
    if git_host.provider.as_deref() != Some("github") {
        return None;
    }
    if let Some(login) = crate::github::gh_username() {
        return Some(login);
    }
    let local = LocalConfig::default();
    let token = effective_github_token(&local, git_host)?;
    crate::github::fetch_authenticated_user(&token).ok()
}

pub fn resolve_collaborators(config: &Config, local: &LocalConfig) -> (Vec<String>, Vec<String>) {
    let mut warnings = Vec::new();
    if config.git_host.provider.as_deref() == Some("github") {
        if let Some(ref repo) = config.git_host.repo {
            if let Some(token) = effective_github_token(local, &config.git_host) {
                match crate::github::fetch_repo_collaborators(&token, repo) {
                    Ok(logins) => return (logins, warnings),
                    Err(e) => warnings.push(format!("apm: GitHub collaborators fetch failed: {e:#}")),
                }
            }
        }
    }
    (config.project.collaborators.clone(), warnings)
}

impl WorkersConfig {
    pub fn merge_local(&mut self, local: &LocalWorkersOverride) {
        for (k, v) in &local.env {
            self.env.insert(k.clone(), v.clone());
        }
        if let Some(ref m) = local.model {
            if !m.is_empty() {
                self.model = Some(m.clone());
            }
        }
    }
}

impl Config {
    /// Returns epic IDs that have reached the global `max_workers_per_epic` limit
    /// given the currently active worker epic assignments.
    pub fn blocked_epics(&self, active_epic_ids: &[Option<String>]) -> Vec<String> {
        let limit = self.agents.max_workers_per_epic;
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for eid in active_epic_ids.iter().filter_map(|e| e.as_deref()) {
            *counts.entry(eid).or_insert(0) += 1;
        }
        counts.into_iter()
            .filter(|(_, count)| *count >= limit)
            .map(|(eid, _)| eid.to_string())
            .collect()
    }

    /// Returns true when the default-branch worker slot is full.
    /// A value of 0 for `max_workers_on_default` means no additional cap.
    pub fn is_default_branch_blocked(&self, active_epic_ids: &[Option<String>]) -> bool {
        if self.agents.max_workers_on_default == 0 {
            return false;
        }
        let count = active_epic_ids.iter().filter(|e| e.is_none()).count();
        count >= self.agents.max_workers_on_default
    }

    /// States where `actor` can actively pick up / act on tickets.
    ///
    /// - `"agent"`: states that have at least one outgoing transition with `trigger = "command:start"`.
    /// - `"supervisor"`: non-terminal states that have no `command:start` outgoing transition.
    /// - Any other value: empty vec.
    pub fn actionable_states_for(&self, actor: &str) -> Vec<String> {
        match actor {
            "agent" => self.workflow.states.iter()
                .filter(|s| s.transitions.iter().any(|t| t.trigger == "command:start"))
                .map(|s| s.id.clone())
                .collect(),
            "supervisor" => self.workflow.states.iter()
                .filter(|s| !s.terminal
                    && !s.transitions.iter().any(|t| t.trigger == "command:start"))
                .map(|s| s.id.clone())
                .collect(),
            _ => vec![],
        }
    }

    pub fn terminal_state_ids(&self) -> std::collections::HashSet<String> {
        let mut ids: std::collections::HashSet<String> = self.workflow.states.iter()
            .filter(|s| s.terminal)
            .map(|s| s.id.clone())
            .collect();
        ids.insert("closed".to_string());
        ids
    }

    pub fn implementation_state_ids(&self) -> std::collections::HashSet<String> {
        let mut ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        // State-level path: states with a non-spec-writer worker_profile
        for state in &self.workflow.states {
            if let Some(ref wp) = state.worker_profile {
                if !wp.ends_with("/spec-writer") {
                    ids.insert(state.id.clone());
                }
            }
        }
        // Transition-based path (existing logic, additive)
        for state in &self.workflow.states {
            for t in &state.transitions {
                let dest_is_spec_writer = self.workflow.states.iter()
                    .find(|s| s.id == t.to)
                    .and_then(|s| s.worker_profile.as_deref())
                    .map(|wp| wp.ends_with("/spec-writer"))
                    .unwrap_or(false);
                let is_coder_start = t.trigger == "command:start" && !dest_is_spec_writer;
                let is_merge_completion = matches!(t.completion,
                    CompletionStrategy::Pr | CompletionStrategy::Merge | CompletionStrategy::PrOrEpicMerge);
                if is_coder_start || is_merge_completion {
                    ids.insert(t.to.clone());
                }
            }
        }
        ids
    }

    pub fn find_section(&self, name: &str) -> Option<&TicketSection> {
        self.ticket.sections.iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    pub fn has_section(&self, name: &str) -> bool {
        self.find_section(name).is_some()
    }

    pub fn load(repo_root: &Path) -> Result<Self> {
        let apm_dir = repo_root.join(".apm");
        let apm_dir_config = apm_dir.join("config.toml");
        let path = apm_dir_config;
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!(
                "cannot read {} -- run 'apm init' to initialise this repository",
                path.display()
            ))?;
        let mut config: Config = toml::from_str(&contents)
            .map_err(|e| {
                if e.to_string().contains("worker_profile") {
                    anyhow::anyhow!(
                        "{}: `transition.worker_profile` is no longer supported — \
                         move `worker_profile` to the state block instead",
                        path.display()
                    )
                } else {
                    anyhow::anyhow!("cannot parse {}: {}", path.display(), e)
                }
            })?;

        let workflow_path = apm_dir.join("workflow.toml");
        if workflow_path.exists() {
            let wf_contents = std::fs::read_to_string(&workflow_path)
                .with_context(|| format!("cannot read {}", workflow_path.display()))?;
            let wf: WorkflowFile = toml::from_str(&wf_contents)
                .map_err(|e| {
                    if e.to_string().contains("worker_profile") {
                        anyhow::anyhow!(
                            "{}: `transition.worker_profile` is no longer supported — \
                             move `worker_profile` to the state block instead",
                            workflow_path.display()
                        )
                    } else {
                        anyhow::anyhow!("cannot parse {}: {}", workflow_path.display(), e)
                    }
                })?;
            if !config.workflow.states.is_empty() {
                config.load_warnings.push(
                    "both .apm/workflow.toml and [workflow] in config.toml exist; workflow.toml takes precedence".into()
                );
            }
            config.workflow = wf.workflow;
        }

        let ticket_path = apm_dir.join("ticket.toml");
        if ticket_path.exists() {
            let tk_contents = std::fs::read_to_string(&ticket_path)
                .with_context(|| format!("cannot read {}", ticket_path.display()))?;
            let tk: TicketFile = toml::from_str(&tk_contents)
                .with_context(|| format!("cannot parse {}", ticket_path.display()))?;
            if !config.ticket.sections.is_empty() {
                config.load_warnings.push(
                    "both .apm/ticket.toml and [[ticket.sections]] in config.toml exist; ticket.toml takes precedence".into()
                );
            }
            config.ticket = tk.ticket;
        }

        let local_path = apm_dir.join("local.toml");
        if local_path.exists() {
            let local_contents = std::fs::read_to_string(&local_path)
                .with_context(|| format!("cannot read {}", local_path.display()))?;
            let local: LocalConfig = toml::from_str(&local_contents)
                .with_context(|| format!("cannot parse {}", local_path.display()))?;
            config.workers.merge_local(&local.workers);
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn ticket_section_full_parse() {
        let toml = r#"
name        = "Problem"
type        = "free"
required    = true
placeholder = "What is broken or missing?"
"#;
        let s: TicketSection = toml::from_str(toml).unwrap();
        assert_eq!(s.name, "Problem");
        assert_eq!(s.type_, SectionType::Free);
        assert!(s.required);
        assert_eq!(s.placeholder.as_deref(), Some("What is broken or missing?"));
    }

    #[test]
    fn ticket_section_minimal_parse() {
        let toml = r#"
name = "Open questions"
type = "qa"
"#;
        let s: TicketSection = toml::from_str(toml).unwrap();
        assert_eq!(s.name, "Open questions");
        assert_eq!(s.type_, SectionType::Qa);
        assert!(!s.required);
        assert!(s.placeholder.is_none());
    }

    #[test]
    fn section_type_all_variants() {
        #[derive(Deserialize)]
        struct W { t: SectionType }
        let free: W = toml::from_str("t = \"free\"").unwrap();
        assert_eq!(free.t, SectionType::Free);
        let tasks: W = toml::from_str("t = \"tasks\"").unwrap();
        assert_eq!(tasks.t, SectionType::Tasks);
        let qa: W = toml::from_str("t = \"qa\"").unwrap();
        assert_eq!(qa.t, SectionType::Qa);
    }

    #[test]
    fn completion_strategy_all_variants() {
        #[derive(Deserialize)]
        struct W { c: CompletionStrategy }
        let pr: W = toml::from_str("c = \"pr\"").unwrap();
        assert_eq!(pr.c, CompletionStrategy::Pr);
        let merge: W = toml::from_str("c = \"merge\"").unwrap();
        assert_eq!(merge.c, CompletionStrategy::Merge);
        let pull: W = toml::from_str("c = \"pull\"").unwrap();
        assert_eq!(pull.c, CompletionStrategy::Pull);
        let none: W = toml::from_str("c = \"none\"").unwrap();
        assert_eq!(none.c, CompletionStrategy::None);
        let prem: W = toml::from_str("c = \"pr_or_epic_merge\"").unwrap();
        assert_eq!(prem.c, CompletionStrategy::PrOrEpicMerge);
    }

    #[test]
    fn completion_strategy_default() {
        assert_eq!(CompletionStrategy::default(), CompletionStrategy::None);
    }

    #[test]
    fn transition_config_new_fields() {
        let toml = r#"
to              = "implemented"
trigger         = "manual"
completion      = "pr"
focus_section   = "Code review"
context_section = "Problem"
"#;
        let t: TransitionConfig = toml::from_str(toml).unwrap();
        assert_eq!(t.completion, CompletionStrategy::Pr);
        assert_eq!(t.focus_section.as_deref(), Some("Code review"));
        assert_eq!(t.context_section.as_deref(), Some("Problem"));
    }

    #[test]
    fn transition_config_new_fields_default() {
        let toml = r#"
to      = "ready"
trigger = "manual"
"#;
        let t: TransitionConfig = toml::from_str(toml).unwrap();
        assert_eq!(t.completion, CompletionStrategy::None);
        assert!(t.focus_section.is_none());
        assert!(t.context_section.is_none());
        assert!(t.outcome.is_none());
    }

    #[test]
    fn transition_worker_profile_rejected() {
        let toml = r#"
to             = "in_progress"
trigger        = "command:start"
worker_profile = "claude/coder"
"#;
        let result = toml::from_str::<TransitionConfig>(toml);
        assert!(result.is_err(), "worker_profile on transition must be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("worker_profile"),
            "error must name the field; got: {msg}"
        );
    }

    #[test]
    fn state_worker_profile_accepted() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"
"#;
        let result = toml::from_str::<Config>(toml);
        assert!(result.is_ok(), "worker_profile at state level must be accepted; err: {:?}", result.err());
        let config = result.unwrap();
        let state = config.workflow.states.iter().find(|s| s.id == "in_progress").unwrap();
        assert_eq!(state.worker_profile.as_deref(), Some("claude/coder"));
    }

    #[test]
    fn resolve_outcome_explicit_override() {
        let t: TransitionConfig = toml::from_str(r#"
to      = "ammend"
outcome = "rejected"
"#).unwrap();
        let s: StateConfig = toml::from_str(r#"
id    = "ammend"
label = "Ammend"
"#).unwrap();
        assert_eq!(super::resolve_outcome(&t, &s), "rejected");
    }

    #[test]
    fn resolve_outcome_implicit_success() {
        let t: TransitionConfig = toml::from_str(r#"
to         = "implemented"
completion = "merge"
"#).unwrap();
        let s: StateConfig = toml::from_str(r#"
id    = "implemented"
label = "Implemented"
"#).unwrap();
        assert_eq!(super::resolve_outcome(&t, &s), "success");
    }

    #[test]
    fn resolve_outcome_implicit_cancelled() {
        let t: TransitionConfig = toml::from_str(r#"
to = "closed"
"#).unwrap();
        let s: StateConfig = toml::from_str(r#"
id       = "closed"
label    = "Closed"
terminal = true
"#).unwrap();
        assert_eq!(super::resolve_outcome(&t, &s), "cancelled");
    }

    #[test]
    fn resolve_outcome_implicit_needs_input() {
        let t: TransitionConfig = toml::from_str(r#"
to = "blocked"
"#).unwrap();
        let s: StateConfig = toml::from_str(r#"
id    = "blocked"
label = "Blocked"
"#).unwrap();
        assert_eq!(super::resolve_outcome(&t, &s), "needs_input");
    }

    #[test]
    fn workers_config_parses() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
container = "apm-worker:latest"
default = "claude/coder"

[workers.keychain]
ANTHROPIC_API_KEY = "anthropic-api-key"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.workers.container.as_deref(), Some("apm-worker:latest"));
        assert_eq!(config.workers.keychain.get("ANTHROPIC_API_KEY").map(|s| s.as_str()), Some("anthropic-api-key"));
    }

    #[test]
    fn workers_config_default() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.workers.container.is_none());
        assert!(config.workers.keychain.is_empty());
        assert!(config.workers.default.is_empty());
        assert!(config.workers.model.is_none());
        assert!(config.workers.env.is_empty());
    }

    #[test]
    fn workers_config_default_field() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
default = "claude/coder"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.workers.default, "claude/coder");
    }

    #[test]
    fn workers_default_missing_fails_parse() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
container = "apm-worker:latest"
"#;
        let result = toml::from_str::<Config>(toml);
        assert!(result.is_err(), "expected parse error when [workers] has no default key");
    }

    #[test]
    fn workers_config_env_field() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
default = "claude/coder"

[workers.env]
CUSTOM_VAR = "value"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.workers.env.get("CUSTOM_VAR").map(|s| s.as_str()), Some("value"));
    }

    #[test]
    fn local_config_parses() {
        let toml = r#"
[workers]
command = "aider"
model = "gpt-4"

[workers.env]
OPENAI_API_KEY = "sk-test"
"#;
        let local: LocalConfig = toml::from_str(toml).unwrap();
        assert_eq!(local.workers.command.as_deref(), Some("aider"));
        assert_eq!(local.workers.model.as_deref(), Some("gpt-4"));
        assert_eq!(local.workers.env.get("OPENAI_API_KEY").map(|s| s.as_str()), Some("sk-test"));
        assert!(local.workers.args.is_none());
    }

    #[test]
    fn merge_local_extends_env() {
        let mut wc = WorkersConfig::default();
        let local = LocalWorkersOverride {
            command: None,
            args: None,
            model: None,
            env: [("KEY".to_string(), "val".to_string())].into(),
        };
        wc.merge_local(&local);
        assert_eq!(wc.env.get("KEY").map(|s| s.as_str()), Some("val"));
    }

    #[test]
    fn agents_skip_permissions_parses_and_defaults() {
        let base = "[project]\nname = \"test\"\n[tickets]\ndir = \"tickets\"\n";

        // absent → false
        let config: Config = toml::from_str(base).unwrap();
        assert!(!config.agents.skip_permissions, "absent skip_permissions should default to false");

        // [agents] section without the key → still false
        let with_agents = format!("{base}[agents]\n");
        let config: Config = toml::from_str(&with_agents).unwrap();
        assert!(!config.agents.skip_permissions, "[agents] without skip_permissions should default to false");

        // explicit true
        let explicit_true = format!("{base}[agents]\nskip_permissions = true\n");
        let config: Config = toml::from_str(&explicit_true).unwrap();
        assert!(config.agents.skip_permissions, "explicit skip_permissions = true should be true");

        // explicit false
        let explicit_false = format!("{base}[agents]\nskip_permissions = false\n");
        let config: Config = toml::from_str(&explicit_false).unwrap();
        assert!(!config.agents.skip_permissions, "explicit skip_permissions = false should be false");
    }

    #[test]
    fn actionable_states_for_agent_includes_ready() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"

[[workflow.states]]
id = "in_progress"
label = "In Progress"

[[workflow.states]]
id = "specd"
label = "Specd"

  [[workflow.states.transitions]]
  to      = "ready"
  trigger = "manual"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let states = config.actionable_states_for("agent");
        assert!(states.contains(&"ready".to_string()));
        assert!(!states.contains(&"specd".to_string()));
        assert!(!states.contains(&"in_progress".to_string()));
    }

    #[test]
    fn actionable_states_for_supervisor_includes_in_design() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id = "in_design"
label = "In Design"

  [[workflow.states.transitions]]
  to = "specd"
  trigger = "manual"

[[workflow.states]]
id = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to = "in_progress"
  trigger = "command:start"

[[workflow.states]]
id = "in_progress"
label = "In Progress"
terminal = true
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let states = config.actionable_states_for("supervisor");
        assert!(states.contains(&"in_design".to_string()),
            "in_design has no command:start outgoing; must be supervisor-actionable");
        assert!(!states.contains(&"ready".to_string()),
            "ready has command:start outgoing; must not be supervisor-actionable");
        assert!(!states.contains(&"in_progress".to_string()),
            "terminal states must not be supervisor-actionable");
    }

    #[test]
    fn actionable_states_for_unknown_actor_returns_empty() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.actionable_states_for("engineer").is_empty());
    }

    #[test]
    fn state_config_deny_unknown_fields_rejects_actionable() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id         = "ready"
label      = "Ready"
actionable = ["agent"]
"#;
        let result: Result<Config, _> = toml::from_str(toml);
        assert!(result.is_err(), "actionable field must be rejected by deny_unknown_fields");
    }

    #[test]
    fn work_epic_parses() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[work]
epic = "ab12cd34"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.work.epic.as_deref(), Some("ab12cd34"));
    }

    #[test]
    fn work_config_defaults_to_none() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.work.epic.is_none());
    }

    #[test]
    fn implementation_state_ids_coder_start_and_merge_completion() {
        // A workflow with a coder command:start to in_progress and a pr_or_epic_merge
        // to implemented should return {"in_progress", "implemented"}.
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

  [[workflow.states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  completion = "pr_or_epic_merge"

[[workflow.states]]
id    = "implemented"
label = "Implemented"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ids = config.implementation_state_ids();
        let expected: std::collections::HashSet<String> =
            ["in_progress", "implemented"].iter().map(|s| s.to_string()).collect();
        assert_eq!(ids, expected);
    }

    #[test]
    fn implementation_state_ids_none_completion_still_nonempty_via_coder_start() {
        // A workflow where in_progress->implemented uses completion="none" must still
        // yield a non-empty set because the coder command:start signal provides in_progress.
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

  [[workflow.states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  completion = "none"

[[workflow.states]]
id    = "implemented"
label = "Implemented"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ids = config.implementation_state_ids();
        assert_eq!(ids, ["in_progress".to_string()].into_iter().collect::<std::collections::HashSet<_>>());
    }

    #[test]
    fn implementation_state_ids_no_coder_start_uses_merge_completion() {
        // A workflow with no command:start but a merge-completion transition to "shipped"
        // must return {"shipped"}.
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

  [[workflow.states.transitions]]
  to         = "shipped"
  trigger    = "manual"
  completion = "merge"

[[workflow.states]]
id    = "shipped"
label = "Shipped"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ids = config.implementation_state_ids();
        assert_eq!(ids, ["shipped".to_string()].into_iter().collect::<std::collections::HashSet<_>>());
    }

    #[test]
    fn implementation_state_ids_command_start_no_profile_treated_as_coder() {
        // A command:start transition with no worker_profile must be treated as a
        // coder entry (None profile counts as non-spec-writer).
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ids = config.implementation_state_ids();
        assert_eq!(ids, ["in_progress".to_string()].into_iter().collect::<std::collections::HashSet<_>>());
    }

    #[test]
    fn implementation_state_ids_spec_writer_start_excluded() {
        // A command:start to a state with worker_profile = "claude/spec-writer" must NOT
        // contribute an implementation state.
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to      = "in_design"
  trigger = "command:start"

[[workflow.states]]
id             = "in_design"
label          = "In Design"
worker_profile = "claude/spec-writer"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ids = config.implementation_state_ids();
        assert!(ids.is_empty(), "spec-writer start must not count as an implementation state");
    }

    #[test]
    fn implementation_state_ids_order_invariant() {
        // Building the workflow with states in two different orders must yield the
        // same implementation_state_ids set.
        let toml_v1 = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

  [[workflow.states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  completion = "pr_or_epic_merge"

[[workflow.states]]
id    = "implemented"
label = "Implemented"
"#;
        // Same states, reversed order.
        let toml_v2 = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "implemented"
label = "Implemented"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

  [[workflow.states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  completion = "pr_or_epic_merge"

[[workflow.states]]
id    = "ready"
label = "Ready"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
"#;
        let c1: Config = toml::from_str(toml_v1).unwrap();
        let c2: Config = toml::from_str(toml_v2).unwrap();
        assert_eq!(
            c1.implementation_state_ids(),
            c2.implementation_state_ids(),
            "implementation_state_ids must be invariant to state list order"
        );
    }

    #[test]
    fn state_worker_profile_parses() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let state = config.workflow.states.iter().find(|s| s.id == "in_progress").unwrap();
        assert_eq!(state.worker_profile.as_deref(), Some("claude/coder"));
    }

    #[test]
    fn implementation_state_ids_state_worker_profile_preferred() {
        // A workflow where in_progress has state-level worker_profile = "claude/coder"
        // but no command:start transition — must still appear in implementation_state_ids.
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

  [[workflow.states.transitions]]
  to      = "implemented"
  trigger = "manual"

[[workflow.states]]
id    = "implemented"
label = "Implemented"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ids = config.implementation_state_ids();
        assert!(ids.contains(&"in_progress".to_string()),
            "in_progress must appear when state has worker_profile = claude/coder; got: {:?}", ids);
    }

    #[test]
    fn sync_aggressive_defaults_to_true() {
        let base = "[project]\nname = \"test\"\n[tickets]\ndir = \"tickets\"\n";

        // no [sync] section
        let config: Config = toml::from_str(base).unwrap();
        assert!(config.sync.aggressive, "no [sync] section should default to true");

        // [sync] section with no aggressive key
        let with_sync = format!("{base}[sync]\n");
        let config: Config = toml::from_str(&with_sync).unwrap();
        assert!(config.sync.aggressive, "[sync] without aggressive key should default to true");

        // explicit false
        let explicit_false = format!("{base}[sync]\naggressive = false\n");
        let config: Config = toml::from_str(&explicit_false).unwrap();
        assert!(!config.sync.aggressive, "explicit aggressive = false should be false");

        // explicit true
        let explicit_true = format!("{base}[sync]\naggressive = true\n");
        let config: Config = toml::from_str(&explicit_true).unwrap();
        assert!(config.sync.aggressive, "explicit aggressive = true should be true");
    }

    #[test]
    fn collaborators_parses() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.project.collaborators, vec!["alice", "bob"]);
    }

    #[test]
    fn collaborators_defaults_empty() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.project.collaborators.is_empty());
    }

    #[test]
    fn resolve_identity_returns_username_when_present() {
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("local.toml"), "username = \"alice\"\n").unwrap();
        assert_eq!(resolve_identity(tmp.path()), "alice");
    }

    #[test]
    fn resolve_identity_returns_unassigned_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(resolve_identity(tmp.path()), "unassigned");
    }

    #[test]
    fn resolve_identity_returns_unassigned_when_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("local.toml"), "username = \"\"\n").unwrap();
        assert_eq!(resolve_identity(tmp.path()), "unassigned");
    }

    #[test]
    fn resolve_identity_returns_unassigned_when_username_key_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("local.toml"), "[workers]\ncommand = \"claude\"\n").unwrap();
        assert_eq!(resolve_identity(tmp.path()), "unassigned");
    }

    #[test]
    fn local_config_username_parses() {
        let toml = r#"
username = "bob"
"#;
        let local: LocalConfig = toml::from_str(toml).unwrap();
        assert_eq!(local.username.as_deref(), Some("bob"));
    }

    #[test]
    fn local_config_username_defaults_none() {
        let local: LocalConfig = toml::from_str("").unwrap();
        assert!(local.username.is_none());
    }

    #[test]
    fn server_config_defaults() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.origin, "http://localhost:3000");
    }

    #[test]
    fn server_config_custom_origin() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[server]
origin = "https://apm.example.com"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.origin, "https://apm.example.com");
    }

    #[test]
    fn git_host_config_parses() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "owner/name"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.git_host.provider.as_deref(), Some("github"));
        assert_eq!(config.git_host.repo.as_deref(), Some("owner/name"));
    }

    #[test]
    fn git_host_config_absent_defaults_none() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.git_host.provider.is_none());
        assert!(config.git_host.repo.is_none());
    }

    #[test]
    fn local_config_github_token_parses() {
        let toml = r#"github_token = "ghp_abc123""#;
        let local: LocalConfig = toml::from_str(toml).unwrap();
        assert_eq!(local.github_token.as_deref(), Some("ghp_abc123"));
    }

    #[test]
    fn local_config_github_token_absent_defaults_none() {
        let local: LocalConfig = toml::from_str("").unwrap();
        assert!(local.github_token.is_none());
    }

    #[test]
    fn tickets_archive_dir_parses() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
archive_dir = "archive/tickets"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.tickets.archive_dir.as_deref(),
            Some(std::path::Path::new("archive/tickets"))
        );
    }

    #[test]
    fn tickets_archive_dir_absent_defaults_none() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.tickets.archive_dir.is_none());
    }

    #[test]
    fn agents_max_workers_per_epic_defaults_to_one() {
        let toml = "[project]\nname = \"test\"\n\n[tickets]\ndir = \"tickets\"\n";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.agents.max_workers_per_epic, 1);
    }

    #[test]
    fn blocked_epics_global_limit_one() {
        let toml = "[project]\nname = \"test\"\n\n[tickets]\ndir = \"tickets\"\n";
        let config: Config = toml::from_str(toml).unwrap();
        // limit=1, one active worker in epic A → epic A is blocked
        let active = vec![Some("epicA".to_string())];
        let blocked = config.blocked_epics(&active);
        assert!(blocked.contains(&"epicA".to_string()));
    }

    #[test]
    fn blocked_epics_global_limit_two() {
        let toml = "[project]\nname = \"test\"\n\n[tickets]\ndir = \"tickets\"\n\n[agents]\nmax_workers_per_epic = 2\n";
        let config: Config = toml::from_str(toml).unwrap();
        // limit=2, one active worker in epic A → epic A is NOT blocked
        let active = vec![Some("epicA".to_string())];
        let blocked = config.blocked_epics(&active);
        assert!(!blocked.contains(&"epicA".to_string()));
    }

    #[test]
    fn default_branch_not_blocked_when_no_active_non_epic_workers() {
        let base = "[project]\nname = \"test\"\n\n[tickets]\ndir = \"tickets\"\n";
        let config: Config = toml::from_str(base).unwrap();
        assert_eq!(config.agents.max_workers_on_default, 1);
        // limit=1, 0 active non-epic workers → not blocked
        let active: Vec<Option<String>> = vec![];
        assert!(!config.is_default_branch_blocked(&active));
    }

    #[test]
    fn default_branch_blocked_when_one_active_non_epic_worker_and_limit_one() {
        let base = "[project]\nname = \"test\"\n\n[tickets]\ndir = \"tickets\"\n";
        let config: Config = toml::from_str(base).unwrap();
        // limit=1, 1 active non-epic worker → blocked
        let active = vec![None];
        assert!(config.is_default_branch_blocked(&active));
    }

    #[test]
    fn default_branch_not_blocked_when_limit_zero() {
        let toml = "[project]\nname = \"test\"\n\n[tickets]\ndir = \"tickets\"\n\n[agents]\nmax_workers_on_default = 0\n";
        let config: Config = toml::from_str(toml).unwrap();
        // limit=0, any number of active non-epic workers → not blocked
        let active = vec![None, None, None];
        assert!(!config.is_default_branch_blocked(&active));
    }

    #[test]
    fn default_branch_not_blocked_when_all_workers_are_epic_linked() {
        let base = "[project]\nname = \"test\"\n\n[tickets]\ndir = \"tickets\"\n";
        let config: Config = toml::from_str(base).unwrap();
        // limit=1, all active workers are epic-linked → not blocked
        let active = vec![Some("epicA".to_string()), Some("epicB".to_string())];
        assert!(!config.is_default_branch_blocked(&active));
    }

    #[test]
    fn prefers_apm_agent_type() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("APM_AGENT_NAME");
        std::env::set_var("APM_AGENT_TYPE", "explicit-type");
        assert_eq!(resolve_caller_name(), "explicit-type");
        std::env::remove_var("APM_AGENT_TYPE");
    }

    #[test]
    fn prefers_apm_agent_name() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("APM_AGENT_TYPE");
        std::env::set_var("APM_AGENT_NAME", "explicit-agent");
        assert_eq!(resolve_caller_name(), "explicit-agent");
        std::env::remove_var("APM_AGENT_NAME");
    }

    #[test]
    fn falls_back_to_user() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("APM_AGENT_TYPE");
        std::env::remove_var("APM_AGENT_NAME");
        std::env::set_var("USER", "unix-user");
        std::env::remove_var("USERNAME");
        assert_eq!(resolve_caller_name(), "unix-user");
        std::env::remove_var("USER");
    }

    #[test]
    fn defaults_to_apm() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("APM_AGENT_TYPE");
        std::env::remove_var("APM_AGENT_NAME");
        std::env::remove_var("USER");
        std::env::remove_var("USERNAME");
        assert_eq!(resolve_caller_name(), "apm");
    }

}
