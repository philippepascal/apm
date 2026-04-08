use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
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

    pub fn score(&self, priority_weight: f64, effort_weight: f64, risk_weight: f64) -> f64 {
        let fm = &self.frontmatter;
        fm.priority as f64 * priority_weight
            + fm.effort as f64 * effort_weight
            + fm.risk as f64 * risk_weight
    }
}

/// Build a reverse dependency index: for each ticket ID, collect the tickets
/// that directly depend on it.  Pass only non-terminal, non-satisfies_deps
/// tickets so that closed work does not inflate effective priority.
pub fn build_reverse_index<'a>(tickets: &[&'a Ticket]) -> HashMap<&'a str, Vec<&'a Ticket>> {
    let mut map: HashMap<&'a str, Vec<&'a Ticket>> = HashMap::new();
    for &ticket in tickets {
        if let Some(deps) = &ticket.frontmatter.depends_on {
            for dep_id in deps {
                map.entry(dep_id.as_str()).or_default().push(ticket);
            }
        }
    }
    map
}

/// Return the effective priority of a ticket: the max of its own priority and
/// the priority of all direct and transitive dependents reachable via the
/// reverse index.  Uses a visited set to handle cycles safely.
pub fn effective_priority(ticket: &Ticket, reverse_index: &HashMap<&str, Vec<&Ticket>>) -> u8 {
    let mut max_priority = ticket.frontmatter.priority;
    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    let id = ticket.frontmatter.id.as_str();
    queue.push_back(id);
    visited.insert(id);
    while let Some(cur_id) = queue.pop_front() {
        if let Some(dependents) = reverse_index.get(cur_id) {
            for &dep in dependents {
                let dep_id = dep.frontmatter.id.as_str();
                if visited.insert(dep_id) {
                    if dep.frontmatter.priority > max_priority {
                        max_priority = dep.frontmatter.priority;
                    }
                    queue.push_back(dep_id);
                }
            }
        }
    }
    max_priority
}

