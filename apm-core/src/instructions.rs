use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

use crate::config::Config;

// ---------------------------------------------------------------------------
// Static fallback content
// ---------------------------------------------------------------------------

static STATIC_STATE_MACHINE: &str = "| From | To | Command |\n\
|------|----|----------|\n\
| new | groomed | apm state <id> groomed |\n\
| new | closed | apm state <id> closed |\n\
| groomed | in_design | apm state <id> in_design |\n\
| groomed | closed | apm state <id> closed |\n\
| in_design | specd | apm state <id> specd |\n\
| in_design | question | apm state <id> question |\n\
| specd | ready | apm state <id> ready |\n\
| specd | ammend | apm state <id> ammend |\n\
| specd | in_design | apm state <id> in_design |\n\
| ammend | in_design | apm start <id> |\n\
| ready | in_progress | apm start <id> |\n\
| in_progress | implemented | apm state <id> implemented |\n\
| in_progress | blocked | apm state <id> blocked |\n\
| blocked | ready | apm state <id> ready |\n\
| implemented | closed | apm state <id> closed |\n\
| implemented | ready | apm state <id> ready |\n";

static STATIC_TICKET_FORMAT: &str = "Standard frontmatter fields (TOML between +++ delimiters):\n\
\n\
Required fields:\n\
  id          — unique 8-char hex identifier\n\
  title       — short human-readable summary\n\
  state       — current workflow state (e.g. new, ready, in_progress)\n\
  priority    — integer; higher = picked first by apm next\n\
  effort      — integer 1-10; implementation scale estimate\n\
  risk        — integer 1-10; technical risk estimate\n\
  author      — username who created the ticket\n\
  owner       — username responsible for the ticket\n\
  branch      — git branch name (ticket/<id>-<slug>)\n\
  created_at  — ISO 8601 timestamp\n\
  updated_at  — ISO 8601 timestamp\n\
\n\
Optional fields:\n\
  epic          — parent epic ID\n\
  target_branch — integration target (defaults to project default branch)\n\
  depends_on    — comma-separated list of blocking ticket IDs\n\
\n\
Body sections (under ## Spec):\n\
\n\
  ### Problem (free, required)\n\
    What is broken or missing, and why it matters.\n\
\n\
  ### Acceptance criteria (tasks, required)\n\
    Checkbox list; each item independently testable.\n\
\n\
  ### Out of scope (free, required)\n\
    Explicit list of what this ticket does not cover.\n\
\n\
  ### Approach (free, required)\n\
    How the implementation will work.\n\
\n\
  ### Open questions (qa)\n\
    Blocking questions for the supervisor.\n\
\n\
  ### Amendment requests (tasks)\n\
    Supervisor-requested changes to the spec.\n\
\n\
  ## History (auto-managed)\n\
    Transition log written by apm. Never edit manually.\n\
\n\
Ticket file rules:\n\
  - Do not hand-edit the History section — apm state appends rows automatically.\n\
  - Do not rename the ticket file. The filename (tickets/<id>-<slug>.md) is derived\n\
    from the branch name and is load-bearing for all apm lookups.\n\
  - Find the exact filename with: ls tickets/<id>-*.md\n";

