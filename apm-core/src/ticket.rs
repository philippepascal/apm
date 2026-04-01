use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
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

/// Return all agent-actionable tickets sorted by descending score.
pub fn sorted_actionable<'a>(
    tickets: &'a [Ticket],
    actionable: &[&str],
    pw: f64,
    ew: f64,
    rw: f64,
) -> Vec<&'a Ticket> {
    let mut candidates: Vec<&Ticket> = tickets
        .iter()
        .filter(|t| actionable.contains(&t.frontmatter.state.as_str()))
        .collect();
    candidates.sort_by(|a, b| {
        b.score(pw, ew, rw)
            .partial_cmp(&a.score(pw, ew, rw))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates
}

/// Return the highest-scoring ticket from `tickets` whose state is in
/// `actionable` and (if `startable` is non-empty) also in `startable`.
pub fn pick_next<'a>(
    tickets: &'a [Ticket],
    actionable: &[&str],
    startable: &[&str],
    pw: f64,
    ew: f64,
    rw: f64,
) -> Option<&'a Ticket> {
    sorted_actionable(tickets, actionable, pw, ew, rw)
        .into_iter()
        .find(|t| {
            let state = t.frontmatter.state.as_str();
            startable.is_empty() || startable.contains(&state)
        })
}

/// Load all tickets by reading directly from their git branches.
/// No filesystem cache is involved.
pub fn load_all_from_git(root: &Path, tickets_dir_rel: &std::path::Path) -> Result<Vec<Ticket>> {
    let branches = crate::git::ticket_branches(root)?;
    let mut tickets = Vec::new();
    for branch in &branches {
        let suffix = branch.trim_start_matches("ticket/");
        // Skip bare short-ID refs (e.g. ticket/268f5694) created by fetch operations.
        // A real ticket branch always has a slug after the ID: ticket/<id>-<slug>.
        if suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
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
    tickets.sort_by_key(|t| t.frontmatter.created_at);
    Ok(tickets)
}

/// Read a ticket's state from a specific branch by relative path.
pub fn state_from_branch(root: &Path, branch: &str, rel_path: &str) -> Option<String> {
    let content = crate::git::read_from_branch(root, branch, rel_path).ok()?;
    let dummy = root.join(rel_path);
    Ticket::parse(&dummy, &content).ok().map(|t| t.frontmatter.state)
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

/// Close a ticket from any state.  Commits the change to the ticket branch,
/// pushes it (non-fatal if no remote), then merges into the default branch
/// so that `apm clean` can detect and remove the worktree.
pub fn close(
    root: &Path,
    config: &crate::config::Config,
    id_arg: &str,
    reason: Option<&str>,
    agent: &str,
    aggressive: bool,
) -> Result<()> {
    let mut tickets = load_all_from_git(root, &config.tickets.dir)?;
    let prefixes = id_arg_prefixes(id_arg)?;

    // Search ticket branches first, then fall back to the default branch.
    // This handles stale "implemented" tickets whose branch was deleted.
    let branch_matches: Vec<usize> = tickets.iter()
        .enumerate()
        .filter(|(_, t)| prefixes.iter().any(|p| t.frontmatter.id.starts_with(p.as_str())))
        .map(|(i, _)| i)
        .collect();
    // Deduplicate in case both prefixes matched the same ticket.
    let branch_matches: Vec<usize> = {
        let mut seen = std::collections::HashSet::new();
        branch_matches.into_iter().filter(|&i| seen.insert(tickets[i].frontmatter.id.clone())).collect()
    };

    let mut from_default: Option<Ticket> = None;
    let id: String = match branch_matches.len() {
        1 => tickets[branch_matches[0]].frontmatter.id.clone(),
        0 => {
            let default_branch = &config.project.default_branch;
            let mut found: Option<Ticket> = None;
            if let Ok(files) = crate::git::list_files_on_branch(root, default_branch, &config.tickets.dir.to_string_lossy()) {
                for rel_path in files {
                    if !rel_path.ends_with(".md") { continue; }
                    if let Ok(content) = crate::git::read_from_branch(root, default_branch, &rel_path) {
                        let dummy = root.join(&rel_path);
                        if let Ok(t) = Ticket::parse(&dummy, &content) {
                            if prefixes.iter().any(|p| t.frontmatter.id.starts_with(p.as_str())) {
                                found = Some(t);
                                break;
                            }
                        }
                    }
                }
            }
            match found {
                Some(t) => { let id = t.frontmatter.id.clone(); from_default = Some(t); id }
                None => bail!("no ticket matches '{id_arg}'"),
            }
        }
        _ => {
            let names: Vec<String> = branch_matches.iter()
                .map(|&i| tickets[i].frontmatter.id.clone())
                .collect();
            bail!("ambiguous prefix '{}', matches: {}", id_arg, names.join(", "));
        }
    };

    let ticket_pos = tickets.iter().position(|t| t.frontmatter.id == id);
    let t: &mut Ticket = match ticket_pos {
        Some(pos) => &mut tickets[pos],
        None => from_default.as_mut().ok_or_else(|| anyhow::anyhow!("ticket {id:?} not found"))?,
    };

    if t.frontmatter.state == "closed" {
        anyhow::bail!("ticket {id:?} is already closed");
    }

    let now = chrono::Utc::now();
    let prev = t.frontmatter.state.clone();
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
    let by = match reason {
        Some(r) => format!("{agent} (reason: {r})"),
        None => agent.to_string(),
    };

    t.frontmatter.state = "closed".into();
    t.frontmatter.updated_at = Some(now);

    let row = format!("| {when} | {prev} | closed | {by} |");
    if t.body.contains("## History") {
        if !t.body.ends_with('\n') {
            t.body.push('\n');
        }
        t.body.push_str(&row);
        t.body.push('\n');
    } else {
        t.body.push_str(&format!(
            "\n## History\n\n| When | From | To | By |\n|------|------|----|----|\n{row}\n"
        ));
    }

    let content = t.serialize()?;
    let rel_path = format!(
        "{}/{}",
        config.tickets.dir.to_string_lossy(),
        t.path.file_name().unwrap().to_string_lossy()
    );
    let branch = t.frontmatter.branch.clone()
        .or_else(|| crate::git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));

    crate::git::commit_to_branch(root, &branch, &rel_path, &content, &format!("ticket({id}): close"))?;
    crate::logger::log("state_transition", &format!("{id:?} {prev} -> closed"));

    if let Err(e) = crate::git::merge_branch_into_default(root, &branch, &config.project.default_branch) {
        eprintln!("warning: merge into {} failed: {e:#}", config.project.default_branch);
    }

    if aggressive {
        if let Err(e) = crate::git::push_branch(root, &branch) {
            eprintln!("warning: push failed for {branch}: {e:#}");
        }
    }

    println!("{id}: {prev} → closed");
    Ok(())
}

pub fn create(
    root: &std::path::Path,
    config: &crate::config::Config,
    title: String,
    author: String,
    context: Option<String>,
    context_section: Option<String>,
    aggressive: bool,
    section_sets: Vec<(String, String)>,
) -> Result<Ticket> {
    let tickets_dir = root.join(&config.tickets.dir);
    std::fs::create_dir_all(&tickets_dir)?;

    let id = crate::git::gen_hex_id();
    let slug = slugify(&title);
    let filename = format!("{id}-{slug}.md");
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);
    let branch = format!("ticket/{id}-{slug}");
    let now = chrono::Utc::now();
    let fm = Frontmatter {
        id: id.clone(),
        title: title.clone(),
        state: "new".into(),
        priority: 0,
        effort: 0,
        risk: 0,
        author: Some(author.clone()),
        supervisor: None,
        agent: None,
        branch: Some(branch.clone()),
        created_at: Some(now),
        updated_at: Some(now),
        focus_section: None,
    };
    let when = now.format("%Y-%m-%dT%H:%MZ");
    let history_footer = format!("## History\n\n| When | From | To | By |\n|------|------|----|----|\n| {when} | — | new | {author} |\n");
    let body_template = if config.ticket.sections.is_empty() {
        format!("## Spec\n\n### Problem\n\n### Acceptance criteria\n\n### Out of scope\n\n### Approach\n\n{history_footer}")
    } else {
        let mut s = String::from("## Spec\n\n");
        for sec in &config.ticket.sections {
            let placeholder = sec.placeholder.as_deref().unwrap_or("");
            s.push_str(&format!("### {}\n\n{}\n\n", sec.name, placeholder));
        }
        s.push_str(&history_footer);
        s
    };
    let body = if let Some(ctx) = &context {
        let transition_section = config.workflow.states.iter()
            .find(|s| s.id == "new")
            .and_then(|s| s.transitions.iter().find(|tr| tr.to == "in_design"))
            .and_then(|tr| tr.context_section.clone());
        let section = context_section
            .clone()
            .or(transition_section)
            .unwrap_or_else(|| "Problem".to_string());
        let heading = format!("### {section}\n\n");
        if !body_template.contains(&heading) {
            anyhow::bail!("section '### {section}' not found in ticket body template");
        }
        body_template.replacen(&heading, &format!("### {section}\n\n{ctx}\n\n"), 1)
    } else {
        body_template
    };
    let path = tickets_dir.join(&filename);
    let mut t = Ticket { frontmatter: fm, body, path };

    if !section_sets.is_empty() {
        let config_active = !config.ticket.sections.is_empty();
        for (name, value) in &section_sets {
            let trimmed = value.trim().to_string();
            if config_active {
                let section_config = config.ticket.sections.iter()
                    .find(|s| s.name.eq_ignore_ascii_case(name))
                    .ok_or_else(|| anyhow::anyhow!("unknown section {:?}", name))?;
                let formatted = crate::spec::apply_section_type(&section_config.type_, trimmed);
                if crate::spec::is_doc_field(name) {
                    let mut doc = t.document()?;
                    crate::spec::set_section(&mut doc, name, formatted);
                    t.body = doc.serialize();
                } else {
                    crate::spec::set_section_body(&mut t.body, name, &formatted);
                }
            } else {
                let mut doc = t.document()?;
                crate::spec::set_section(&mut doc, name, trimmed);
                t.body = doc.serialize();
            }
        }
    }

    let content = t.serialize()?;

    crate::git::commit_to_branch(
        root,
        &branch,
        &rel_path,
        &content,
        &format!("ticket({id}): create {title}"),
    )?;

    if aggressive {
        if let Err(e) = crate::git::push_branch(root, &branch) {
            eprintln!("warning: push failed: {e:#}");
        }
    }

    Ok(t)
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

pub fn list_filtered<'a>(
    tickets: &'a [Ticket],
    config: &crate::config::Config,
    state_filter: Option<&str>,
    unassigned: bool,
    all: bool,
    supervisor_filter: Option<&str>,
    actionable_filter: Option<&str>,
) -> Vec<&'a Ticket> {
    let terminal: std::collections::HashSet<&str> = config.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();
    let actionable_map: std::collections::HashMap<&str, &Vec<String>> = config.workflow.states.iter()
        .map(|s| (s.id.as_str(), &s.actionable))
        .collect();

    tickets.iter().filter(|t| {
        let fm = &t.frontmatter;
        let state_ok = state_filter.map_or(true, |s| fm.state == s);
        let agent_ok = !unassigned || fm.agent.is_none();
        let state_is_terminal = state_filter.map_or(false, |s| terminal.contains(s));
        let terminal_ok = all || state_is_terminal || !terminal.contains(fm.state.as_str());
        let supervisor_ok = supervisor_filter.map_or(true, |s| fm.supervisor.as_deref() == Some(s));
        let actionable_ok = actionable_filter.map_or(true, |actor| {
            actionable_map.get(fm.state.as_str())
                .map_or(false, |actors| actors.iter().any(|a| a == actor || a == "any"))
        });
        state_ok && agent_ok && terminal_ok && supervisor_ok && actionable_ok
    }).collect()
}

