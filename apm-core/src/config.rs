use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

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
    pub tickets: TicketsConfig,
    #[serde(default)]
    pub workflow: WorkflowConfig,
    #[serde(default)]
    pub agents: AgentsConfig,
    #[serde(default)]
    pub worktrees: WorktreesConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
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
