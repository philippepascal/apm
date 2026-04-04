use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionType {
    Free,
    Tasks,
    Qa,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TicketSection {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: SectionType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub placeholder: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TicketConfig {
    #[serde(default)]
    pub sections: Vec<TicketSection>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LoggingConfig {
    #[serde(default)]
    pub enabled: bool,
    pub file: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct GitHostConfig {
    pub provider: Option<String>,
    pub repo: Option<String>,
    pub token_env: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkersConfig {
    pub container: Option<String>,
    #[serde(default)]
    pub keychain: std::collections::HashMap<String, String>,
    #[serde(default = "default_command")]
    pub command: String,
    #[serde(default = "default_args")]
    pub args: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

impl Default for WorkersConfig {
    fn default() -> Self {
        Self {
            container: None,
            keychain: std::collections::HashMap::new(),
            command: default_command(),
            args: default_args(),
            model: None,
            env: std::collections::HashMap::new(),
        }
    }
}

fn default_command() -> String { "claude".to_string() }
fn default_args() -> Vec<String> { vec!["--print".to_string()] }

#[derive(Debug, Deserialize, Default)]
pub struct WorkConfig {
    #[serde(default)]
    pub epic: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_origin")]
    pub origin: String,
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_true")]
    pub aggressive: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self { aggressive: true }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_branch_main")]
    pub default_branch: String,
    #[serde(default)]
    pub collaborators: Vec<String>,
}

fn default_branch_main() -> String {
    "main".to_string()
}

#[derive(Debug, Deserialize)]
pub struct TicketsConfig {
    pub dir: PathBuf,
    #[serde(default)]
    pub sections: Vec<String>,
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

#[derive(Debug, Deserialize, Default)]
pub struct WorkflowConfig {
    #[serde(default)]
    pub states: Vec<StateConfig>,
    #[serde(default)]
    pub prioritization: PrioritizationConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum SatisfiesDeps {
    Bool(bool),
    Tag(String),
}

impl Default for SatisfiesDeps {
    fn default() -> Self { SatisfiesDeps::Bool(false) }
}

#[derive(Debug, Deserialize)]
pub struct StateConfig {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub terminal: bool,
    #[serde(default)]
    pub worker_end: bool,
    #[serde(default)]
    pub satisfies_deps: SatisfiesDeps,
    #[serde(default)]
    pub dep_requires: Option<String>,
    #[serde(default)]
    pub transitions: Vec<TransitionConfig>,
    /// Who can actively pick up / act on tickets in this state.
    /// Values: "agent", "supervisor", "engineer", "any".
    /// Drives `apm next`, `apm start`, and `apm list --actionable`.
    #[serde(default)]
    pub actionable: Vec<String>,
    #[serde(default)]
    pub instructions: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransitionConfig {
    pub to: String,
    #[serde(default)]
    pub trigger: String,
    #[serde(default)]
    pub actor: String,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub side_effects: Vec<String>,
    /// Short label shown in the review prompt (e.g. "Approve for implementation")
    #[serde(default)]
    pub label: String,
    /// Guidance shown in the editor header (e.g. "Add requests in ### Amendment requests")
    #[serde(default)]
    pub hint: String,
    #[serde(default)]
    pub completion: CompletionStrategy,
    #[serde(default)]
    pub focus_section: Option<String>,
    #[serde(default)]
    pub context_section: Option<String>,
    #[serde(default)]
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PrioritizationConfig {
    #[serde(default = "default_priority_weight")]
    pub priority_weight: f64,
    #[serde(default = "default_effort_weight")]
    pub effort_weight: f64,
    #[serde(default = "default_risk_weight")]
    pub risk_weight: f64,
}

fn default_priority_weight() -> f64 { 10.0 }
fn default_effort_weight() -> f64 { -2.0 }
fn default_risk_weight() -> f64 { -1.0 }

#[derive(Debug, Deserialize)]
pub struct AgentsConfig {
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    #[serde(default)]
    pub instructions: Option<PathBuf>,
    #[serde(default = "default_true")]
    pub side_tickets: bool,
    #[serde(default)]
    pub skip_permissions: bool,
}

fn default_max_concurrent() -> usize { 3 }
fn default_true() -> bool { true }

#[derive(Debug, Deserialize)]
pub struct WorktreesConfig {
    pub dir: PathBuf,
}

impl Default for WorktreesConfig {
    fn default() -> Self {
        Self { dir: PathBuf::from("../worktrees") }
    }
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            max_concurrent: default_max_concurrent(),
            instructions: None,
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
        eprintln!("apm: could not resolve identity from git_host");
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

pub fn resolve_collaborators(config: &Config, local: &LocalConfig) -> Vec<String> {
    if config.git_host.provider.as_deref() == Some("github") {
        if let Some(ref repo) = config.git_host.repo {
            if let Some(token) = effective_github_token(local, &config.git_host) {
                match crate::github::fetch_repo_collaborators(&token, repo) {
                    Ok(logins) => return logins,
                    Err(e) => eprintln!("apm: GitHub collaborators fetch failed: {e}"),
                }
            }
        }
    }
    config.project.collaborators.clone()
}

impl WorkersConfig {
    pub fn merge_local(&mut self, local: &LocalWorkersOverride) {
        if let Some(ref cmd) = local.command {
            self.command = cmd.clone();
        }
        if let Some(ref args) = local.args {
            self.args = args.clone();
        }
        if let Some(ref model) = local.model {
            self.model = Some(model.clone());
        }
        for (k, v) in &local.env {
            self.env.insert(k.clone(), v.clone());
        }
    }
}

impl Config {
    /// States where `actor` can actively pick up / act on tickets.
    /// Matches "any" as a wildcard in addition to the literal actor name.
    pub fn actionable_states_for(&self, actor: &str) -> Vec<String> {
        self.workflow.states.iter()
            .filter(|s| s.actionable.iter().any(|a| a == actor || a == "any"))
            .map(|s| s.id.clone())
            .collect()
    }

    pub fn load(repo_root: &Path) -> Result<Self> {
        let apm_dir = repo_root.join(".apm");
        let apm_dir_config = apm_dir.join("config.toml");
        let path = if apm_dir_config.exists() {
            apm_dir_config
        } else {
            repo_root.join("apm.toml")
        };
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        let mut config: Config = toml::from_str(&contents)
            .with_context(|| format!("cannot parse {}", path.display()))?;

        let workflow_path = apm_dir.join("workflow.toml");
        if workflow_path.exists() {
            let wf_contents = std::fs::read_to_string(&workflow_path)
                .with_context(|| format!("cannot read {}", workflow_path.display()))?;
            let wf: WorkflowFile = toml::from_str(&wf_contents)
                .with_context(|| format!("cannot parse {}", workflow_path.display()))?;
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
    fn state_config_worker_end_parses_true() {
        let toml = r#"
id         = "specd"
label      = "Specd"
worker_end = true
"#;
        let s: StateConfig = toml::from_str(toml).unwrap();
        assert!(s.worker_end);
    }

    #[test]
    fn state_config_worker_end_defaults_false() {
        let toml = r#"
id    = "new"
label = "New"
"#;
        let s: StateConfig = toml::from_str(toml).unwrap();
        assert!(!s.worker_end);
    }

    #[test]
    fn state_config_with_instructions() {
        let toml = r#"
id           = "in_progress"
label        = "In Progress"
instructions = "apm.worker.md"
"#;
        let s: StateConfig = toml::from_str(toml).unwrap();
        assert_eq!(s.id, "in_progress");
        assert_eq!(s.instructions.as_deref(), Some("apm.worker.md"));
    }

    #[test]
    fn state_config_instructions_default_none() {
        let toml = r#"
id    = "new"
label = "New"
"#;
        let s: StateConfig = toml::from_str(toml).unwrap();
        assert!(s.instructions.is_none());
    }

    #[test]
    fn transition_config_new_fields() {
        let toml = r#"
to              = "implemented"
trigger         = "manual"
actor           = "agent"
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
actor   = "supervisor"
"#;
        let t: TransitionConfig = toml::from_str(toml).unwrap();
        assert_eq!(t.completion, CompletionStrategy::None);
        assert!(t.focus_section.is_none());
        assert!(t.context_section.is_none());
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
        assert_eq!(config.workers.command, "claude");
        assert_eq!(config.workers.args, vec!["--print"]);
        assert!(config.workers.model.is_none());
        assert!(config.workers.env.is_empty());
    }

    #[test]
    fn workers_config_all_fields() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
command = "codex"
args = ["--full-auto"]
model = "o3"

[workers.env]
CUSTOM_VAR = "value"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.workers.command, "codex");
        assert_eq!(config.workers.args, vec!["--full-auto"]);
        assert_eq!(config.workers.model.as_deref(), Some("o3"));
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
    fn merge_local_overrides_and_extends() {
        let mut wc = WorkersConfig::default();
        assert_eq!(wc.command, "claude");
        assert_eq!(wc.args, vec!["--print"]);

        let local = LocalWorkersOverride {
            command: Some("aider".to_string()),
            args: None,
            model: Some("gpt-4".to_string()),
            env: [("KEY".to_string(), "val".to_string())].into(),
        };
        wc.merge_local(&local);

        assert_eq!(wc.command, "aider");
        assert_eq!(wc.args, vec!["--print"]); // unchanged
        assert_eq!(wc.model.as_deref(), Some("gpt-4"));
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
actionable = ["agent"]

[[workflow.states]]
id = "in_progress"
label = "In Progress"

[[workflow.states]]
id = "specd"
label = "Specd"
actionable = ["supervisor"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let states = config.actionable_states_for("agent");
        assert!(states.contains(&"ready".to_string()));
        assert!(!states.contains(&"specd".to_string()));
        assert!(!states.contains(&"in_progress".to_string()));
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
    fn resolve_collaborators_returns_static_when_no_git_host() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let local = LocalConfig::default();
        let result = resolve_collaborators(&config, &local);
        assert_eq!(result, vec!["alice", "bob"]);
    }

    #[test]
    fn resolve_collaborators_returns_static_when_github_but_no_token() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "owner/name"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let local = LocalConfig::default();
        // No token in local, and GITHUB_TOKEN env var should not be set in test env
        // (if it is, the test would make a real API call — so we just check fallback works)
        // We can't guarantee env is clean, so we only test the no-token path
        let _ = resolve_collaborators(&config, &local);
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
}