pub fn set_field(fm: &mut Frontmatter, field: &str, value: &str) -> anyhow::Result<()> {
    match field {
        "priority" => fm.priority = value.parse().map_err(|_| anyhow::anyhow!("priority must be 0–255"))?,
        "effort"   => fm.effort   = value.parse().map_err(|_| anyhow::anyhow!("effort must be 0–255"))?,
        "risk"     => fm.risk     = value.parse().map_err(|_| anyhow::anyhow!("risk must be 0–255"))?,
        "author"   => anyhow::bail!("author is immutable"),
        "supervisor" => fm.supervisor = if value == "-" { None } else { Some(value.to_string()) },
        "agent"    => fm.agent    = if value == "-" { None } else { Some(value.to_string()) },
        "branch"   => fm.branch   = if value == "-" { None } else { Some(value.to_string()) },
        "title"    => fm.title    = value.to_string(),
        other => anyhow::bail!("unknown field: {other}"),
    }
    Ok(())
}

fn append_history_row(body: &mut String, from: &str, to: &str, when: &str, by: &str) {
    let row = format!("| {when} | {from} | {to} | {by} |");
    if body.contains("## History") {
        if !body.ends_with('\n') {
            body.push('\n');
        }
        body.push_str(&row);
        body.push('\n');
    } else {
        body.push_str(&format!(
            "\n## History\n\n| When | From | To | By |\n|------|------|----|----|\n{row}\n"
        ));
    }
}

