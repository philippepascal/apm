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
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SyncConfig {
    #[serde(default)]
    pub aggressive: bool,
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
}

impl Default for TicketsConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("tickets"),
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
        let path = repo_root.join("apm.toml");
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
}
