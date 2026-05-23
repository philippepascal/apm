use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::{Config, TransitionConfig, WorkerProfileConfig};

// ---------------------------------------------------------------------------
// Static fallback content
// ---------------------------------------------------------------------------

static STATIC_STATE_MACHINE: &str = "Standard APM workflow states and transitions:\n\
\n\
### new\n\
A ticket has been created but not yet groomed.\n\
Actionable by: supervisor, engineer\n\
  → groomed (trigger: manual, role: supervisor)\n\
  → closed (trigger: cancel, role: supervisor)\n\
\n\
### groomed\n\
Ticket is ready for spec writing.\n\
Actionable by: agent\n\
  → in_design (trigger: apm state <id> in_design, role: spec-writer)\n\
  → closed (trigger: cancel, role: supervisor)\n\
\n\
### in_design\n\
Spec is being actively written or revised.\n\
Actionable by: agent\n\
  → specd (trigger: apm state <id> specd, role: spec-writer)\n\
  → question (trigger: apm state <id> question, role: spec-writer)\n\
\n\
### specd\n\
Spec is complete; awaiting supervisor review.\n\
Actionable by: supervisor\n\
  → ready (trigger: approve, role: supervisor)\n\
  → ammend (trigger: request changes, role: supervisor)\n\
  → in_design (trigger: reject, role: supervisor)\n\
\n\
### ammend\n\
Spec requires revisions per supervisor requests.\n\
Actionable by: agent\n\
  → in_design (trigger: apm state <id> in_design, role: spec-writer)\n\
\n\
### ready\n\
Ticket is approved and queued for implementation.\n\
Actionable by: agent\n\
  → in_progress (trigger: apm start <id>, role: worker)\n\
\n\
### in_progress\n\
Implementation is in progress.\n\
Actionable by: agent\n\
  → implemented (trigger: apm state <id> implemented, role: worker)\n\
  → blocked (trigger: apm state <id> blocked, role: worker)\n\
\n\
### blocked\n\
Implementation is blocked on a supervisor decision.\n\
Actionable by: supervisor\n\
  → ready (trigger: unblock, role: supervisor)\n\
\n\
### implemented\n\
Implementation complete; awaiting supervisor review.\n\
Actionable by: supervisor\n\
  → closed (trigger: approve, role: supervisor)\n\
  → ready (trigger: reject, role: supervisor)\n\
\n\
### closed\n\
Terminal state. Ticket is done. No further transitions.\n";

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