pub fn handoff(ticket: &mut Ticket, new_agent: &str, now: DateTime<Utc>) -> Result<Option<String>> {
    let old_agent = match &ticket.frontmatter.agent {
        None => bail!("no agent assigned — use `apm start` instead"),
        Some(a) => a.clone(),
    };
    if old_agent == new_agent {
        return Ok(None);
    }
    ticket.frontmatter.agent = Some(new_agent.to_string());
    ticket.frontmatter.updated_at = Some(now);
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
    append_history_row(&mut ticket.body, &old_agent, new_agent, &when, "handoff");
    Ok(Some(old_agent))
}

pub fn list_worktrees_with_tickets(
    root: &Path,
    tickets_dir: &Path,
) -> Result<Vec<(std::path::PathBuf, String, Option<Ticket>)>> {
    let worktrees = crate::git::list_ticket_worktrees(root)?;
    let tickets = load_all_from_git(root, tickets_dir).unwrap_or_default();
    let result = worktrees.into_iter().map(|(wt_path, branch)| {
        let ticket = tickets.iter().find(|t| {
            t.frontmatter.branch.as_deref() == Some(branch.as_str())
                || crate::git::branch_name_from_path(&t.path).as_deref() == Some(branch.as_str())
        }).cloned();
        (wt_path, branch, ticket)
    }).collect();
    Ok(result)
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
        assert!(t.frontmatter.agent.is_none());
        assert!(t.frontmatter.branch.is_none());
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

    // ── list_filtered ─────────────────────────────────────────────────────

    fn test_config_with_states(terminal_states: &[&str]) -> crate::config::Config {
        let mut states_toml = String::new();
        for s in ["new", "ready", "in_progress"] {
            states_toml.push_str(&format!(
                "[[workflow.states]]\nid = \"{s}\"\nlabel = \"{s}\"\nterminal = false\nactionable = [\"agent\"]\n\n"
            ));
        }
        for s in terminal_states {
            states_toml.push_str(&format!(
                "[[workflow.states]]\nid = \"{s}\"\nlabel = \"{s}\"\nterminal = true\n\n"
            ));
        }
        let full = format!(
            "[project]\nname = \"test\"\n\n[tickets]\ndir = \"tickets\"\n\n{states_toml}"
        );
        toml::from_str(&full).unwrap()
    }

    fn make_ticket(id: &str, state: &str, agent: Option<&str>) -> Ticket {
        let agent_line = agent.map(|a| format!("agent = \"{a}\"\n")).unwrap_or_default();
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"T{id}\"\nstate = \"{state}\"\n{agent_line}+++\n\n"
        );
        Ticket::parse(dummy_path(), &raw).unwrap()
    }

    #[test]
    fn list_filtered_by_state() {
        let config = test_config_with_states(&["closed"]);
        let tickets = vec![
            make_ticket("0001", "new", None),
            make_ticket("0002", "ready", None),
            make_ticket("0003", "new", None),
        ];
        let result = list_filtered(&tickets, &config, Some("new"), false, false, None, None);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|t| t.frontmatter.state == "new"));
    }

    #[test]
    fn list_filtered_terminal_hidden_by_default() {
        let config = test_config_with_states(&["closed"]);
        let tickets = vec![
            make_ticket("0001", "new", None),
            make_ticket("0002", "closed", None),
        ];
        // By default, terminal states are hidden.
        let result = list_filtered(&tickets, &config, None, false, false, None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].frontmatter.state, "new");

        // With all=true, terminal states are shown.
        let result_all = list_filtered(&tickets, &config, None, false, true, None, None);
        assert_eq!(result_all.len(), 2);

        // With state_filter matching the terminal state, it's shown.
        let result_filtered = list_filtered(&tickets, &config, Some("closed"), false, false, None, None);
        assert_eq!(result_filtered.len(), 1);
        assert_eq!(result_filtered[0].frontmatter.state, "closed");
    }

    #[test]
    fn list_filtered_unassigned() {
        let config = test_config_with_states(&[]);
        let tickets = vec![
            make_ticket("0001", "new", None),
            make_ticket("0002", "new", Some("alice")),
            make_ticket("0003", "ready", None),
        ];
        let result = list_filtered(&tickets, &config, None, true, false, None, None);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|t| t.frontmatter.agent.is_none()));
    }

    // ── set_field ─────────────────────────────────────────────────────────

    fn make_frontmatter() -> Frontmatter {
        Frontmatter {
            id: "0001".to_string(),
            title: "Test".to_string(),
            state: "new".to_string(),
            priority: 0,
            effort: 0,
            risk: 0,
            author: None,
            supervisor: None,
            agent: None,
            branch: None,
            created_at: None,
            updated_at: None,
            focus_section: None,
        }
    }

    #[test]
    fn set_field_priority_valid() {
        let mut fm = make_frontmatter();
        set_field(&mut fm, "priority", "5").unwrap();
        assert_eq!(fm.priority, 5);
    }

    #[test]
    fn set_field_priority_overflow() {
        let mut fm = make_frontmatter();
        let err = set_field(&mut fm, "priority", "256").unwrap_err();
        assert!(err.to_string().contains("priority must be 0"));
    }

    #[test]
    fn set_field_author_immutable() {
        let mut fm = make_frontmatter();
        let err = set_field(&mut fm, "author", "alice").unwrap_err();
        assert!(err.to_string().contains("author is immutable"));
    }

    #[test]
    fn set_field_unknown_field() {
        let mut fm = make_frontmatter();
        let err = set_field(&mut fm, "foo", "bar").unwrap_err();
        assert!(err.to_string().contains("unknown field: foo"));
    }

    #[test]
    fn set_field_agent_clear() {
        let mut fm = make_frontmatter();
        fm.agent = Some("alice".to_string());
        set_field(&mut fm, "agent", "-").unwrap();
        assert!(fm.agent.is_none());
    }

    // ── handoff ───────────────────────────────────────────────────────────

    fn make_ticket_with_agent(agent: Option<&str>) -> Ticket {
        make_ticket("0001", "in_progress", agent)
    }

    #[test]
    fn handoff_no_agent_errors() {
        let mut t = make_ticket_with_agent(None);
        let now = chrono::Utc::now();
        let err = handoff(&mut t, "bob", now).unwrap_err();
        assert!(err.to_string().contains("no agent assigned"));
    }

    #[test]
    fn handoff_idempotent() {
        let mut t = make_ticket_with_agent(Some("alice"));
        let now = chrono::Utc::now();
        let result = handoff(&mut t, "alice", now).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn handoff_successful() {
        let mut t = make_ticket_with_agent(Some("alice"));
        let now = chrono::Utc::now();
        let result = handoff(&mut t, "bob", now).unwrap();
        assert_eq!(result, Some("alice".to_string()));
        assert_eq!(t.frontmatter.agent.as_deref(), Some("bob"));
        assert!(t.body.contains("## History"));
        assert!(t.body.contains("handoff"));
    }
}
