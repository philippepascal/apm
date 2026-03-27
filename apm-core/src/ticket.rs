use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
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
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supervisor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
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

/// Load all tickets by reading directly from their git branches.
/// No filesystem cache is involved.
pub fn load_all_from_git(root: &Path, tickets_dir_rel: &std::path::Path) -> Result<Vec<Ticket>> {
    let branches = crate::git::ticket_branches(root)?;
    let mut tickets = Vec::new();
    for branch in &branches {
        let suffix = branch.trim_start_matches("ticket/");
        let filename = format!("{suffix}.md");
        let rel_path = format!("{}/{}", tickets_dir_rel.to_string_lossy(), filename);
        let dummy_path = root.join(&rel_path);
        match crate::git::read_from_branch(root, branch, &rel_path) {
            Ok(content) => match Ticket::parse(&dummy_path, &content) {
                Ok(t) => tickets.push(t),
                Err(e) => eprintln!("warning: {branch}: {e:#}"),
            },
            Err(e) => eprintln!("warning: cannot read {branch}: {e:#}"),
        }
    }
    tickets.sort_by_key(|t| t.frontmatter.id);
    Ok(tickets)
}

pub fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(40)
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn dummy_path() -> &'static Path {
        Path::new("test.md")
    }

    fn minimal_raw(extra_fm: &str, body: &str) -> String {
        format!(
            "+++\nid = 1\ntitle = \"Test\"\nstate = \"new\"\n{extra_fm}+++\n\n{body}"
        )
    }

    // --- parse ---

    #[test]
    fn parse_well_formed() {
        let raw = minimal_raw("priority = 5\n", "## Spec\n\nHello\n");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert_eq!(t.frontmatter.id, 1);
        assert_eq!(t.frontmatter.title, "Test");
        assert_eq!(t.frontmatter.state, "new");
        assert_eq!(t.frontmatter.priority, 5);
        assert_eq!(t.body, "## Spec\n\nHello\n");
    }

    #[test]
    fn parse_optional_fields_default() {
        let raw = minimal_raw("", "");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert_eq!(t.frontmatter.priority, 0);
        assert_eq!(t.frontmatter.effort, 0);
        assert_eq!(t.frontmatter.risk, 0);
        assert!(t.frontmatter.agent.is_none());
        assert!(t.frontmatter.branch.is_none());
    }

    #[test]
    fn parse_missing_opening_delimiter() {
        let raw = "id = 1\ntitle = \"Test\"\nstate = \"new\"\n+++\n\nbody\n";
        let err = Ticket::parse(dummy_path(), raw).unwrap_err();
        assert!(err.to_string().contains("missing frontmatter"));
    }

    #[test]
    fn parse_unclosed_frontmatter() {
        let raw = "+++\nid = 1\ntitle = \"Test\"\nstate = \"new\"\n\nbody\n";
        let err = Ticket::parse(dummy_path(), raw).unwrap_err();
        assert!(err.to_string().contains("unclosed frontmatter"));
    }

    #[test]
    fn parse_invalid_toml() {
        let raw = "+++\nid = not_a_number\n+++\n\nbody\n";
        let err = Ticket::parse(dummy_path(), raw).unwrap_err();
        assert!(err.to_string().contains("cannot parse frontmatter"));
    }

    // --- serialize round-trip ---

    #[test]
    fn serialize_round_trips() {
        let raw = minimal_raw("effort = 3\nrisk = 1\n", "## Spec\n\ncontent\n");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        let serialized = t.serialize().unwrap();
        let t2 = Ticket::parse(dummy_path(), &serialized).unwrap();
        assert_eq!(t2.frontmatter.id, t.frontmatter.id);
        assert_eq!(t2.frontmatter.title, t.frontmatter.title);
        assert_eq!(t2.frontmatter.state, t.frontmatter.state);
        assert_eq!(t2.frontmatter.effort, t.frontmatter.effort);
        assert_eq!(t2.frontmatter.risk, t.frontmatter.risk);
        assert_eq!(t2.body, t.body);
    }

    // --- slugify ---

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn slugify_special_chars() {
        assert_eq!(slugify("Add apm init --hooks (install git hooks)"), "add-apm-init-hooks-install-git-hooks");
    }

    #[test]
    fn slugify_truncates_at_40() {
        let long = "a".repeat(50);
        assert_eq!(slugify(&long).len(), 40);
    }

    #[test]
    fn slugify_collapses_separators() {
        assert_eq!(slugify("foo  --  bar"), "foo-bar");
    }

    // --- next_id ---

    #[test]
    fn next_id_creates_and_increments() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        assert_eq!(next_id(p).unwrap(), 1);
        assert_eq!(next_id(p).unwrap(), 2);
        assert_eq!(next_id(p).unwrap(), 3);
    }
}