static SHELL_DISCIPLINE_BODY: &str = "Keep each Bash call to a single operation.\n\
\n\
Do not chain commands:\n\
\n\
  # Wrong — && chains defeat allow-list matching\n\
  apm sync && apm list --state ready\n\
\n\
  # Right — one call per operation\n\
  apm sync\n\
  apm list --state ready\n\
\n\
Do not use $() subshells:\n\
\n\
  # Wrong — triggers permission prompt\n\
  apm spec 1234 --section Problem --set \"$(cat /tmp/problem.md)\"\n\
\n\
  # Right — write content with the Write tool, then reference by file\n\
  apm spec 1234 --section Problem --set-file /tmp/problem.md\n\
\n\
Do not use background jobs (&):\n\
\n\
  # Wrong — & defeats pattern matching\n\
  apm state 1234 implemented & apm state 5678 implemented & wait\n\
\n\
  # Right — sequential calls\n\
  apm state 1234 implemented\n\
  apm state 5678 implemented\n\
\n\
Use git -C for all git operations in worktrees:\n\
\n\
  # Wrong — cd && git triggers security check\n\
  cd \"$wt\" && git add .\n\
\n\
  # Right\n\
  git -C \"$wt\" add <files>\n\
\n\
Use bash -c for multi-step commands that must share a directory:\n\
\n\
  # Right — single bash call, matches Bash(bash *)\n\
  bash -c \"cd $wt && cargo test --workspace 2>&1\"\n\
\n\
Use the Write tool instead of heredocs or $() for temp files:\n\
  Write the file via the Write tool, then pass --set-file to apm spec.\n\
\n\
Off-limits — do not read or write these files:\n\
\n\
  .claude/              (settings, memory, CLAUDE.md)\n\
  .apm/                 (except the ticket file)\n\
  .gitignore, .github/  (project config)\n";

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
///   Scopes the state-machine and command-reference sections.
/// - `commands` — `(name, about)` pairs extracted from the CLI by the caller.
///   Keeps `apm-core` free of a clap dependency.
///
/// Returns a plain-text string with no ANSI escape codes.
pub fn generate(root: &Path, role: Option<&str>, commands: &[(String, String)]) -> Result<String> {
    let config = Config::load(root).ok();
    let mut out = String::new();

    // 1. State machine
    out.push_str("## State Machine\n\n");
    out.push_str(&state_machine_body(config.as_ref(), role));

    // 2. Ticket format
    out.push_str("## Ticket Format\n\n");
    out.push_str(&ticket_format_body(config.as_ref()));

    // 3. Shell discipline
    out.push_str("## Shell Discipline\n\n");
    out.push_str(SHELL_DISCIPLINE_BODY);
    out.push('\n');

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

    // Build filter set when a role is requested.
    let filter: Option<HashSet<String>> = if let Some(role_name) = role {
        let mut source_states: HashSet<String> = HashSet::new();
        let mut target_states: HashSet<String> = HashSet::new();

        for state in &config.workflow.states {
            for transition in &state.transitions {
                let t_role = derive_transition_role(transition, &config.worker_profiles);
                if t_role == role_name {
                    source_states.insert(state.id.clone());
                    target_states.insert(transition.to.clone());
                }
            }
        }

        if source_states.is_empty() && target_states.is_empty() {
            None
        } else {
            let mut combined = source_states;
            combined.extend(target_states);
            Some(combined)
        }
    } else {
        None
    };

    for state in &config.workflow.states {
        if let Some(ref filter_set) = filter {
            if !filter_set.contains(&state.id) {
                continue;
            }
        }

        // State heading
        out.push_str(&format!("### {} ({})\n", state.label, state.id));

        if !state.description.is_empty() {
            out.push_str(&state.description);
            out.push('\n');
        }
        if !state.actionable.is_empty() {
            out.push_str(&format!("Actionable by: {}\n", state.actionable.join(", ")));
        }
        if state.terminal {
            out.push_str("Terminal state\n");
        }

        // Transitions — emit all when unscoped, or only role-matching ones.
        for transition in &state.transitions {
            if filter.is_some() {
                let t_role = derive_transition_role(transition, &config.worker_profiles);
                if t_role != role.unwrap_or("") {
                    continue;
                }
            }
            let t_role = derive_transition_role(transition, &config.worker_profiles);
            let mut line = format!("  → {}", transition.to);
            if !transition.trigger.is_empty() {
                line.push_str(&format!(", trigger: {}", transition.trigger));
            }
            line.push_str(&format!(", role: {}", t_role));
            out.push_str(&line);
            out.push('\n');
        }

        out.push('\n');
    }

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

fn derive_transition_role(
    t: &TransitionConfig,
    profiles: &HashMap<String, WorkerProfileConfig>,
) -> String {
    // (a) profile.role if a matching profile is found
    if let Some(ref profile_name) = t.profile {
        if let Some(profile) = profiles.get(profile_name) {
            if let Some(ref role) = profile.role {
                return role.clone();
            }
        }
    }
    // (b) basename of instructions path, strip "apm." prefix and ".md" suffix
    if let Some(ref instructions) = t.instructions {
        let path = Path::new(instructions);
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            let without_prefix = file_name.strip_prefix("apm.").unwrap_or(file_name);
            let role = without_prefix.strip_suffix(".md").unwrap_or(without_prefix);
            if !role.is_empty() {
                return role.to_string();
            }
        }
    }
    // (c) default
    "worker".to_string()
}

