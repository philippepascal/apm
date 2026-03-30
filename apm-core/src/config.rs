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
    #[default]
    None,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LoggingConfig {
    #[serde(default)]
    pub enabled: bool,
    pub file: Option<std::path::PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProviderConfig {
    #[serde(rename = "type", default)]
    pub type_: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WorkersConfig {
    pub container: Option<String>,
    #[serde(default)]
    pub keychain: std::collections::HashMap<String, String>,
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
    pub provider: Option<ProviderConfig>,
    #[serde(default)]
    pub workers: WorkersConfig,
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
}

fn default_branch_main() -> String {
    "main".to_string()
}

#[derive(Debug, Deserialize)]
pub struct TicketsConfig {
    pub dir: PathBuf,
    #[serde(default)]
    pub sections: Vec<String>,
}

impl Default for TicketsConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("tickets"),
            sections: Vec::new(),
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

#[derive(Debug, Deserialize)]
pub struct StateConfig {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub terminal: bool,
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

impl Config {
    /// States where `actor` can actively pick up / act on tickets.
    /// Matches "any" as a wildcard in addition to the literal actor name.
    pub fn actionable_states_for<'a>(&'a self, actor: &str) -> Vec<&'a str> {
        self.workflow.states.iter()
            .filter(|s| s.actionable.iter().any(|a| a == actor || a == "any"))
            .map(|s| s.id.as_str())
            .collect()
    }

    pub fn load(repo_root: &Path) -> Result<Self> {
        let apm_dir_path = repo_root.join(".apm").join("config.toml");
        let path = if apm_dir_path.exists() {
            apm_dir_path
        } else {
            repo_root.join("apm.toml")
        };
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        toml::from_str(&contents)
            .with_context(|| format!("cannot parse {}", path.display()))
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
        let none: W = toml::from_str("c = \"none\"").unwrap();
        assert_eq!(none.c, CompletionStrategy::None);
    }

    #[test]
    fn completion_strategy_default() {
        assert_eq!(CompletionStrategy::default(), CompletionStrategy::None);
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
}
