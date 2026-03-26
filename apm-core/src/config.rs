use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    #[serde(default)]
    pub tickets: TicketsConfig,
    #[serde(default)]
    pub workflow: WorkflowConfig,
    #[serde(default)]
    pub agents: AgentsConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
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
}

#[derive(Debug, Deserialize)]
pub struct TransitionConfig {
    pub to: String,
    pub trigger: String,
    #[serde(default)]
    pub actor: String,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub side_effects: Vec<String>,
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
    #[serde(default = "default_actionable_states")]
    pub actionable_states: Vec<String>,
    #[serde(default)]
    pub instructions: Option<PathBuf>,
}

fn default_max_concurrent() -> usize { 3 }
fn default_actionable_states() -> Vec<String> {
    vec!["new".into(), "ammend".into(), "ready".into()]
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            max_concurrent: default_max_concurrent(),
            actionable_states: default_actionable_states(),
            instructions: None,
        }
    }
}

impl Config {
    pub fn load(repo_root: &Path) -> Result<Self> {
        let path = repo_root.join("apm.toml");
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        toml::from_str(&contents)
            .with_context(|| format!("cannot parse {}", path.display()))
    }
}