static SESSION_IDENTITY_BODY: &str = "Generate a unique session name at the start of every session.\n\
Use a fixed string — do not use $() substitution inline, as it triggers\n\
permission prompts. Pick a name of the form claude-MMDD-HHMM-XXXX\n\
(e.g. claude-0325-1430-a3f9) and export it before running any apm command:\n\
\n\
  export APM_AGENT_NAME=claude-0325-1430-a3f9\n\
\n\
Hold the same name for the entire session. Do not regenerate mid-session.\n\
Engineers set APM_AGENT_NAME to their own username when working directly.\n";

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Generate full APM system-knowledge text.
///
/// - `root` — project root used to load `Config` (workflow + ticket config).
///   Falls back to static built-in descriptions when config is absent.
/// - `role` — optional role name (e.g. `"worker"`, `"spec-writer"`).
///   When absent, returns a role index listing available roles instead of
///   the full system-knowledge sections.
/// - `ticket_id` — optional ticket id. When present, every occurrence of the
///   literal placeholder `<id>` in the rendered output is substituted.
/// - `commands` — `(name, about)` pairs extracted from the CLI by the caller.
///   Keeps `apm-core` free of a clap dependency.
///
/// Returns a plain-text string with no ANSI escape codes.
pub fn generate(root: &Path, role: Option<&str>, ticket_id: Option<&str>, commands: &[(String, String)], current_state: Option<&str>) -> Result<String> {
    // No-role: return role index immediately (no state machine, no sections).
    if role.is_none() {
        return Ok(role_index_body(root));
    }

    let config = Config::load(root).ok();
    let mut out = String::new();

    // 1. State machine
    out.push_str("## State Machine\n\n");
    out.push_str(&state_machine_body(config.as_ref(), role));

    // 2. Exit scenarios (only when ticket id is supplied)
    if ticket_id.is_some() {
        let body = exit_scenarios_body(config.as_ref(), current_state);
        if !body.is_empty() {
            out.push_str("## Exit scenarios\n\n");
            out.push_str("Choose the matching scenario and run the commands. Replace any <placeholder> text with your own.\n\n");
            out.push_str(&body);
        }
    }

    // 3. Ticket format
    out.push_str("## Ticket Format\n\n");
    out.push_str(&ticket_format_body(config.as_ref()));

    // 4. Session identity
    out.push_str("## Session Identity\n\n");
    out.push_str(SESSION_IDENTITY_BODY);
    out.push('\n');

    // 5. Command reference — omit section entirely when no commands are provided
    let cr = command_reference_body(role, commands);
    if !cr.is_empty() {
        out.push_str("## Command Reference\n\n");
        out.push_str(&cr);
    }

    // Ticket-id substitution: replace every <id> placeholder with the actual id.
    if let Some(id) = ticket_id {
        out = out.replace("<id>", id);
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Section builders
// ---------------------------------------------------------------------------

fn state_machine_body(config: Option<&Config>, role: Option<&str>) -> String {
    if let Some(cfg) = config {
        if !cfg.workflow.states.is_empty() {
            return format_live_state_machine(cfg, role);
        }
    }
    STATIC_STATE_MACHINE.to_string()
}

fn format_live_state_machine(config: &Config, role: Option<&str>) -> String {
    let mut out = String::new();
    out.push_str("| From | To | Command |\n");
    out.push_str("|------|----|----------|\n");

    for state in &config.workflow.states {
        let state_role: Option<&str> = state.worker_profile.as_deref()
            .and_then(|wp| wp.split_once('/').map(|(_, r)| r));

        for transition in &state.transitions {
            if let Some(role_name) = role {
                if state_role != Some(role_name) {
                    continue;
                }
            }
            let command = if transition.trigger == "command:start" {
                "apm start <id>".to_string()
            } else {
                format!("apm state <id> {}", transition.to)
            };
            out.push_str(&format!("| {} | {} | {} |\n", state.id, transition.to, command));
        }
    }
    out.push('\n');
    out
}

fn ticket_format_body(config: Option<&Config>) -> String {
    if let Some(cfg) = config {
        if !cfg.ticket.sections.is_empty() {
            return format_live_ticket_format(cfg);
        }
    }
    STATIC_TICKET_FORMAT.to_string()
}

fn format_live_ticket_format(config: &Config) -> String {
    let mut out = String::new();

    out.push_str("Standard frontmatter fields (TOML between +++ delimiters):\n\n");
    out.push_str("Required fields:\n");
    out.push_str("  id, title, state, priority, effort, risk, author, owner, branch,\n");
    out.push_str("  created_at, updated_at\n\n");
    out.push_str("Optional fields:\n");
    out.push_str("  epic, target_branch, depends_on\n\n");
    out.push_str("Body sections (under ## Spec):\n\n");

    for section in &config.ticket.sections {
        use crate::config::SectionType;
        let type_label = match section.type_ {
            SectionType::Free => "free",
            SectionType::Tasks => "tasks",
            SectionType::Qa => "qa",
        };
        let req_label = if section.required { ", required" } else { "" };
        out.push_str(&format!(
            "  ### {} ({}{})  \n",
            section.name, type_label, req_label
        ));
        if let Some(ref placeholder) = section.placeholder {
            out.push_str(&format!("    {}\n", placeholder));
        }
    }

    out.push_str("\n  ## History (auto-managed)\n");
    out.push_str("    Transition log written by apm. Never edit manually.\n");
    out.push_str("\nTicket file rules:\n");
    out.push_str("  - Do not hand-edit the History section — apm state appends rows automatically.\n");
    out.push_str("  - Do not rename the ticket file. The filename (tickets/<id>-<slug>.md) is derived\n");
    out.push_str("    from the branch name and is load-bearing for all apm lookups.\n");
    out.push_str("  - Find the exact filename with: ls tickets/<id>-*.md\n");
    out
}

fn role_index_body(root: &Path) -> String {
    let mut out = String::from("## Available Roles\n\n");

    let hardcoded: &[(&str, &str)] = &[
        ("coder", "Implements tickets in a git worktree"),
        ("spec-writer", "Writes and revises ticket specs"),
        ("main-agent", "Project management companion for the supervisor"),
    ];

    let hardcoded_names: HashSet<&str> = hardcoded.iter().map(|(n, _)| *n).collect();
    let mut extra_roles: Vec<String> = Vec::new();

    let agents_dir = root.join(".apm/agents");
    if agents_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&agents_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let agent_dir = entry.path();
                if !agent_dir.is_dir() {
                    continue;
                }
                if let Ok(files) = std::fs::read_dir(&agent_dir) {
                    for file in files.filter_map(|e| e.ok()) {
                        if let Ok(name) = file.file_name().into_string() {
                            if let Some(rest) = name.strip_prefix("apm.") {
                                if let Some(role) = rest.strip_suffix(".md") {
                                    if !hardcoded_names.contains(role)
                                        && !extra_roles.iter().any(|r| r == role)
                                    {
                                        extra_roles.push(role.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    extra_roles.sort();

    for (name, desc) in hardcoded {
        out.push_str(&format!("  {:<16}{}\n", name, desc));
    }
    for role in &extra_roles {
        out.push_str(&format!("  {:<16}(custom role)\n", role));
    }
    out.push('\n');
    out
}

fn command_reference_body(role: Option<&str>, commands: &[(String, String)]) -> String {
    let allowlist = role.and_then(role_command_allowlist);

    let filtered: Vec<&(String, String)> = if let Some(allow) = allowlist {
        commands
            .iter()
            .filter(|(name, _)| allow.contains(&name.as_str()))
            .collect()
    } else {
        commands.iter().collect()
    };

    if filtered.is_empty() {
        return String::new();
    }

    let max_name = filtered.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    let col_width = 4 + max_name; // "apm " prefix

    let mut out = String::new();
    for (name, about) in &filtered {
        let label = format!("apm {}", name);
        out.push_str(&format!("  {:<col_width$}  {}\n", label, about));
    }
    out.push('\n');
    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const WORKER_COMMAND_ALLOWLIST: &[&str] = &["show", "state", "spec", "set", "new", "instructions"];

/// Name + description tuples for the six worker-permitted `apm` commands.
/// Names must stay in sync with WORKER_COMMAND_ALLOWLIST (ticket 9c66e199).
/// Descriptions are purpose-built for agent consumption; they are NOT copied
/// from clap `///` doc comments. If a subcommand's fundamental purpose changes,
/// update both this const and the clap string in apm/src/main.rs in the same commit.
pub(crate) const WORKER_COMMANDS: &[(&str, &str)] = &[
    ("instructions", "Output APM system knowledge for agents: state machine, ticket format, session identity, and command reference"),
    ("new",          "Create a new ticket"),
    ("set",          "Set a field on a ticket"),
    ("show",         "Show a ticket"),
    ("spec",         "Read or write individual spec sections of a ticket"),
    ("state",        "Transition a ticket's state"),
];

fn role_command_allowlist(_role: &str) -> Option<&'static [&'static str]> {
    Some(WORKER_COMMAND_ALLOWLIST)
}

fn exit_scenarios_body(config: Option<&Config>, current_state: Option<&str>) -> String {
    let (config, current_state) = match (config, current_state) {
        (Some(c), Some(s)) => (c, s),
        _ => return String::new(),
    };
    let state_cfg = match config.workflow.states.iter().find(|s| s.id == current_state) {
        Some(s) if s.worker_profile.is_some() => s,
        _ => return String::new(),
    };
    let hinted: Vec<_> = state_cfg.transitions.iter()
        .filter(|t| t.worker_hint.is_some())
        .collect();
    if hinted.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for t in hinted {
        let hint = t.worker_hint.as_ref().unwrap();
        out.push_str(&format!("### {}\n\n", hint));
        if let Some(pre) = &t.worker_pre {
            out.push_str(pre);
            out.push('\n');
        }
        out.push_str(&format!("apm state <id> {}\n\n", t.to));
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_commands() -> Vec<(String, String)> {
        vec![]
    }

    fn sample_commands() -> Vec<(String, String)> {
        vec![
            ("show".to_string(), "Show a ticket".to_string()),
            ("start".to_string(), "Claim a ticket".to_string()),
            ("state".to_string(), "Transition state".to_string()),
            ("spec".to_string(), "Read or write spec sections".to_string()),
            ("new".to_string(), "Create a new ticket".to_string()),
            ("sync".to_string(), "Sync with remote".to_string()),
            ("list".to_string(), "List tickets".to_string()),
            ("next".to_string(), "Return next actionable ticket".to_string()),
            ("set".to_string(), "Set a field on a ticket".to_string()),
            ("prompt".to_string(), "Print system prompt".to_string()),
            ("instructions".to_string(), "Print APM system knowledge".to_string()),
        ]
    }

    #[test]
    fn generate_no_role_lists_roles() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), None, None, &empty_commands(), None).unwrap();
        assert!(out.contains("coder"), "coder missing from role index");
        assert!(out.contains("spec-writer"), "spec-writer missing from role index");
        assert!(out.contains("main-agent"), "main-agent missing from role index");
        assert!(!out.contains("## State Machine"), "State Machine should be absent with no role");
    }

    #[test]
    fn generate_role_table_precedes_command_reference() {
        let tmp = tempfile::tempdir().unwrap();
        // Use sample_commands so Command Reference is present for ordering check
        let out = generate(tmp.path(), Some("worker"), None, &sample_commands(), None).unwrap();
        let pos_sm = out.find("## State Machine").unwrap();
        let pos_cr = out.find("## Command Reference").unwrap();
        assert!(pos_sm < pos_cr, "State Machine must precede Command Reference");
    }

    #[test]
    fn generate_no_ansi() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), None, None, &sample_commands(), None).unwrap();
        assert!(!out.contains('\x1b'), "ANSI escape code found in output");
    }

    #[test]
    fn generate_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let commands = sample_commands();
        let out1 = generate(tmp.path(), Some("worker"), None, &commands, None).unwrap();
        let out2 = generate(tmp.path(), Some("worker"), None, &commands, None).unwrap();
        assert_eq!(out1, out2, "generate is not idempotent");
    }

    #[test]
    fn generate_role_independent_sections() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), Some("worker"), None, &sample_commands(), None).unwrap();
        assert!(out.contains("## Session Identity"), "Session Identity missing with role");
        assert!(out.contains("APM_AGENT_NAME"), "APM_AGENT_NAME identity missing");
        // State machine must use table format
        assert!(out.contains("| From | To | Command |"), "table header missing");
    }

    #[test]
    fn shell_discipline_absent_from_instructions() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), None, None, &empty_commands(), None).unwrap();
        assert!(!out.contains("## Shell Discipline"), "Shell Discipline must not appear in apm instructions");
        assert!(!out.contains("Do not batch tool calls in parallel"), "parallel batching rule must not appear in apm instructions");
    }

    #[test]
    fn generate_worker_scopes_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), Some("worker"), None, &sample_commands(), None).unwrap();

        let cr_pos = out.find("## Command Reference").unwrap();
        let cr_section = &out[cr_pos..];

        // Six unified commands are present
        assert!(cr_section.contains("apm show"), "'apm show' missing for worker role");
        assert!(cr_section.contains("apm state"), "'apm state' missing for worker role");
        assert!(cr_section.contains("apm spec"), "'apm spec' missing for worker role");
        assert!(cr_section.contains("apm set"), "'apm set' missing for worker role");
        assert!(cr_section.contains("apm new"), "'apm new' missing for worker role");
        assert!(cr_section.contains("apm instructions"), "'apm instructions' missing for worker role");

        // Supervisor commands excluded
        assert!(!cr_section.contains("apm start"), "'apm start' found in worker command reference but should be excluded");
        assert!(!cr_section.contains("apm sync"), "'apm sync' found in worker command reference but should be excluded");
        assert!(!cr_section.contains("apm prompt"), "'apm prompt' found in worker command reference but should be excluded");
    }

    #[test]
    fn generate_spec_writer_scopes_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), Some("spec-writer"), None, &sample_commands(), None).unwrap();

        let cr_pos = out.find("## Command Reference").unwrap();
        let cr_section = &out[cr_pos..];

        // Six unified commands are present
        assert!(cr_section.contains("apm show"), "'apm show' missing for spec-writer");
        assert!(cr_section.contains("apm state"), "'apm state' missing for spec-writer");
        assert!(cr_section.contains("apm spec"), "'apm spec' missing for spec-writer");
        assert!(cr_section.contains("apm set"), "'apm set' missing for spec-writer");
        assert!(cr_section.contains("apm new"), "'apm new' missing for spec-writer");
        assert!(cr_section.contains("apm instructions"), "'apm instructions' missing for spec-writer");

        // Supervisor commands excluded
        assert!(!cr_section.contains("apm start"), "'apm start' found in spec-writer command reference but should be excluded");
    }

    #[test]
    fn generate_unknown_role_gets_worker_allowlist() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), Some("unknown-role-xyz"), None, &sample_commands(), None).unwrap();

        let cr_pos = out.find("## Command Reference").unwrap();
        let cr_section = &out[cr_pos..];

        // Six unified commands are present for any role
        assert!(cr_section.contains("apm show"), "'apm show' missing for unknown role");
        assert!(cr_section.contains("apm state"), "'apm state' missing for unknown role");
        assert!(cr_section.contains("apm spec"), "'apm spec' missing for unknown role");
        assert!(cr_section.contains("apm set"), "'apm set' missing for unknown role");
        assert!(cr_section.contains("apm new"), "'apm new' missing for unknown role");
        assert!(cr_section.contains("apm instructions"), "'apm instructions' missing for unknown role");

        // Supervisor commands excluded
        assert!(!cr_section.contains("apm prompt"), "'apm prompt' found for unknown role but should be excluded");
    }

    #[test]
    fn generate_with_id_no_placeholder_remains() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), Some("worker"), Some("abc12345"), &[], None).unwrap();
        assert!(!out.contains("<id>"), "no <id> placeholder should remain after substitution");
        assert!(out.contains("abc12345"), "ticket id should appear in output");
    }

    #[test]
    fn imperative_table_format_header() {
        let config_toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id = "ready"
label = "Ready"

[[workflow.states.transitions]]
to = "in_progress"
trigger = "command:start"

[[workflow.states]]
id = "in_progress"
label = "In Progress"
worker_profile = "claude/coder"

[[workflow.states.transitions]]
to = "implemented"
trigger = "done"
"#;
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("config.toml"), config_toml).unwrap();

        let out = generate(tmp.path(), Some("coder"), None, &[], None).unwrap();
        // State machine section must use table format
        let sm_pos = out.find("## State Machine").unwrap();
        let sm_section = &out[sm_pos..];
        assert!(
            sm_section.contains("| From | To | Command |"),
            "table header missing from state machine section; got:\n{sm_section}"
        );
    }

    #[test]
    fn live_state_machine_filters_by_role() {

        let config_toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id = "ready"
label = "Ready"
worker_profile = "claude/coder"

[[workflow.states.transitions]]
to = "in_progress"
trigger = "start"

[[workflow.states]]
id = "in_progress"
label = "In Progress"
worker_profile = "claude/coder"

[[workflow.states.transitions]]
to = "implemented"
trigger = "done"

[[workflow.states]]
id = "implemented"
label = "Implemented"

[[workflow.states.transitions]]
to = "closed"
trigger = "approve"

[[workflow.states]]
id = "groomed"
label = "Groomed"
worker_profile = "claude/spec-writer"

[[workflow.states.transitions]]
to = "in_design"
trigger = "claim"

[[workflow.states]]
id = "in_design"
label = "In Design"
worker_profile = "claude/spec-writer"

[[workflow.states.transitions]]
to = "specd"
trigger = "submit"

[[workflow.states]]
id = "specd"
label = "Specd"

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#;
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("config.toml"), config_toml).unwrap();

        let commands: Vec<(String, String)> = vec![];

        // Helper: extract just the state machine section (between ## State Machine and ## Ticket Format)
        fn state_machine_section(out: &str) -> &str {
            let start = out.find("## State Machine\n").unwrap();
            let end = out.find("## Ticket Format\n").unwrap();
            &out[start..end]
        }

        // Coder role: should include ready, in_progress, implemented but not groomed/specd/in_design
        let out = generate(tmp.path(), Some("coder"), None, &commands, None).unwrap();
        let sm = state_machine_section(&out);
        assert!(sm.contains("in_progress"), "in_progress missing for coder");
        assert!(sm.contains("ready"), "ready (source of coder transition) missing");
        assert!(sm.contains("implemented"), "implemented (target of coder transition) missing");
        assert!(!sm.contains("groomed"), "groomed should not appear for coder role");
        assert!(!sm.contains("in_design"), "in_design should not appear for coder role");
        assert!(!sm.contains("specd"), "specd should not appear for coder role");

        // spec-writer role: should include groomed, in_design, specd but not ready/in_progress
        let out = generate(tmp.path(), Some("spec-writer"), None, &commands, None).unwrap();
        let sm = state_machine_section(&out);
        assert!(sm.contains("groomed"), "groomed missing for spec-writer");
        assert!(sm.contains("in_design"), "in_design missing for spec-writer");
        assert!(sm.contains("specd"), "specd (target) missing for spec-writer");
        assert!(!sm.contains("ready"), "ready should not appear in state machine for spec-writer role");
        assert!(!sm.contains("in_progress"), "in_progress should not appear in state machine for spec-writer role");
    }

    #[test]
    fn live_ticket_format_from_config() {
        let config_toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[ticket.sections]]