fn role_command_allowlist(role: &str) -> Option<&'static [&'static str]> {
    match role {
        "spec-writer" => Some(&["show", "spec", "set", "state", "new", "sync", "list", "next"]),
        "worker" => Some(&["show", "start", "state", "spec", "new", "sync", "list", "next"]),
        _ => None,
    }
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
        ]
    }

    #[test]
    fn generate_no_role_contains_all_sections() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), None, &empty_commands()).unwrap();
        assert!(out.contains("## State Machine"), "State Machine header missing");
        assert!(out.contains("## Ticket Format"), "Ticket Format header missing");
        assert!(out.contains("## Shell Discipline"), "Shell Discipline header missing");
        assert!(out.contains("## Session Identity"), "Session Identity header missing");
        // Command Reference is omitted when no commands are passed
        assert!(!out.contains("## Command Reference"), "Command Reference header should be absent with empty commands");
    }

    #[test]
    fn generate_no_role_sections_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        // Use sample_commands so Command Reference is present for ordering check
        let out = generate(tmp.path(), None, &sample_commands()).unwrap();
        let pos_sm = out.find("## State Machine").unwrap();
        let pos_tf = out.find("## Ticket Format").unwrap();
        let pos_sd = out.find("## Shell Discipline").unwrap();
        let pos_si = out.find("## Session Identity").unwrap();
        let pos_cr = out.find("## Command Reference").unwrap();
        assert!(pos_sm < pos_tf, "State Machine must precede Ticket Format");
        assert!(pos_tf < pos_sd, "Ticket Format must precede Shell Discipline");
        assert!(pos_sd < pos_si, "Shell Discipline must precede Session Identity");
        assert!(pos_si < pos_cr, "Session Identity must precede Command Reference");
    }

    #[test]
    fn generate_no_ansi() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), None, &sample_commands()).unwrap();
        assert!(!out.contains('\x1b'), "ANSI escape code found in output");
    }

    #[test]
    fn generate_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let commands = sample_commands();
        let out1 = generate(tmp.path(), Some("worker"), &commands).unwrap();
        let out2 = generate(tmp.path(), Some("worker"), &commands).unwrap();
        assert_eq!(out1, out2, "generate is not idempotent");
    }

    #[test]
    fn generate_role_independent_sections() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), Some("worker"), &sample_commands()).unwrap();
        assert!(out.contains("## Shell Discipline"), "Shell Discipline missing with role");
        assert!(out.contains("## Session Identity"), "Session Identity missing with role");
        // Both sections must contain substantive content
        assert!(out.contains("git -C"), "git -C discipline missing");
        assert!(out.contains("APM_AGENT_NAME"), "APM_AGENT_NAME identity missing");
    }

    #[test]
    fn generate_worker_scopes_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), Some("worker"), &sample_commands()).unwrap();

        // worker allowlist includes "start"
        assert!(out.contains("apm start"), "'apm start' not found for worker role");

        // worker allowlist excludes "prompt" (not in worker list)
        // find the Command Reference section and assert "apm prompt" absent there
        let cr_pos = out.find("## Command Reference").unwrap();
        let cr_section = &out[cr_pos..];
        assert!(
            !cr_section.contains("apm prompt"),
            "'apm prompt' found in worker command reference but should be excluded"
        );

        // Static fallback includes in_progress (worker acts there)
        assert!(
            out.contains("in_progress"),
            "in_progress state missing from worker output"
        );
    }

    #[test]
    fn generate_spec_writer_scopes_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), Some("spec-writer"), &sample_commands()).unwrap();

        // spec-writer allowlist includes "spec" and "set"
        let cr_pos = out.find("## Command Reference").unwrap();
        let cr_section = &out[cr_pos..];
        assert!(cr_section.contains("apm spec"), "'apm spec' missing for spec-writer");
        assert!(cr_section.contains("apm set"), "'apm set' missing for spec-writer");

        // spec-writer allowlist excludes "start"
        assert!(
            !cr_section.contains("apm start"),
            "'apm start' found in spec-writer command reference but should be excluded"
        );
    }

    #[test]
    fn generate_unknown_role_falls_back_to_full_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let out = generate(tmp.path(), Some("unknown-role-xyz"), &sample_commands()).unwrap();

        // All commands should be present since unknown role falls back to unscoped
        let cr_pos = out.find("## Command Reference").unwrap();
        let cr_section = &out[cr_pos..];
        assert!(cr_section.contains("apm start"), "start missing for unknown role");
        assert!(cr_section.contains("apm prompt"), "prompt missing for unknown role");
    }

    #[test]
    fn derive_transition_role_from_instructions_path() {
        let profiles = HashMap::new();
        let t = crate::config::TransitionConfig {
            to: "specd".to_string(),
            trigger: "submit".to_string(),
            label: String::new(),
            hint: String::new(),
            completion: crate::config::CompletionStrategy::None,
            focus_section: None,
            context_section: None,
            warning: None,
            profile: None,
            instructions: Some(".apm/agents/default/apm.spec-writer.md".to_string()),
            role_prefix: None,
            agent: None,
            on_failure: None,
            outcome: None,
        };
        assert_eq!(derive_transition_role(&t, &profiles), "spec-writer");
    }

    #[test]
    fn derive_transition_role_defaults_to_worker() {
        let profiles = HashMap::new();
        let t = crate::config::TransitionConfig {
            to: "implemented".to_string(),
            trigger: String::new(),
            label: String::new(),
            hint: String::new(),
            completion: crate::config::CompletionStrategy::None,
            focus_section: None,
            context_section: None,
            warning: None,
            profile: None,
            instructions: None,
            role_prefix: None,
            agent: None,
            on_failure: None,
            outcome: None,
        };
        assert_eq!(derive_transition_role(&t, &profiles), "worker");
    }

    #[test]
    fn derive_transition_role_from_profile() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "my_profile".to_string(),
            WorkerProfileConfig {
                role: Some("spec-writer".to_string()),
                ..Default::default()
            },
        );
        let t = crate::config::TransitionConfig {
            to: "specd".to_string(),
            trigger: String::new(),
            label: String::new(),
            hint: String::new(),
            completion: crate::config::CompletionStrategy::None,
            focus_section: None,
            context_section: None,
            warning: None,
            profile: Some("my_profile".to_string()),
            instructions: None,
            role_prefix: None,
            agent: None,
            on_failure: None,
            outcome: None,
        };
        assert_eq!(derive_transition_role(&t, &profiles), "spec-writer");
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
actionable = ["agent"]

