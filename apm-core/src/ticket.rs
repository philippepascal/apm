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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_section: Option<String>,
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

// ── TicketDocument ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChecklistItem {
    pub checked: bool,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptySection(&'static str),
    NoAcceptanceCriteria,
    UncheckedCriterion(usize),
    UncheckedAmendment(usize),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySection(s) => write!(f, "### {s} section is empty"),
            Self::NoAcceptanceCriteria => write!(f, "### Acceptance criteria has no checklist items"),
            Self::UncheckedCriterion(i) => write!(f, "acceptance criterion #{} is not checked", i + 1),
            Self::UncheckedAmendment(i) => write!(f, "amendment request #{} is not checked", i + 1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TicketDocument {
    pub problem: String,
    pub acceptance_criteria: Vec<ChecklistItem>,
    pub out_of_scope: String,
    pub approach: String,
    pub open_questions: Option<String>,
    pub amendment_requests: Option<Vec<ChecklistItem>>,
    raw_history: String,
}

fn parse_checklist(text: &str) -> Vec<ChecklistItem> {
    text.lines()
        .filter_map(|line| {
            let l = line.trim();
            if l.starts_with("- [ ] ") {
                Some(ChecklistItem { checked: false, text: l[6..].to_string() })
            } else if l.starts_with("- [x] ") || l.starts_with("- [X] ") {
                Some(ChecklistItem { checked: true, text: l[6..].to_string() })
            } else {
                None
            }
        })
        .collect()
}

fn serialize_checklist(items: &[ChecklistItem]) -> String {
    items.iter()
        .map(|i| format!("- [{}] {}", if i.checked { "x" } else { " " }, i.text))
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_sections(text: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut current: Option<String> = None;
    let mut lines: Vec<&str> = Vec::new();
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("### ") {
            if let Some(prev) = current.take() {
                map.insert(prev, lines.join("\n").trim().to_string());
            }
            current = Some(name.trim().to_string());
            lines.clear();
        } else if line.starts_with("## ") {
            if let Some(prev) = current.take() {
                map.insert(prev, lines.join("\n").trim().to_string());
            }
            lines.clear();
        } else if current.is_some() {
            lines.push(line);
        }
    }
    if let Some(name) = current {
        map.insert(name, lines.join("\n").trim().to_string());
    }
    map
}

impl TicketDocument {
    pub fn parse(body: &str) -> Result<Self> {
        let (spec_part, raw_history) = if let Some(pos) = body.find("\n## History") {
            (&body[..pos], body[pos + 1..].to_string())
        } else {
            (body, String::new())
        };

        let sections = extract_sections(spec_part);

        for name in ["Problem", "Acceptance criteria", "Out of scope", "Approach"] {
            if !sections.contains_key(name) {
                anyhow::bail!("missing required section: ### {name}");
            }
        }

        Ok(Self {
            problem: sections["Problem"].clone(),
            acceptance_criteria: parse_checklist(&sections["Acceptance criteria"]),
            out_of_scope: sections["Out of scope"].clone(),
            approach: sections["Approach"].clone(),
            open_questions: sections.get("Open questions").cloned(),
            amendment_requests: sections.get("Amendment requests").map(|s| parse_checklist(s)),
            raw_history,
        })
    }

    pub fn serialize(&self) -> String {
        let mut out = String::from("## Spec\n");

        out.push_str("\n### Problem\n\n");
        out.push_str(&self.problem);
        out.push('\n');

        out.push_str("\n### Acceptance criteria\n\n");
        if !self.acceptance_criteria.is_empty() {
            out.push_str(&serialize_checklist(&self.acceptance_criteria));
            out.push('\n');
        }

        out.push_str("\n### Out of scope\n\n");
        out.push_str(&self.out_of_scope);
        out.push('\n');

        out.push_str("\n### Approach\n\n");
        out.push_str(&self.approach);
        out.push('\n');

        if let Some(oq) = &self.open_questions {
            out.push_str("\n### Open questions\n\n");
            out.push_str(oq);
            out.push('\n');
        }

        if let Some(ar) = &self.amendment_requests {
            out.push_str("\n### Amendment requests\n\n");
            out.push_str(&serialize_checklist(ar));
            out.push('\n');
        }

        if !self.raw_history.is_empty() {
            out.push('\n');
            out.push_str(&self.raw_history);
        }

        out
    }

    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        if self.problem.is_empty() {
            errors.push(ValidationError::EmptySection("Problem"));
        }
        if self.acceptance_criteria.is_empty() {
            errors.push(ValidationError::NoAcceptanceCriteria);
        }
        if self.out_of_scope.is_empty() {
            errors.push(ValidationError::EmptySection("Out of scope"));
        }
        if self.approach.is_empty() {
            errors.push(ValidationError::EmptySection("Approach"));
        }
        errors
    }

    pub fn unchecked_criteria(&self) -> Vec<usize> {
        self.acceptance_criteria.iter().enumerate()
            .filter(|(_, c)| !c.checked)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn unchecked_amendments(&self) -> Vec<usize> {
        self.amendment_requests.as_deref().unwrap_or(&[]).iter().enumerate()
            .filter(|(_, c)| !c.checked)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn toggle_criterion(&mut self, index: usize, checked: bool) -> Result<()> {
        if index >= self.acceptance_criteria.len() {
            anyhow::bail!("criterion index {index} out of range (have {})", self.acceptance_criteria.len());
        }
        self.acceptance_criteria[index].checked = checked;
        Ok(())
    }
}

impl Ticket {
    pub fn document(&self) -> Result<TicketDocument> {
        TicketDocument::parse(&self.body)
    }
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

    // ── TicketDocument ────────────────────────────────────────────────────

    fn full_body(ac: &str) -> String {
        format!(
            "## Spec\n\n### Problem\n\nSome problem.\n\n### Acceptance criteria\n\n{ac}\n\n### Out of scope\n\nNothing.\n\n### Approach\n\nDo it.\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|"
        )
    }

    #[test]
    fn document_parse_required_sections() {
        let body = full_body("- [ ] item one\n- [x] item two");
        let doc = TicketDocument::parse(&body).unwrap();
        assert_eq!(doc.problem, "Some problem.");
        assert_eq!(doc.acceptance_criteria.len(), 2);
        assert!(!doc.acceptance_criteria[0].checked);
        assert!(doc.acceptance_criteria[1].checked);
        assert_eq!(doc.out_of_scope, "Nothing.");
        assert_eq!(doc.approach, "Do it.");
    }

    #[test]
    fn document_parse_missing_section_errors() {
        let body = "## Spec\n\n### Problem\n\nSome problem.\n\n## History\n\n";
        let err = TicketDocument::parse(body).unwrap_err();
        assert!(err.to_string().contains("missing required section"));
    }

    #[test]
    fn document_round_trip() {
        let body = full_body("- [ ] criterion A\n- [x] criterion B");
        let doc = TicketDocument::parse(&body).unwrap();
        let serialized = doc.serialize();
        let doc2 = TicketDocument::parse(&serialized).unwrap();
        assert_eq!(doc2.problem, doc.problem);
        assert_eq!(doc2.acceptance_criteria.len(), doc.acceptance_criteria.len());
        assert_eq!(doc2.acceptance_criteria[0].checked, false);
        assert_eq!(doc2.acceptance_criteria[1].checked, true);
        assert_eq!(doc2.out_of_scope, doc.out_of_scope);
        assert_eq!(doc2.approach, doc.approach);
    }

    #[test]
    fn document_validate_empty_sections() {
        let body = "## Spec\n\n### Problem\n\n\n### Acceptance criteria\n\n- [ ] x\n\n### Out of scope\n\n\n### Approach\n\ncontent\n";
        let doc = TicketDocument::parse(body).unwrap();
        let errs = doc.validate();
        let msgs: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        assert!(msgs.iter().any(|m| m.contains("Problem")));
        assert!(msgs.iter().any(|m| m.contains("Out of scope")));
        assert!(!msgs.iter().any(|m| m.contains("Approach")));
    }

    #[test]
    fn document_validate_no_criteria() {
        let body = "## Spec\n\n### Problem\n\nfoo\n\n### Acceptance criteria\n\n\n### Out of scope\n\nbar\n\n### Approach\n\nbaz\n";
        let doc = TicketDocument::parse(body).unwrap();
        let errs = doc.validate();
        assert!(errs.iter().any(|e| matches!(e, ValidationError::NoAcceptanceCriteria)));
    }

    #[test]
    fn document_toggle_criterion() {
        let body = full_body("- [ ] item one\n- [ ] item two");
        let mut doc = TicketDocument::parse(&body).unwrap();
        assert!(!doc.acceptance_criteria[0].checked);
        doc.toggle_criterion(0, true).unwrap();
        assert!(doc.acceptance_criteria[0].checked);
    }

    #[test]
    fn document_unchecked_criteria() {
        let body = full_body("- [ ] one\n- [x] two\n- [ ] three");
        let doc = TicketDocument::parse(&body).unwrap();
        assert_eq!(doc.unchecked_criteria(), vec![0, 2]);
    }

    #[test]
    fn document_history_preserved() {
        let body = full_body("- [x] done");
        let doc = TicketDocument::parse(&body).unwrap();
        let s = doc.serialize();
        assert!(s.contains("## History"));
        assert!(s.contains("| When |"));
    }
}
