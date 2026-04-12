use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn deserialize_id<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    use serde::de::{self, Visitor};
    struct IdVisitor;
    impl<'de> Visitor<'de> for IdVisitor {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an integer or hex string")
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
            Ok(format!("{v:04}"))
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<String, E> {
            Ok(format!("{v:04}"))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_string<E: de::Error>(self, v: String) -> Result<String, E> {
            Ok(v)
        }
    }
    d.deserialize_any(IdVisitor)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
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
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
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

    pub fn document(&self) -> Result<TicketDocument> {
        TicketDocument::parse(&self.body)
    }
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

// ── TicketDocument ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChecklistItem {
    pub checked: bool,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptySection(String),
    NoAcceptanceCriteria,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySection(s) => write!(f, "### {s} section is empty"),
            Self::NoAcceptanceCriteria => write!(f, "### Acceptance criteria has no checklist items"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TicketDocument {
    pub sections: IndexMap<String, String>,
    pub(crate) raw_history: String,
}

pub(crate) fn parse_checklist(text: &str) -> Vec<ChecklistItem> {
    text.lines()
        .filter_map(|line| {
            let l = line.trim();
            if let Some(s) = l.strip_prefix("- [ ] ") {
                Some(ChecklistItem { checked: false, text: s.to_string() })
            } else if let Some(s) = l.strip_prefix("- [x] ") {
                Some(ChecklistItem { checked: true, text: s.to_string() })
            } else {
                l.strip_prefix("- [X] ").map(|s| ChecklistItem { checked: true, text: s.to_string() })
            }
        })
        .collect()
}

pub(crate) fn serialize_checklist(items: &[ChecklistItem]) -> String {
    items.iter()
        .map(|i| format!("- [{}] {}", if i.checked { "x" } else { " " }, i.text))
        .collect::<Vec<_>>()
        .join("\n")
}

impl TicketDocument {
    pub fn parse(body: &str) -> Result<Self> {
        let (spec_part, raw_history) = if let Some(pos) = body.find("\n## History") {
            (&body[..pos], body[pos + 1..].to_string())
        } else {
            (body, String::new())
        };

        let mut sections = IndexMap::new();
        let mut current_name: Option<String> = None;
        let mut current_lines: Vec<&str> = Vec::new();

        for line in spec_part.lines() {
            if let Some(name) = line.strip_prefix("### ") {
                if let Some(prev) = current_name.take() {
                    sections.insert(prev, current_lines.join("\n").trim().to_string());
                }
                current_name = Some(name.trim().to_string());
                current_lines.clear();
            } else if line.starts_with("## ") {
                if let Some(prev) = current_name.take() {
                    sections.insert(prev, current_lines.join("\n").trim().to_string());
                }
                current_lines.clear();
            } else if current_name.is_some() {
                current_lines.push(line);
            }
        }
        if let Some(name) = current_name {
            sections.insert(name, current_lines.join("\n").trim().to_string());
        }

        Ok(Self { sections, raw_history })
    }

    pub fn serialize(&self) -> String {
        let mut out = String::from("## Spec\n");

        for (name, value) in &self.sections {
            out.push_str(&format!("\n### {}\n\n", name));
            if !value.is_empty() {
                out.push_str(value);
                out.push('\n');
            }
        }

        if !self.raw_history.is_empty() {
            out.push('\n');
            out.push_str(&self.raw_history);
        }

        out
    }

    pub fn validate(&self, config_sections: &[crate::config::TicketSection]) -> Vec<ValidationError> {
        use crate::config::SectionType;
        let mut errors = Vec::new();
        for sec in config_sections {
            if !sec.required {
                continue;
            }
            let val = self.sections.get(&sec.name).map(|s| s.as_str()).unwrap_or("");
            if val.is_empty() {
                if sec.type_ == SectionType::Tasks {
                    errors.push(ValidationError::NoAcceptanceCriteria);
                } else {
                    errors.push(ValidationError::EmptySection(sec.name.clone()));
                }
                continue;
            }
            if sec.type_ == SectionType::Tasks && parse_checklist(val).is_empty() {
                errors.push(ValidationError::NoAcceptanceCriteria);
            }
        }
        errors
    }
}

/// Normalize a user-supplied ID argument to a canonical prefix string.
/// Accepts: plain integer (zero-padded to 4 chars), or 4–8 hex char string.
pub fn normalize_id_arg(arg: &str) -> Result<String> {
    if !arg.is_empty() && arg.chars().all(|c| c.is_ascii_digit()) {
        let n: u64 = arg.parse().context("invalid integer ID")?;
        return Ok(format!("{n:04}"));
    }
    if arg.len() < 4 || arg.len() > 8 {
        bail!("invalid ticket ID {:?}: use 4–8 hex chars or a plain integer", arg);
    }
    if !arg.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid ticket ID {:?}: not a hex string", arg);
    }
    Ok(arg.to_lowercase())
}

/// Return all candidate prefix strings for a user-supplied ID argument.
///
/// For all-digit inputs shorter than 4 chars, both the zero-padded form and
/// the raw digit string are returned (the raw string is the correct hex prefix).
/// For all other inputs a single-element vec is returned.
pub fn id_arg_prefixes(arg: &str) -> Result<Vec<String>> {
    let canonical = normalize_id_arg(arg)?;
    if arg.chars().all(|c| c.is_ascii_digit()) && arg.len() < 4 {
        Ok(vec![canonical, arg.to_string()])
    } else {
        Ok(vec![canonical])
    }
}

/// Resolve a user-supplied ID argument to a unique ticket ID from a loaded list.
pub fn resolve_id_in_slice(tickets: &[Ticket], arg: &str) -> Result<String> {
    let prefixes = id_arg_prefixes(arg)?;
    let mut seen = std::collections::HashSet::new();
    let matches: Vec<&Ticket> = tickets.iter()
        .filter(|t| {
            let id = &t.frontmatter.id;
            prefixes.iter().any(|p| id.starts_with(p.as_str())) && seen.insert(id.clone())
        })
        .collect();
    match matches.len() {
        0 => bail!("no ticket matches '{arg}'"),
        1 => Ok(matches[0].frontmatter.id.clone()),
        _ => {
            let mut msg = format!("error: prefix '{arg}' is ambiguous");
            for t in &matches {
                msg.push_str(&format!("\n  {}  {}", t.frontmatter.id, t.frontmatter.title));
            }
            bail!("{msg}")
        }
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
            "+++\nid = \"0001\"\ntitle = \"Test\"\nstate = \"new\"\n{extra_fm}+++\n\n{body}"
        )
    }

    fn minimal_raw_int(extra_fm: &str, body: &str) -> String {
        format!(
            "+++\nid = 1\ntitle = \"Test\"\nstate = \"new\"\n{extra_fm}+++\n\n{body}"
        )
    }

    // --- parse ---

    #[test]
    fn parse_well_formed() {
        let raw = minimal_raw("priority = 5\n", "## Spec\n\nHello\n");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert_eq!(t.frontmatter.id, "0001");
        assert_eq!(t.frontmatter.title, "Test");
        assert_eq!(t.frontmatter.state, "new");
        assert_eq!(t.frontmatter.priority, 5);
        assert_eq!(t.body, "## Spec\n\nHello\n");
    }

    #[test]
    fn parse_integer_id_is_zero_padded() {
        let raw = minimal_raw_int("", "");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert_eq!(t.frontmatter.id, "0001");
    }

    #[test]
    fn parse_optional_fields_default() {
        let raw = minimal_raw("", "");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert_eq!(t.frontmatter.priority, 0);
        assert_eq!(t.frontmatter.effort, 0);
        assert_eq!(t.frontmatter.risk, 0);
        assert!(t.frontmatter.branch.is_none());
    }

    #[test]
    fn parse_epic_field() {
        let raw = minimal_raw("epic = \"ab12cd34\"\n", "");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert_eq!(t.frontmatter.epic, Some("ab12cd34".to_string()));
    }

    #[test]
    fn parse_target_branch_field() {
        let raw = minimal_raw("target_branch = \"epic/ab12cd34-user-auth\"\n", "");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert_eq!(t.frontmatter.target_branch, Some("epic/ab12cd34-user-auth".to_string()));
    }

    #[test]
    fn parse_depends_on_field() {
        let raw = minimal_raw("depends_on = [\"cd56ef78\", \"12ab34cd\"]\n", "");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert_eq!(t.frontmatter.depends_on, Some(vec!["cd56ef78".to_string(), "12ab34cd".to_string()]));
    }

    #[test]
    fn parse_omits_new_fields() {
        let raw = minimal_raw("", "");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert!(t.frontmatter.epic.is_none());
        assert!(t.frontmatter.target_branch.is_none());
        assert!(t.frontmatter.depends_on.is_none());
    }

    #[test]
    fn serialize_omits_absent_fields() {
        let raw = minimal_raw("", "## Spec\n\ncontent\n");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        let serialized = t.serialize().unwrap();
        assert!(!serialized.contains("epic"));
        assert!(!serialized.contains("target_branch"));
        assert!(!serialized.contains("depends_on"));
    }

    #[test]
    fn parse_missing_opening_delimiter() {
        let raw = "id = \"0001\"\ntitle = \"Test\"\nstate = \"new\"\n+++\n\nbody\n";
        let err = Ticket::parse(dummy_path(), raw).unwrap_err();
        assert!(err.to_string().contains("missing frontmatter"));
    }

    #[test]
    fn parse_unclosed_frontmatter() {
        let raw = "+++\nid = \"0001\"\ntitle = \"Test\"\nstate = \"new\"\n\nbody\n";
        let err = Ticket::parse(dummy_path(), raw).unwrap_err();
        assert!(err.to_string().contains("unclosed frontmatter"));
    }

    #[test]
    fn parse_invalid_toml() {
        let raw = "+++\nid = not_a_number\n+++\n\nbody\n";
        let err = Ticket::parse(dummy_path(), raw).unwrap_err();
        assert!(err.to_string().contains("cannot parse frontmatter"));
    }

    #[test]
    fn epic_and_depends_on_round_trip() {
        let raw = minimal_raw(
            "epic = \"ab12cd34\"\ndepends_on = [\"cd56ef78\", \"12ab34cd\"]\n",
            "## Spec\n\ncontent\n",
        );
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        assert_eq!(t.frontmatter.epic, Some("ab12cd34".to_string()));
        assert_eq!(
            t.frontmatter.depends_on,
            Some(vec!["cd56ef78".to_string(), "12ab34cd".to_string()])
        );
        let serialized = t.serialize().unwrap();
        assert!(serialized.contains("epic = \"ab12cd34\""));
        assert!(serialized.contains("depends_on = [\"cd56ef78\", \"12ab34cd\"]"));
        let t2 = Ticket::parse(dummy_path(), &serialized).unwrap();
        assert_eq!(t2.frontmatter.epic, Some("ab12cd34".to_string()));
        assert_eq!(
            t2.frontmatter.depends_on,
            Some(vec!["cd56ef78".to_string(), "12ab34cd".to_string()])
        );
    }

    #[test]
    fn target_branch_round_trips() {
        let raw = minimal_raw("target_branch = \"epic/abc\"\n", "## Spec\n\ncontent\n");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        let serialized = t.serialize().unwrap();
        assert!(serialized.contains("target_branch = \"epic/abc\""));
        let t2 = Ticket::parse(dummy_path(), &serialized).unwrap();
        assert_eq!(t2.frontmatter.target_branch, Some("epic/abc".to_string()));
    }

    #[test]
    fn target_branch_absent_not_added_on_round_trip() {
        let raw = minimal_raw("", "## Spec\n\ncontent\n");
        let t = Ticket::parse(dummy_path(), &raw).unwrap();
        let serialized = t.serialize().unwrap();
        assert!(!serialized.contains("target_branch"));
        let t2 = Ticket::parse(dummy_path(), &serialized).unwrap();
        assert!(t2.frontmatter.target_branch.is_none());
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

    // --- normalize_id_arg ---

    #[test]
    fn normalize_integer_pads_to_four() {
        assert_eq!(normalize_id_arg("35").unwrap(), "0035");
        assert_eq!(normalize_id_arg("1").unwrap(), "0001");
        assert_eq!(normalize_id_arg("9999").unwrap(), "9999");
    }

    #[test]
    fn normalize_hex_passthrough() {
        assert_eq!(normalize_id_arg("a3f9b2c1").unwrap(), "a3f9b2c1");
        assert_eq!(normalize_id_arg("a3f9").unwrap(), "a3f9");
    }

    #[test]
    fn normalize_too_short_errors() {
        assert!(normalize_id_arg("abc").is_err());
    }

    #[test]
    fn normalize_non_hex_errors() {
        assert!(normalize_id_arg("gggg").is_err());
    }

    // --- id_arg_prefixes ---

    #[test]
    fn prefixes_short_digit_returns_two() {
        let p = id_arg_prefixes("314").unwrap();
        assert_eq!(p, vec!["0314", "314"]);
    }

    #[test]
    fn prefixes_four_digit_returns_one() {
        let p = id_arg_prefixes("3142").unwrap();
        assert_eq!(p, vec!["3142"]);
    }

    #[test]
    fn prefixes_hex_returns_one() {
        let p = id_arg_prefixes("a3f9").unwrap();
        assert_eq!(p, vec!["a3f9"]);
    }

    // --- resolve_id_in_slice ---

    fn make_ticket_with_title(id: &str, title: &str) -> Ticket {
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"{title}\"\nstate = \"new\"\n+++\n\nbody\n"
        );
        let path = std::path::PathBuf::from(format!("tickets/{id}.md"));
        Ticket::parse(&path, &raw).unwrap()
    }

    #[test]
    fn resolve_short_digit_prefix_unique() {
        let tickets = vec![make_ticket_with_title("314abcde", "Alpha")];
        assert_eq!(resolve_id_in_slice(&tickets, "314").unwrap(), "314abcde");
    }

    #[test]
    fn resolve_integer_one_matches_0001() {
        let tickets = vec![make_ticket_with_title("0001", "One")];
        assert_eq!(resolve_id_in_slice(&tickets, "1").unwrap(), "0001");
    }

    #[test]
    fn resolve_four_digit_prefix() {
        let tickets = vec![make_ticket_with_title("3142abcd", "Beta")];
        assert_eq!(resolve_id_in_slice(&tickets, "3142").unwrap(), "3142abcd");
    }

    #[test]
    fn resolve_ambiguous_prefix_lists_candidates() {
        let tickets = vec![
            make_ticket_with_title("314abcde", "Alpha"),
            make_ticket_with_title("3142xxxx", "Beta"),
        ];
        let err = resolve_id_in_slice(&tickets, "314").unwrap_err().to_string();
        assert!(err.contains("ambiguous"), "expected 'ambiguous' in: {err}");
        assert!(err.contains("314abcde"), "expected first id in: {err}");
        assert!(err.contains("3142xxxx"), "expected second id in: {err}");
    }

    // ── TicketDocument ────────────────────────────────────────────────────

    fn full_body(ac: &str) -> String {
        format!(
            "## Spec\n\n### Problem\n\nSome problem.\n\n### Acceptance criteria\n\n{ac}\n\n### Out of scope\n\nNothing.\n\n### Approach\n\nDo it.\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|"
        )
    }

    fn minimal_ticket_sections() -> Vec<crate::config::TicketSection> {
        use crate::config::{SectionType, TicketSection};
        vec![
            TicketSection { name: "Problem".into(), type_: SectionType::Free, required: true, placeholder: None },
            TicketSection { name: "Acceptance criteria".into(), type_: SectionType::Tasks, required: true, placeholder: None },
            TicketSection { name: "Out of scope".into(), type_: SectionType::Free, required: true, placeholder: None },
            TicketSection { name: "Approach".into(), type_: SectionType::Free, required: true, placeholder: None },
        ]
    }

    #[test]
    fn document_parse_required_sections() {
        let body = full_body("- [ ] item one\n- [x] item two");
        let doc = TicketDocument::parse(&body).unwrap();
        assert_eq!(doc.sections.get("Problem").map(|s| s.as_str()), Some("Some problem."));
        let ac = doc.sections.get("Acceptance criteria").unwrap();
        assert!(ac.contains("- [ ] item one"));
        assert!(ac.contains("- [x] item two"));
        assert_eq!(doc.sections.get("Out of scope").map(|s| s.as_str()), Some("Nothing."));
        assert_eq!(doc.sections.get("Approach").map(|s| s.as_str()), Some("Do it."));
    }

    #[test]
    fn document_parse_missing_section_fails_validate() {
        let body = "## Spec\n\n### Problem\n\nSome problem.\n\n## History\n\n";
        let doc = TicketDocument::parse(body).unwrap();
        let errs = doc.validate(&minimal_ticket_sections());
        assert!(!errs.is_empty(), "expected validation errors for missing required sections");
    }

    #[test]
    fn document_parse_unknown_section_preserved() {
        let body = "## Spec\n\n### Problem\n\nfoo\n\n### Acceptance criteria\n\n- [x] done\n\n### Out of scope\n\nbar\n\n### Approach\n\nbaz\n\n### Foo\n\nsome custom content\n\n## History\n\n";
        let doc = TicketDocument::parse(body).unwrap();
        assert_eq!(doc.sections.get("Foo").map(|s| s.as_str()), Some("some custom content"));
        let s = doc.serialize();
        assert!(s.contains("### Foo"), "unknown section should be preserved in serialization");
        assert!(s.contains("some custom content"));
    }

    #[test]
    fn document_parse_code_review_preserved() {
        let body = "## Spec\n\n### Problem\n\nfoo\n\n### Acceptance criteria\n\n- [x] done\n\n### Out of scope\n\nbar\n\n### Approach\n\nbaz\n\n### Code review\n\n- [ ] Check tests\n\n## History\n\n";
        let doc = TicketDocument::parse(body).unwrap();
        let s = doc.serialize();
        assert!(s.contains("### Code review"), "Code review section should survive round-trip");
        assert!(s.contains("- [ ] Check tests"));
    }

    #[test]
    fn document_round_trip() {
        let body = full_body("- [ ] criterion A\n- [x] criterion B");
        let doc = TicketDocument::parse(&body).unwrap();
        let serialized = doc.serialize();
        let doc2 = TicketDocument::parse(&serialized).unwrap();
        assert_eq!(doc2.sections.get("Problem"), doc.sections.get("Problem"));
        assert_eq!(doc2.sections.get("Acceptance criteria"), doc.sections.get("Acceptance criteria"));
        assert_eq!(doc2.sections.get("Out of scope"), doc.sections.get("Out of scope"));
        assert_eq!(doc2.sections.get("Approach"), doc.sections.get("Approach"));
    }

    #[test]
    fn document_validate_empty_sections() {
        let body = "## Spec\n\n### Problem\n\n\n### Acceptance criteria\n\n- [ ] x\n\n### Out of scope\n\n\n### Approach\n\ncontent\n";
        let doc = TicketDocument::parse(body).unwrap();
        let errs = doc.validate(&minimal_ticket_sections());
        let msgs: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        assert!(msgs.iter().any(|m| m.contains("Problem")));
        assert!(msgs.iter().any(|m| m.contains("Out of scope")));
        assert!(!msgs.iter().any(|m| m.contains("Approach")));
    }

    #[test]
    fn document_validate_no_criteria() {
        let body = "## Spec\n\n### Problem\n\nfoo\n\n### Acceptance criteria\n\n\n### Out of scope\n\nbar\n\n### Approach\n\nbaz\n";
        let doc = TicketDocument::parse(body).unwrap();
        let errs = doc.validate(&minimal_ticket_sections());
        assert!(errs.iter().any(|e| matches!(e, ValidationError::NoAcceptanceCriteria)));
    }

    #[test]
    fn document_validate_required_from_config() {
        use crate::config::{SectionType, TicketSection};
        let body = "## Spec\n\n### Problem\n\nfoo\n\n";
        let doc = TicketDocument::parse(body).unwrap();
        let sections = vec![
            TicketSection { name: "Problem".into(), type_: SectionType::Free, required: true, placeholder: None },
            TicketSection { name: "Context".into(), type_: SectionType::Free, required: true, placeholder: None },
        ];
        let errs = doc.validate(&sections);
        let msgs: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        assert!(msgs.iter().any(|m| m.contains("Context")), "required config section should be validated");
        assert!(!msgs.iter().any(|m| m.contains("Problem")), "present section should not error");
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