/// Return all agent-actionable tickets sorted by descending score.
pub fn sorted_actionable<'a>(
    tickets: &'a [Ticket],
    actionable: &[&str],
    pw: f64,
    ew: f64,
    rw: f64,
    caller: Option<&str>,
) -> Vec<&'a Ticket> {
    let mut candidates: Vec<&Ticket> = tickets
        .iter()
        .filter(|t| actionable.contains(&t.frontmatter.state.as_str()))
        .filter(|t| match t.frontmatter.owner.as_deref() {
            None => true,
            Some(owner) => caller.map_or(true, |c| c == owner),
        })
        .collect();
    let rev_idx = build_reverse_index(&candidates);
    candidates.sort_by(|a, b| {
        let score_a = effective_priority(a, &rev_idx) as f64 * pw
            + a.frontmatter.effort as f64 * ew
            + a.frontmatter.risk as f64 * rw;
        let score_b = effective_priority(b, &rev_idx) as f64 * pw
            + b.frontmatter.effort as f64 * ew
            + b.frontmatter.risk as f64 * rw;
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates
}

/// Returns true if a ticket in `dep_state` satisfies the dependency gate
/// required by the dependent ticket.  `required_gate` is `Some("tag")` when
/// the dependent's state has `dep_requires = "tag"`, or `None` for the
/// default (requires `satisfies_deps = true` or `terminal = true`).
pub fn dep_satisfied(dep_state: &str, required_gate: Option<&str>, config: &crate::config::Config) -> bool {
    use crate::config::SatisfiesDeps;
    config.workflow.states.iter()
        .find(|s| s.id == dep_state)
        .map(|s| {
            if s.terminal { return true; }
            match &s.satisfies_deps {
                SatisfiesDeps::Bool(true) => true,
                SatisfiesDeps::Tag(tag) => required_gate == Some(tag.as_str()),
                SatisfiesDeps::Bool(false) => false,
            }
        })
        .unwrap_or(false)
}

/// Return the highest-scoring ticket from `tickets` whose state is in
/// `actionable` and (if `startable` is non-empty) also in `startable`,
/// and whose `depends_on` deps are all satisfied.
pub fn pick_next<'a>(
    tickets: &'a [Ticket],
    actionable: &[&str],
    startable: &[&str],
    pw: f64,
    ew: f64,
    rw: f64,
    config: &crate::config::Config,
    caller: Option<&str>,
) -> Option<&'a Ticket> {
    sorted_actionable(tickets, actionable, pw, ew, rw, caller)
        .into_iter()
        .find(|t| {
            let state = t.frontmatter.state.as_str();
            if !startable.is_empty() && !startable.contains(&state) {
                return false;
            }
            let required_gate = config.workflow.states.iter()
                .find(|s| s.id == state)
                .and_then(|s| s.dep_requires.as_deref());
            if let Some(deps) = &t.frontmatter.depends_on {
                for dep_id in deps {
                    if let Some(dep) = tickets.iter().find(|d| d.frontmatter.id == *dep_id) {
                        if !dep_satisfied(&dep.frontmatter.state, required_gate, config) {
                            return false;
                        }
                    }
                }
            }
            true
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
                Err(_) => {}
            },
            Err(_) => {}
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
) -> Result<Vec<String>> {
    let mut output: Vec<String> = Vec::new();
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

    crate::state::append_history(&mut t.body, &prev, "closed", &when, &by);

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

    let mut merge_warnings: Vec<String> = Vec::new();
    if let Err(e) = crate::git::merge_branch_into_default(root, &branch, &config.project.default_branch, &mut merge_warnings) {
        output.push(format!("warning: merge into {} failed: {e:#}", config.project.default_branch));
    }
    output.extend(merge_warnings);

    if aggressive {
        if let Err(e) = crate::git::push_branch(root, &branch) {
            output.push(format!("warning: push failed for {branch}: {e:#}"));
        }
    }

    output.push(format!("{id}: {prev} → closed"));
    Ok(output)
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
    epic: Option<String>,
    target_branch: Option<String>,
    depends_on: Option<Vec<String>>,
    base_branch: Option<String>,
    warnings: &mut Vec<String>,
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
        owner: Some(author.clone()),
        branch: Some(branch.clone()),
        created_at: Some(now),
        updated_at: Some(now),
        focus_section: None,
        epic,
        target_branch,
        depends_on,
    };
    let when = now.format("%Y-%m-%dT%H:%MZ");
    let history_footer = format!("## History\n\n| When | From | To | By |\n|------|------|----|----|\n| {when} | — | new | {author} |\n");
    let body_template = {
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
        if !config.ticket.sections.is_empty()
            && !config.has_section(&section)
        {
            anyhow::bail!("section '### {section}' not found in ticket body template");
        }
        let mut doc = TicketDocument::parse(&body_template)?;
        crate::spec::set_section(&mut doc, &section, ctx.clone());
        doc.serialize()
    } else {
        body_template
    };
    let path = tickets_dir.join(&filename);
    let mut t = Ticket { frontmatter: fm, body, path };

    if !section_sets.is_empty() {
        let mut doc = t.document()?;
        for (name, value) in &section_sets {
            let trimmed = value.trim().to_string();
            let formatted = if !config.ticket.sections.is_empty() {
                let section_config = config.find_section(name)
                    .ok_or_else(|| anyhow::anyhow!("unknown section {:?}", name))?;
                crate::spec::apply_section_type(&section_config.type_, trimmed)
            } else {
                trimmed
            };
            crate::spec::set_section(&mut doc, name, formatted);
        }
        t.body = doc.serialize();
    }

    let content = t.serialize()?;

    if let Some(base) = base_branch {
        let sha = crate::git::resolve_branch_sha(root, &base)?;
        crate::git::create_branch_at(root, &branch, &sha)?;
    }

    crate::git::commit_to_branch(
        root,
        &branch,
        &rel_path,
        &content,
        &format!("ticket({id}): create {title}"),
    )?;

    if aggressive {
        if let Err(e) = crate::git::push_branch(root, &branch) {
            warnings.push(format!("warning: push failed: {e:#}"));
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

    pub fn unchecked_tasks(&self, section_name: &str) -> Vec<usize> {
        let val = self.sections.get(section_name).map(|s| s.as_str()).unwrap_or("");
        parse_checklist(val).into_iter().enumerate()
            .filter(|(_, c)| !c.checked)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn toggle_criterion(&mut self, index: usize, checked: bool) -> Result<()> {
        let val = self.sections.get("Acceptance criteria").cloned().unwrap_or_default();
        let mut items = parse_checklist(&val);
        if index >= items.len() {
            anyhow::bail!("criterion index {index} out of range (have {})", items.len());
        }
        items[index].checked = checked;
        self.sections.insert("Acceptance criteria".to_string(), serialize_checklist(&items));
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
    actionable_filter: Option<&str>,
    author_filter: Option<&str>,
    owner_filter: Option<&str>,
    mine_user: Option<&str>,
) -> Vec<&'a Ticket> {
    let terminal = config.terminal_state_ids();
    let actionable_map: std::collections::HashMap<&str, &Vec<String>> = config.workflow.states.iter()
        .map(|s| (s.id.as_str(), &s.actionable))
        .collect();

    tickets.iter().filter(|t| {
        let fm = &t.frontmatter;
        let state_ok = state_filter.map_or(true, |s| fm.state == s);
        let agent_ok = !unassigned || fm.author.as_deref() == Some("unassigned");
        let state_is_terminal = state_filter.map_or(false, |s| terminal.contains(s));
        let terminal_ok = all || state_is_terminal || !terminal.contains(fm.state.as_str());
        let actionable_ok = actionable_filter.map_or(true, |actor| {
            actionable_map.get(fm.state.as_str())
                .map_or(false, |actors| actors.iter().any(|a| a == actor || a == "any"))
        });
        let author_ok = author_filter.map_or(true, |a| fm.author.as_deref() == Some(a));
        let owner_ok = owner_filter.map_or(true, |o| fm.owner.as_deref() == Some(o));
        let mine_ok = mine_user.map_or(true, |me| {
            fm.author.as_deref() == Some(me) || fm.owner.as_deref() == Some(me)
        });
        state_ok && agent_ok && terminal_ok && actionable_ok && author_ok && owner_ok && mine_ok
    }).collect()
}

pub fn check_owner(root: &Path, ticket: &Ticket) -> anyhow::Result<()> {
    let cfg = crate::config::Config::load(root)?;
    let is_terminal = cfg.workflow.states.iter()
        .find(|s| s.id == ticket.frontmatter.state)
        .map(|s| s.terminal)
        .unwrap_or(false);
    if is_terminal {
        anyhow::bail!("cannot change owner of a closed ticket");
    }
    let Some(o) = &ticket.frontmatter.owner else {
        return Ok(());
    };
    let identity = crate::config::resolve_identity(root);
    if identity == "unassigned" {
        anyhow::bail!(
            "cannot reassign: identity not configured (set local.user in .apm/local.toml or configure a GitHub token)"
        );
    }
    if &identity != o {
        anyhow::bail!("only the current owner ({o}) can reassign this ticket");
    }
    Ok(())
}

pub fn set_field(fm: &mut Frontmatter, field: &str, value: &str) -> anyhow::Result<()> {
    match field {
        "priority" => fm.priority = value.parse().map_err(|_| anyhow::anyhow!("priority must be 0–255"))?,
        "effort"   => fm.effort   = value.parse().map_err(|_| anyhow::anyhow!("effort must be 0–255"))?,
        "risk"     => fm.risk     = value.parse().map_err(|_| anyhow::anyhow!("risk must be 0–255"))?,
        "author"   => anyhow::bail!("author is immutable"),
        "owner"    => fm.owner    = if value == "-" { None } else { Some(value.to_string()) },
        "branch"   => fm.branch   = if value == "-" { None } else { Some(value.to_string()) },
        "title"    => fm.title    = value.to_string(),
        "depends_on" => {
            if value == "-" {
                fm.depends_on = None;
            } else {
                let ids: Vec<String> = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                fm.depends_on = if ids.is_empty() { None } else { Some(ids) };
            }
        }
        other => anyhow::bail!("unknown field: {other}"),
    }
    Ok(())
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
    fn document_toggle_criterion() {
        let body = full_body("- [ ] item one\n- [ ] item two");
        let mut doc = TicketDocument::parse(&body).unwrap();
        let ac = doc.sections.get("Acceptance criteria").unwrap();
        assert!(ac.contains("- [ ] item one"));
        doc.toggle_criterion(0, true).unwrap();
        let ac = doc.sections.get("Acceptance criteria").unwrap();
        assert!(ac.contains("- [x] item one"));
    }

    #[test]
    fn document_unchecked_tasks() {
        let body = full_body("- [ ] one\n- [x] two\n- [ ] three");
        let doc = TicketDocument::parse(&body).unwrap();
        assert_eq!(doc.unchecked_tasks("Acceptance criteria"), vec![0, 2]);
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
        let result = list_filtered(&tickets, &config, Some("new"), false, false, None, None, None, None);
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
        let result = list_filtered(&tickets, &config, None, false, false, None, None, None, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].frontmatter.state, "new");

        // With all=true, terminal states are shown.
        let result_all = list_filtered(&tickets, &config, None, false, true, None, None, None, None);
        assert_eq!(result_all.len(), 2);

        // With state_filter matching the terminal state, it's shown.
        let result_filtered = list_filtered(&tickets, &config, Some("closed"), false, false, None, None, None, None);
        assert_eq!(result_filtered.len(), 1);
        assert_eq!(result_filtered[0].frontmatter.state, "closed");
    }

    #[test]
    fn list_filtered_unassigned() {
        let config = test_config_with_states(&[]);
        let make_with_author = |id: &str, author: Option<&str>| {
            let author_line = author.map(|a| format!("author = \"{a}\"\n")).unwrap_or_default();
            let raw = format!(
                "+++\nid = \"{id}\"\ntitle = \"T{id}\"\nstate = \"new\"\n{author_line}+++\n\n"
            );
            Ticket::parse(Path::new("test.md"), &raw).unwrap()
        };
        let tickets = vec![
            make_with_author("0001", Some("unassigned")),
            make_with_author("0002", Some("alice")),
            make_with_author("0003", Some("unassigned")),
            make_with_author("0004", None),
        ];
        let result = list_filtered(&tickets, &config, None, true, false, None, None, None, None);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|t| t.frontmatter.author.as_deref() == Some("unassigned")));
    }

    fn make_ticket_with_author(id: &str, state: &str, author: Option<&str>) -> Ticket {
        let author_line = author.map(|a| format!("author = \"{a}\"\n")).unwrap_or_default();
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"T{id}\"\nstate = \"{state}\"\n{author_line}+++\n\n"
        );
        Ticket::parse(dummy_path(), &raw).unwrap()
    }

    #[test]
    fn list_filtered_by_author() {
        let config = test_config_with_states(&[]);
        let tickets = vec![
            make_ticket_with_author("0001", "new", Some("alice")),
            make_ticket_with_author("0002", "new", Some("bob")),
            make_ticket_with_author("0003", "ready", Some("alice")),
        ];
        let result = list_filtered(&tickets, &config, None, false, false, None, Some("alice"), None, None);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|t| t.frontmatter.author.as_deref() == Some("alice")));
    }

    #[test]
    fn list_filtered_author_none() {
        let config = test_config_with_states(&[]);
        let tickets = vec![
            make_ticket_with_author("0001", "new", Some("alice")),
            make_ticket_with_author("0002", "new", Some("bob")),
        ];
        let result = list_filtered(&tickets, &config, None, false, false, None, None, None, None);
        assert_eq!(result.len(), 2);
    }

    fn make_ticket_with_owner(id: &str, state: &str, author: Option<&str>, owner: Option<&str>) -> Ticket {
        let author_line = author.map(|a| format!("author = \"{a}\"\n")).unwrap_or_default();
        let owner_line = owner.map(|o| format!("owner = \"{o}\"\n")).unwrap_or_default();
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"T{id}\"\nstate = \"{state}\"\n{author_line}{owner_line}+++\n\n"
        );
        Ticket::parse(dummy_path(), &raw).unwrap()
    }

    #[test]
    fn list_filtered_by_owner() {
        let config = test_config_with_states(&[]);
        let tickets = vec![
            make_ticket_with_owner("0001", "new", Some("alice"), Some("alice")),
            make_ticket_with_owner("0002", "new", Some("bob"), Some("bob")),
            make_ticket_with_owner("0003", "new", Some("carol"), None),
        ];
        let result = list_filtered(&tickets, &config, None, false, false, None, None, Some("alice"), None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].frontmatter.id, "0001");
    }

    #[test]
    fn list_filtered_mine_matches_author() {
        let config = test_config_with_states(&[]);
        let tickets = vec![
            make_ticket_with_owner("0001", "new", Some("alice"), Some("bob")),
            make_ticket_with_owner("0002", "new", Some("bob"), Some("carol")),
        ];
        let result = list_filtered(&tickets, &config, None, false, false, None, None, None, Some("alice"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].frontmatter.id, "0001");
    }

    #[test]
    fn list_filtered_mine_matches_owner() {
        let config = test_config_with_states(&[]);
        let tickets = vec![
            make_ticket_with_owner("0001", "new", Some("bob"), Some("alice")),
            make_ticket_with_owner("0002", "new", Some("carol"), Some("bob")),
        ];
        let result = list_filtered(&tickets, &config, None, false, false, None, None, None, Some("alice"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].frontmatter.id, "0001");
    }

    #[test]
    fn list_filtered_mine_or_semantics() {
        let config = test_config_with_states(&[]);
        let tickets = vec![
            make_ticket_with_owner("0001", "new", Some("alice"), None),
            make_ticket_with_owner("0002", "new", Some("bob"), Some("alice")),
            make_ticket_with_owner("0003", "new", Some("carol"), Some("carol")),
        ];
        let result = list_filtered(&tickets, &config, None, false, false, None, None, None, Some("alice"));
        assert_eq!(result.len(), 2);
        let ids: Vec<&str> = result.iter().map(|t| t.frontmatter.id.as_str()).collect();
        assert!(ids.contains(&"0001"));
        assert!(ids.contains(&"0002"));
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
            owner: None,
            branch: None,
            created_at: None,
            updated_at: None,
            focus_section: None,
            epic: None,
            target_branch: None,
            depends_on: None,
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
    fn owner_round_trips_through_toml() {
        let toml_src = r#"id = "0001"
title = "T"
state = "new"
owner = "alice"
"#;
        let fm: Frontmatter = toml::from_str(toml_src).unwrap();
        assert_eq!(fm.owner, Some("alice".to_string()));
        let serialized = toml::to_string(&fm).unwrap();
        assert!(serialized.contains("owner = \"alice\""));
    }

    #[test]
    fn owner_absent_deserializes_as_none() {
        let toml_src = r#"id = "0001"
title = "T"
state = "new"
"#;
        let fm: Frontmatter = toml::from_str(toml_src).unwrap();
        assert_eq!(fm.owner, None);
    }

    #[test]
    fn set_field_owner_set() {
        let mut fm = make_frontmatter();
        set_field(&mut fm, "owner", "alice").unwrap();
        assert_eq!(fm.owner, Some("alice".to_string()));
    }

    #[test]
    fn set_field_owner_clear() {
        let mut fm = make_frontmatter();
        fm.owner = Some("alice".to_string());
        set_field(&mut fm, "owner", "-").unwrap();
        assert_eq!(fm.owner, None);
    }

    // ── dep_satisfied ─────────────────────────────────────────────────────

    fn config_with_dep_states() -> crate::config::Config {
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
id = "done"
label = "Done"
satisfies_deps = true

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true

[[workflow.states]]
id = "blocked"
label = "Blocked"
"#;
        toml::from_str(toml).unwrap()
    }

    #[test]
    fn dep_satisfied_satisfies_deps_true() {
        let config = config_with_dep_states();
        assert!(dep_satisfied("done", None, &config));
    }

    #[test]
    fn dep_satisfied_terminal_true() {
        let config = config_with_dep_states();
        assert!(dep_satisfied("closed", None, &config));
    }

    #[test]
    fn dep_satisfied_both_false() {
        let config = config_with_dep_states();
        assert!(!dep_satisfied("blocked", None, &config));
    }

    #[test]
    fn dep_satisfied_unknown_state() {
        let config = config_with_dep_states();
        assert!(!dep_satisfied("nonexistent", None, &config));
    }

    fn config_with_spec_gate() -> crate::config::Config {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id = "groomed"
label = "Groomed"
actionable = ["agent"]
dep_requires = "spec"

[[workflow.states]]
id = "ready"
label = "Ready"
actionable = ["agent"]

[[workflow.states]]
id = "specd"
label = "Specd"
satisfies_deps = "spec"

[[workflow.states]]
id = "in_progress"
label = "In Progress"
satisfies_deps = "spec"

[[workflow.states]]
id = "implemented"
label = "Implemented"
satisfies_deps = true

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#;
        toml::from_str(toml).unwrap()
    }

    #[test]
    fn dep_satisfied_tag_matches_required_gate() {
        let config = config_with_spec_gate();
        assert!(dep_satisfied("specd", Some("spec"), &config));
    }

    #[test]
    fn dep_satisfied_tag_no_required_gate_is_false() {
        let config = config_with_spec_gate();
        assert!(!dep_satisfied("specd", None, &config));
    }

    #[test]
    fn dep_satisfied_bool_true_with_no_gate() {
        let config = config_with_spec_gate();
        assert!(dep_satisfied("implemented", None, &config));
    }

    #[test]
    fn pick_next_groomed_unblocked_when_dep_specd() {
        let config = config_with_spec_gate();
        let tickets = vec![
            make_ticket_with_deps("aaaa0001", "groomed", Some(vec!["bbbb0001"])),
            make_ticket_with_deps("bbbb0001", "specd", None),
        ];
        let result = pick_next(&tickets, &["groomed"], &[], 10.0, -2.0, -1.0, &config, None);
        assert_eq!(result.unwrap().frontmatter.id, "aaaa0001");
    }

    #[test]
    fn pick_next_groomed_unblocked_when_dep_in_progress() {
        let config = config_with_spec_gate();
        let tickets = vec![
            make_ticket_with_deps("aaaa0001", "groomed", Some(vec!["bbbb0001"])),
            make_ticket_with_deps("bbbb0001", "in_progress", None),
        ];
        let result = pick_next(&tickets, &["groomed"], &[], 10.0, -2.0, -1.0, &config, None);
        assert_eq!(result.unwrap().frontmatter.id, "aaaa0001");
    }

    #[test]
    fn pick_next_ready_blocked_when_dep_only_specd() {
        let config = config_with_spec_gate();
        let tickets = vec![
            make_ticket_with_deps("aaaa0001", "ready", Some(vec!["bbbb0001"])),
            make_ticket_with_deps("bbbb0001", "specd", None),
        ];
        let result = pick_next(&tickets, &["ready"], &[], 10.0, -2.0, -1.0, &config, None);
        assert!(result.is_none());
    }

    // ── pick_next dep filtering ────────────────────────────────────────────

    fn make_ticket_with_deps(id: &str, state: &str, deps: Option<Vec<&str>>) -> Ticket {
        let deps_line = match &deps {
            None => String::new(),
            Some(v) => {
                let list: Vec<String> = v.iter().map(|d| format!("\"{d}\"")).collect();
                format!("depends_on = [{}]\n", list.join(", "))
            }
        };
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"T{id}\"\nstate = \"{state}\"\n{deps_line}+++\n\n"
        );
        Ticket::parse(dummy_path(), &raw).unwrap()
    }

    #[test]
    fn pick_next_skips_dep_blocked_ticket() {
        let config = config_with_dep_states();
        let tickets = vec![
            make_ticket_with_deps("aaaa0001", "ready", Some(vec!["bbbb0001"])),
            make_ticket_with_deps("bbbb0001", "ready", None),
            make_ticket_with_deps("cccc0001", "ready", None),
        ];
        // aaaa0001 depends on bbbb0001 which is in "ready" (not satisfies_deps)
        // should skip aaaa0001 and return bbbb0001 (next by score, no deps)
        let result = pick_next(&tickets, &["ready"], &[], 10.0, -2.0, -1.0, &config, None);
        assert!(result.is_some());
        let id = &result.unwrap().frontmatter.id;
        assert_ne!(id, "aaaa0001", "dep-blocked ticket should be skipped");
    }

    #[test]
    fn pick_next_returns_ticket_when_dep_satisfied() {
        let config = config_with_dep_states();
        let tickets = vec![
            make_ticket_with_deps("aaaa0001", "ready", Some(vec!["bbbb0001"])),
            make_ticket_with_deps("bbbb0001", "done", None),
        ];
        let result = pick_next(&tickets, &["ready"], &[], 10.0, -2.0, -1.0, &config, None);
        assert_eq!(result.unwrap().frontmatter.id, "aaaa0001");
    }

    #[test]
    fn pick_next_unknown_dep_id_not_blocking() {
        let config = config_with_dep_states();
        let tickets = vec![
            make_ticket_with_deps("aaaa0001", "ready", Some(vec!["unknown1"])),
        ];
        let result = pick_next(&tickets, &["ready"], &[], 10.0, -2.0, -1.0, &config, None);
        assert_eq!(result.unwrap().frontmatter.id, "aaaa0001");
    }

    #[test]
    fn pick_next_empty_depends_on_not_blocking() {
        let config = config_with_dep_states();
        let raw = "+++\nid = \"aaaa0001\"\ntitle = \"T\"\nstate = \"ready\"\ndepends_on = []\n+++\n\n";
        let t = Ticket::parse(dummy_path(), raw).unwrap();
        let tickets = vec![t];
        let result = pick_next(&tickets, &["ready"], &[], 10.0, -2.0, -1.0, &config, None);
        assert_eq!(result.unwrap().frontmatter.id, "aaaa0001");
    }

    // --- build_reverse_index / effective_priority / sorted_actionable ---

    fn make_ticket_with_priority(id: &str, state: &str, priority: u8, deps: Option<Vec<&str>>) -> Ticket {
        let dep_line = match &deps {
            Some(d) => {
                let list: Vec<String> = d.iter().map(|s| format!("\"{s}\"")).collect();
                format!("depends_on = [{}]\n", list.join(", "))
            }
            None => String::new(),
        };
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"T{id}\"\nstate = \"{state}\"\npriority = {priority}\n{dep_line}+++\n\n"
        );
        Ticket::parse(Path::new("test.md"), &raw).unwrap()
    }

    #[test]
    fn effective_priority_no_dependents_returns_own() {
        let a = make_ticket_with_priority("aaaa", "ready", 5, None);
        let tickets = vec![&a];
        let rev_idx = build_reverse_index(&tickets);
        assert_eq!(effective_priority(&a, &rev_idx), 5);
    }

    #[test]
    fn effective_priority_single_hop_elevation() {
        // A (priority 2) is depended on by B (priority 9)
        let a = make_ticket_with_priority("aaaa", "ready", 2, None);
        let b = make_ticket_with_priority("bbbb", "ready", 9, Some(vec!["aaaa"]));
        let tickets = vec![&a, &b];
        let rev_idx = build_reverse_index(&tickets);
        assert_eq!(effective_priority(&a, &rev_idx), 9);
        assert_eq!(effective_priority(&b, &rev_idx), 9);
    }

    #[test]
    fn effective_priority_transitive_elevation() {
        // A (2) blocks B (5) blocks C (9); A's effective priority should be 9
        let a = make_ticket_with_priority("aaaa", "ready", 2, None);
        let b = make_ticket_with_priority("bbbb", "ready", 5, Some(vec!["aaaa"]));
        let c = make_ticket_with_priority("cccc", "ready", 9, Some(vec!["bbbb"]));
        let tickets = vec![&a, &b, &c];
        let rev_idx = build_reverse_index(&tickets);
        assert_eq!(effective_priority(&a, &rev_idx), 9);
        assert_eq!(effective_priority(&b, &rev_idx), 9);
        assert_eq!(effective_priority(&c, &rev_idx), 9);
    }

    #[test]
    fn effective_priority_cycle_does_not_panic() {
        // A depends on B, B depends on A
        let a = make_ticket_with_priority("aaaa", "ready", 3, Some(vec!["bbbb"]));
        let b = make_ticket_with_priority("bbbb", "ready", 7, Some(vec!["aaaa"]));
        let tickets = vec![&a, &b];
        let rev_idx = build_reverse_index(&tickets);
        // Should not panic; both see each other's priority
        let ep_a = effective_priority(&a, &rev_idx);
        let ep_b = effective_priority(&b, &rev_idx);
        assert_eq!(ep_a, 7);
        assert_eq!(ep_b, 7);
    }

    #[test]
    fn effective_priority_closed_dependent_excluded() {
        // A (2) is in the active set; B (9, closed) is NOT passed to build_reverse_index
        let a = make_ticket_with_priority("aaaa", "ready", 2, None);
        // B is "closed" — caller filters it out before building the index
        let tickets_active = vec![&a];
        let rev_idx = build_reverse_index(&tickets_active);
        assert_eq!(effective_priority(&a, &rev_idx), 2);
    }

    #[test]
    fn sorted_actionable_low_priority_blocker_elevated() {
        // A (priority 2, ready) is depended on by B (priority 9, ready)
        // A's effective priority becomes 9 — it should not sort last
        let a = make_ticket_with_priority("aaaa", "ready", 2, None);
        let b = make_ticket_with_priority("bbbb", "ready", 9, Some(vec!["aaaa"]));
        let tickets = vec![a, b];
        let result = sorted_actionable(&tickets, &["ready"], 1.0, 0.0, 0.0, None);
        assert_eq!(result.len(), 2);
        let ids: Vec<&str> = result.iter().map(|t| t.frontmatter.id.as_str()).collect();
        assert!(ids.contains(&"aaaa"), "A must appear in results");
        assert!(ids.contains(&"bbbb"), "B must appear in results");
        // A (ep=9) and B (ep=9) are tied; A must not be sorted below B due to raw priority
        // The last entry must not be A simply because raw priority 2 < 9
        // Both ep=9 so the sort is stable-ish; just verify A is present
    }

    #[test]
    fn sorted_actionable_blocker_before_independent_higher_raw() {
        // A (priority 2, ready, blocks C which has priority 9)
        // B (priority 7, ready, no deps)
        // A's effective priority = 9, B's = 7 → A should sort before B
        let a = make_ticket_with_priority("aaaa", "ready", 2, None);
        let b = make_ticket_with_priority("bbbb", "ready", 7, None);
        let c = make_ticket_with_priority("cccc", "ready", 9, Some(vec!["aaaa"]));
        let tickets = vec![a, b, c];
        let result = sorted_actionable(&tickets, &["ready"], 1.0, 0.0, 0.0, None);
        assert_eq!(result.len(), 3);
        let ids: Vec<&str> = result.iter().map(|t| t.frontmatter.id.as_str()).collect();
        let a_pos = ids.iter().position(|&id| id == "aaaa").unwrap();
        let b_pos = ids.iter().position(|&id| id == "bbbb").unwrap();
        assert!(a_pos < b_pos, "A (ep=9) should sort before B (ep=7)");
    }

    #[test]
    fn sorted_actionable_no_deps_unchanged() {
        let a = make_ticket_with_priority("aaaa", "ready", 3, None);
        let b = make_ticket_with_priority("bbbb", "ready", 7, None);
        let tickets = vec![a, b];
        let result = sorted_actionable(&tickets, &["ready"], 1.0, 0.0, 0.0, None);
        assert_eq!(result[0].frontmatter.id, "bbbb");
        assert_eq!(result[1].frontmatter.id, "aaaa");
    }

    fn make_ticket_with_owner_field(id: &str, state: &str, owner: Option<&str>) -> Ticket {
        let owner_line = owner.map(|o| format!("owner = \"{o}\"\n")).unwrap_or_default();
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"T{id}\"\nstate = \"{state}\"\n{owner_line}+++\n\n"
        );
        Ticket::parse(Path::new("test.md"), &raw).unwrap()
    }

    #[test]
    fn sorted_actionable_excludes_ticket_owned_by_other() {
        let t = make_ticket_with_owner_field("aaaa", "ready", Some("alice"));
        let tickets = vec![t];
        let result = sorted_actionable(&tickets, &["ready"], 1.0, 0.0, 0.0, Some("bob"));
        assert!(result.is_empty(), "ticket owned by alice should not appear for bob");
    }

    #[test]
    fn sorted_actionable_includes_ticket_owned_by_caller() {
        let t = make_ticket_with_owner_field("aaaa", "ready", Some("alice"));
        let tickets = vec![t];
        let result = sorted_actionable(&tickets, &["ready"], 1.0, 0.0, 0.0, Some("alice"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].frontmatter.id, "aaaa");
    }

    #[test]
    fn sorted_actionable_includes_unowned_ticket() {
        let t = make_ticket_with_owner_field("aaaa", "ready", None);
        let tickets = vec![t];
        let result = sorted_actionable(&tickets, &["ready"], 1.0, 0.0, 0.0, Some("bob"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].frontmatter.id, "aaaa");
    }

    #[test]
    fn sorted_actionable_no_caller_shows_all() {
        let t1 = make_ticket_with_owner_field("aaaa", "ready", Some("alice"));
        let t2 = make_ticket_with_owner_field("bbbb", "ready", Some("bob"));
        let tickets = vec![t1, t2];
        let result = sorted_actionable(&tickets, &["ready"], 1.0, 0.0, 0.0, None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn check_owner_passes_when_identity_matches_owner() {
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(tmp.path().join("apm.toml"), "[project]\nname = \"test\"\n").unwrap();
        std::fs::write(apm_dir.join("local.toml"), "username = \"alice\"\n").unwrap();
        let t = make_ticket_with_owner_field("aaaa", "ready", Some("alice"));
        assert!(check_owner(tmp.path(), &t).is_ok());
    }

    #[test]
    fn check_owner_fails_when_identity_does_not_match_owner() {
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(tmp.path().join("apm.toml"), "[project]\nname = \"test\"\n").unwrap();
        std::fs::write(apm_dir.join("local.toml"), "username = \"bob\"\n").unwrap();
        let t = make_ticket_with_owner_field("aaaa", "ready", Some("alice"));
        let err = check_owner(tmp.path(), &t).unwrap_err();
        assert!(err.to_string().contains("alice"), "error should mention the owner");
    }

    #[test]
    fn check_owner_fails_when_identity_is_unassigned() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("apm.toml"), "[project]\nname = \"test\"\n").unwrap();
        let t = make_ticket_with_owner_field("aaaa", "ready", Some("alice"));
        let err = check_owner(tmp.path(), &t).unwrap_err();
        assert!(err.to_string().contains("identity not configured"));
    }

    #[test]
    fn check_owner_passes_when_ticket_has_no_owner() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("apm.toml"), "[project]\nname = \"test\"\n").unwrap();
        let t = make_ticket_with_owner_field("aaaa", "ready", None);
        assert!(check_owner(tmp.path(), &t).is_ok());
    }

    #[test]
    fn check_owner_rejects_owner_change_on_terminal_state() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg_toml = concat!(
            "[project]\nname = \"test\"\n\n",
            "[[workflow.states]]\nid = \"open\"\nlabel = \"Open\"\nterminal = false\n\n",
            "[[workflow.states]]\nid = \"closed\"\nlabel = \"Closed\"\nterminal = true\n",
        );
        std::fs::write(tmp.path().join("apm.toml"), cfg_toml).unwrap();
        let t = make_ticket_with_owner_field("aaaa", "closed", Some("alice"));
        let err = check_owner(tmp.path(), &t).unwrap_err();
        assert!(
            err.to_string().contains("cannot change owner of a closed ticket"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn check_owner_allows_owner_change_on_non_terminal_state() {
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        let cfg_toml = concat!(
            "[project]\nname = \"test\"\n\n",
            "[[workflow.states]]\nid = \"open\"\nlabel = \"Open\"\nterminal = false\n\n",
            "[[workflow.states]]\nid = \"closed\"\nlabel = \"Closed\"\nterminal = true\n",
        );
        std::fs::write(tmp.path().join("apm.toml"), cfg_toml).unwrap();
        std::fs::write(apm_dir.join("local.toml"), "username = \"alice\"\n").unwrap();
        let t = make_ticket_with_owner_field("aaaa", "open", Some("alice"));
        assert!(check_owner(tmp.path(), &t).is_ok());
    }
}
