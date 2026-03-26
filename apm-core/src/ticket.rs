use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub id: u32,
    pub title: String,
    pub state: String,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub effort: u8,
    #[serde(default)]
    pub risk: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub frontmatter: Frontmatter,
    pub body: String,
    pub path: PathBuf,
}

impl Ticket {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        Self::parse(path, &raw)
    }

    pub fn parse(path: &Path, raw: &str) -> Result<Self> {
        let Some(rest) = raw.strip_prefix("+++\n") else {
            bail!("missing frontmatter in {}", path.display());
        };
        let Some(end) = rest.find("\n+++") else {
            bail!("unclosed frontmatter in {}", path.display());
        };
        let toml_src = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let frontmatter: Frontmatter = toml::from_str(toml_src)
            .with_context(|| format!("cannot parse frontmatter in {}", path.display()))?;
        Ok(Self { frontmatter, body, path: path.to_owned() })
    }

    pub fn serialize(&self) -> Result<String> {
        let fm = toml::to_string(&self.frontmatter)
            .context("cannot serialize frontmatter")?;
        Ok(format!("+++\n{}+++\n\n{}", fm, self.body))
    }

    pub fn save(&self) -> Result<()> {
        let content = self.serialize()?;
        std::fs::write(&self.path, content)
            .with_context(|| format!("cannot write {}", self.path.display()))
    }

    pub fn score(&self, priority_weight: f64, effort_weight: f64, risk_weight: f64) -> f64 {
        let fm = &self.frontmatter;
        fm.priority as f64 * priority_weight
            + fm.effort as f64 * effort_weight
            + fm.risk as f64 * risk_weight
    }
}

pub fn load_all(tickets_dir: &Path) -> Result<Vec<Ticket>> {
    let mut tickets = Vec::new();
    let entries = std::fs::read_dir(tickets_dir)
        .with_context(|| format!("cannot read tickets dir {}", tickets_dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            match Ticket::load(&path) {
                Ok(t) => tickets.push(t),
                Err(e) => eprintln!("warning: {e:#}"),
            }
        }
    }
    tickets.sort_by_key(|t| t.frontmatter.id);
    Ok(tickets)
}

pub fn next_id(tickets_dir: &Path) -> Result<u32> {
    let path = tickets_dir.join("NEXT_ID");
    let id: u32 = if path.exists() {
        std::fs::read_to_string(&path)?.trim().parse().context("invalid NEXT_ID")?
    } else {
        1
    };
    std::fs::write(&path, format!("{}\n", id + 1))?;
    Ok(id)
}