[[workflow.states.transitions]]
to = "in_progress"
trigger = "start"
instructions = ".apm/agents/default/apm.worker.md"

[[workflow.states]]
id = "in_progress"
label = "In Progress"
actionable = ["agent"]

[[workflow.states.transitions]]
to = "implemented"
trigger = "done"
instructions = ".apm/agents/default/apm.worker.md"

[[workflow.states]]
id = "implemented"
label = "Implemented"
actionable = ["supervisor"]

[[workflow.states.transitions]]
to = "closed"
trigger = "approve"

[[workflow.states]]
id = "groomed"
label = "Groomed"
actionable = ["agent"]

[[workflow.states.transitions]]
to = "in_design"
trigger = "claim"
instructions = ".apm/agents/default/apm.spec-writer.md"

[[workflow.states]]
id = "in_design"
label = "In Design"
actionable = ["agent"]

[[workflow.states.transitions]]
to = "specd"
trigger = "submit"
instructions = ".apm/agents/default/apm.spec-writer.md"

[[workflow.states]]
id = "specd"
label = "Specd"
actionable = ["supervisor"]

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

        // Worker role: should include ready, in_progress, implemented but not groomed/specd/in_design
        let out = generate(tmp.path(), Some("worker"), &commands).unwrap();
        let sm = state_machine_section(&out);
        assert!(sm.contains("in_progress"), "in_progress missing for worker");
        assert!(sm.contains("ready"), "ready (source of worker transition) missing");
        assert!(sm.contains("implemented"), "implemented (target of worker transition) missing");
        assert!(!sm.contains("groomed"), "groomed should not appear for worker role");
        assert!(!sm.contains("in_design"), "in_design should not appear for worker role");
        assert!(!sm.contains("specd"), "specd should not appear for worker role");

        // spec-writer role: should include groomed, in_design, specd but not ready/in_progress
        let out = generate(tmp.path(), Some("spec-writer"), &commands).unwrap();
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

        let out = generate(tmp.path(), None, &[]).unwrap();
        assert!(out.contains("Problem"), "Problem section missing");
        assert!(out.contains("Acceptance criteria"), "Acceptance criteria missing");
        assert!(out.contains("Open questions"), "Open questions missing");
        assert!(out.contains("required"), "required flag missing");
    }
}