name = "Problem"
type = "free"
required = true
placeholder = "What is broken?"

[[ticket.sections]]
name = "Acceptance criteria"
type = "tasks"
required = true

[[ticket.sections]]
name = "Open questions"
type = "qa"
"#;
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("config.toml"), config_toml).unwrap();

        // Use role = Some("worker") — no-role now returns role index, not ticket format.
        let out = generate(tmp.path(), Some("worker"), None, &[], None).unwrap();
        assert!(out.contains("Problem"), "Problem section missing");
        assert!(out.contains("Acceptance criteria"), "Acceptance criteria missing");
        assert!(out.contains("Open questions"), "Open questions missing");
        assert!(out.contains("required"), "required flag missing");
    }

    fn exit_section(out: &str) -> &str {
        let start = out.find("## Exit scenarios\n").expect("## Exit scenarios missing");
        let end = out[start..].find("\n## ").map(|i| start + i + 1).unwrap_or(out.len());
        &out[start..end]
    }

    #[test]
    fn exit_scenarios_only_hinted_transitions() {
        let config_toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

  [[workflow.states.transitions]]
  to          = "implemented"
  trigger     = "manual"
  worker_hint = "If you completed the implementation and tests pass"

  [[workflow.states.transitions]]
  to      = "blocked"
  trigger = "manual"

[[workflow.states]]
id    = "specd"
label = "Specd"

  [[workflow.states.transitions]]
  to          = "closed"
  trigger     = "manual"
  worker_hint = "Supervisor only: close the spec"
"#;
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("config.toml"), config_toml).unwrap();

        // with worker-profile state and ticket id: exit scenarios present
        let out = generate(tmp.path(), Some("coder"), Some("abc12345"), &[], Some("in_progress")).unwrap();
        assert!(out.contains("## Exit scenarios"), "exit scenarios section must appear");
        let es = exit_section(&out);
        assert_eq!(es.matches("### If you completed").count(), 1, "exactly one hinted scenario");
        // un-hinted "blocked" transition has no worker_hint so no heading for it in exit scenarios
        assert!(!es.contains("### ") || es.matches("### ").count() == 1, "only the hinted scenario must have a heading");
        assert!(!es.contains("apm state abc12345 blocked"), "un-hinted blocked transition must be absent from exit scenarios");
        assert!(!out.contains("Supervisor only"), "supervisor-state scenarios must be absent (no worker_profile on specd)");

        // no ticket id: no exit scenarios
        let out_no_id = generate(tmp.path(), Some("coder"), None, &[], Some("in_progress")).unwrap();
        assert!(!out_no_id.contains("## Exit scenarios"), "exit scenarios must not appear without ticket id");

        // no current state: no exit scenarios
        let out_no_state = generate(tmp.path(), Some("coder"), Some("abc12345"), &[], None).unwrap();
        assert!(!out_no_state.contains("## Exit scenarios"), "exit scenarios must not appear without current_state");
    }

    #[test]
    fn exit_scenarios_worker_pre_before_apm_state() {
        let config_toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
worker_profile = "claude/coder"

  [[workflow.states.transitions]]
  to          = "blocked"
  trigger     = "manual"
  worker_hint = "If you lack information to proceed"
  worker_pre  = "apm spec <id> --section 'Open questions' --append '<your question text>'"
"#;
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("config.toml"), config_toml).unwrap();

        let out = generate(tmp.path(), Some("coder"), Some("abc12345"), &[], Some("in_progress")).unwrap();
        let es = exit_section(&out);
        let pre_pos = es.find("apm spec abc12345 --section 'Open questions'").expect("worker_pre must appear in exit scenarios");
        let state_pos = es.find("apm state abc12345 blocked").expect("apm state line must appear in exit scenarios");
        assert!(pre_pos < state_pos, "worker_pre must come before apm state");
    }

    #[test]
    fn exit_scenarios_default_workflow_in_progress() {
        let default_workflow = include_str!("default/workflow.toml");
        let tmp = tempfile::tempdir().unwrap();
        let apm_dir = tmp.path().join(".apm");
        std::fs::create_dir_all(&apm_dir).unwrap();
        std::fs::write(apm_dir.join("config.toml"), "[project]\nname = \"test\"\n[tickets]\ndir = \"tickets\"\n").unwrap();
        std::fs::write(apm_dir.join("workflow.toml"), default_workflow).unwrap();

        let out = generate(tmp.path(), Some("coder"), Some("abc12345"), &[], Some("in_progress")).unwrap();
        assert!(out.contains("## Exit scenarios"), "exit scenarios must appear for in_progress");
        let impl_pos = out.find("If you completed the implementation and tests pass").expect("implemented scenario missing");
        let blocked_pos = out.find("If you lack information to proceed").expect("blocked scenario missing");
        assert!(impl_pos < blocked_pos, "implemented scenario must precede blocked scenario");
    }
}
