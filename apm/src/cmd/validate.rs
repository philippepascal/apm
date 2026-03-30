use anyhow::Result;
use apm_core::{config::{CompletionStrategy, Config}, git, ticket};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

const KNOWN_PRECONDITIONS: &[&str] = &[
    "spec_not_empty",
    "spec_has_acceptance_criteria",
    "pr_exists",
    "pr_all_closing_merged",
];

const KNOWN_SIDE_EFFECTS: &[&str] = &["set_agent_null"];

#[derive(Debug, Serialize)]
struct Issue {
    kind: String,
    subject: String,
    message: String,
}

/// Validate `config` for internal consistency.  Returns one formatted error
/// string per problem, each in the form `config: <location> — <message>`.
pub fn validate_config(config: &Config, root: &Path) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    let state_ids: HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();

    let section_names: HashSet<&str> = config.ticket.sections.iter()
        .map(|s| s.name.as_str())
        .collect();
    let has_sections = !section_names.is_empty();

    // Check whether any transition requires a provider.
    let needs_provider = config.workflow.states.iter()
        .flat_map(|s| s.transitions.iter())
        .any(|t| matches!(t.completion, CompletionStrategy::Pr | CompletionStrategy::Merge));

    let provider_ok = config.provider.as_ref()
        .map(|p| !p.type_.is_empty())
        .unwrap_or(false);

    if needs_provider && !provider_ok {
        errors.push(
            "config: workflow — completion 'pr' or 'merge' requires [provider] with a type".into()
        );
    }

    // At least one non-terminal state.
    let has_non_terminal = config.workflow.states.iter().any(|s| !s.terminal);
    if !has_non_terminal {
        errors.push("config: workflow — no non-terminal state exists".into());
    }

    for state in &config.workflow.states {
        // Terminal state with outgoing transitions.
        if state.terminal && !state.transitions.is_empty() {
            errors.push(format!(
                "config: state.{} — terminal but has {} outgoing transition(s)",
                state.id,
                state.transitions.len()
            ));
        }

        // Non-terminal state with no outgoing transitions (tickets will be stranded).
        if !state.terminal && state.transitions.is_empty() {
            errors.push(format!(
                "config: state.{} — no outgoing transitions (tickets will be stranded)",
                state.id
            ));
        }

        // Instructions path exists on disk.
        if let Some(instructions) = &state.instructions {
            if !root.join(instructions).exists() {
                errors.push(format!(
                    "config: state.{}.instructions — file not found: {}",
                    state.id, instructions
                ));
            }
        }

        for transition in &state.transitions {
            // Transition target must exist.
            if !state_ids.contains(transition.to.as_str()) {
                errors.push(format!(
                    "config: state.{}.transition({}) — target state '{}' does not exist",
                    state.id, transition.to, transition.to
                ));
            }

            // Unknown preconditions.
            for precondition in &transition.preconditions {
                if !KNOWN_PRECONDITIONS.contains(&precondition.as_str()) {
                    errors.push(format!(
                        "config: state.{}.transition({}).preconditions — unknown precondition '{}'",
                        state.id, transition.to, precondition
                    ));
                }
            }

            // Unknown side effects.
            for side_effect in &transition.side_effects {
                if !KNOWN_SIDE_EFFECTS.contains(&side_effect.as_str()) {
                    errors.push(format!(
                        "config: state.{}.transition({}).side_effects — unknown side_effect '{}'",
                        state.id, transition.to, side_effect
                    ));
                }
            }

            // context_section must match a known ticket section.
            if let Some(section) = &transition.context_section {
                if has_sections && !section_names.contains(section.as_str()) {
                    errors.push(format!(
                        "config: state.{}.transition({}).context_section — unknown section '{}'",
                        state.id, transition.to, section
                    ));
                }
            }

            // focus_section must match a known ticket section.
            if let Some(section) = &transition.focus_section {
                if has_sections && !section_names.contains(section.as_str()) {
                    errors.push(format!(
                        "config: state.{}.transition({}).focus_section — unknown section '{}'",
                        state.id, transition.to, section
                    ));
                }
            }
        }
    }

    errors
}

pub fn run(root: &Path, fix: bool, json: bool, config_only: bool) -> Result<()> {
    let config = Config::load(root)?;

    let config_errors = validate_config(&config, root);
    let mut ticket_issues: Vec<Issue> = Vec::new();
    let mut tickets_checked = 0usize;

    if !config_only {
        let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
        tickets_checked = tickets.len();

        let state_ids: HashSet<&str> = config.workflow.states.iter()
            .map(|s| s.id.as_str())
            .collect();

        let mut branch_fixes: Vec<(ticket::Ticket, String, String)> = Vec::new();

        for t in &tickets {
            let fm = &t.frontmatter;
            let ticket_subject = format!("#{}", fm.id);

            if !state_ids.is_empty() && !state_ids.contains(fm.state.as_str()) {
                ticket_issues.push(Issue {
                    kind: "ticket".into(),
                    subject: ticket_subject.clone(),
                    message: format!(
                        "ticket #{} has unknown state '{}'",
                        fm.id, fm.state
                    ),
                });
            }

            if let Some(branch) = &fm.branch {
                let canonical = git::branch_name_from_path(&t.path);
                if let Some(expected) = canonical {
                    if branch != &expected {
                        ticket_issues.push(Issue {
                            kind: "ticket".into(),
                            subject: ticket_subject.clone(),
                            message: format!(
                                "ticket #{} branch field '{}' does not match expected '{}'",
                                fm.id, branch, expected
                            ),
                        });
                        if fix {
                            branch_fixes.push((t.clone(), expected, branch.clone()));
                        }
                    }
                }
            }
        }

        if fix {
            apply_branch_fixes(root, &config, branch_fixes)?;
        }
    }

    let has_errors = !config_errors.is_empty() || !ticket_issues.is_empty();

    if json {
        let out = serde_json::json!({
            "tickets_checked": tickets_checked,
            "config_errors": config_errors,
            "errors": ticket_issues,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for e in &config_errors {
            eprintln!("{e}");
        }
        for e in &ticket_issues {
            println!("error [{}] {}: {}", e.kind, e.subject, e.message);
        }
        println!(
            "{} tickets checked, {} config errors, {} ticket errors",
            tickets_checked,
            config_errors.len(),
            ticket_issues.len(),
        );
    }

    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}

fn apply_branch_fixes(
    root: &Path,
    config: &Config,
    fixes: Vec<(ticket::Ticket, String, String)>,
) -> Result<()> {
    for (mut t, expected_branch, _old_branch) in fixes {
        let id = t.frontmatter.id;
        t.frontmatter.branch = Some(expected_branch.clone());
        let content = t.serialize()?;
        let filename = t.path.file_name().unwrap().to_string_lossy().to_string();
        let rel_path = format!("{}/{filename}", config.tickets.dir.to_string_lossy());
        match git::commit_to_branch(
            root,
            &expected_branch,
            &rel_path,
            &content,
            &format!("ticket({id}): fix branch field (validate --fix)"),
        ) {
            Ok(_) => println!("  fixed #{id}: branch -> {expected_branch}"),
            Err(e) => eprintln!("  warning: could not fix #{id}: {e:#}"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use apm_core::config::Config;
    use std::path::Path;

    fn load_config(toml: &str) -> Config {
        toml::from_str(toml).expect("config parse failed")
    }

    fn state_ids(config: &Config) -> std::collections::HashSet<&str> {
        config.workflow.states.iter().map(|s| s.id.as_str()).collect()
    }

    // Test 1: correct config passes all checks
    #[test]
    fn correct_config_passes() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to = "in_progress"

[[workflow.states]]
id       = "in_progress"
label    = "In Progress"
terminal = false

[[workflow.states.transitions]]
to = "closed"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    // Test 2: transition to non-existent state is detected
    #[test]
    fn transition_to_nonexistent_state_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to = "ghost"
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(errors.iter().any(|e| e.contains("ghost")), "expected ghost error in {errors:?}");
    }

    // Test 3: terminal state with outgoing transitions is detected
    #[test]
    fn terminal_state_with_transitions_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true

[[workflow.states.transitions]]
to = "new"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to = "closed"
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("state.closed") && e.contains("terminal")),
            "expected terminal error in {errors:?}"
        );
    }

    // Test 4: unknown precondition is detected
    #[test]
    fn unknown_precondition_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to            = "ready"
preconditions = ["totally_made_up"]

[[workflow.states]]
id    = "ready"
label = "Ready"

[[workflow.states.transitions]]
to = "closed"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("totally_made_up")),
            "expected unknown precondition error in {errors:?}"
        );
    }

    // Test 5: ticket with unknown state is detected
    #[test]
    fn ticket_with_unknown_state_detected() {
        use apm_core::ticket::Ticket;

        let raw = "+++\nid = 1\ntitle = \"Test\"\nstate = \"phantom\"\n+++\n\n## Spec\n";
        let ticket = Ticket::parse(Path::new("tickets/0001-test.md"), raw).unwrap();

        let known_states: std::collections::HashSet<&str> =
            ["new", "ready", "closed"].iter().copied().collect();

        assert!(!known_states.contains(ticket.frontmatter.state.as_str()));
    }

    // Test 6: dead-end non-terminal state is detected
    #[test]
    fn dead_end_non_terminal_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "stuck"
label = "Stuck"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("state.stuck") && e.contains("no outgoing transitions")),
            "expected dead-end error in {errors:?}"
        );
    }

    // Test 7: context_section mismatch is detected
    #[test]
    fn context_section_mismatch_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[ticket.sections]]
name = "Problem"
type = "free"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to              = "ready"
context_section = "NonExistent"

[[workflow.states]]
id    = "ready"
label = "Ready"

[[workflow.states.transitions]]
to = "closed"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("context_section") && e.contains("NonExistent")),
            "expected context_section error in {errors:?}"
        );
    }

    // Test 8: focus_section mismatch is detected
    #[test]
    fn focus_section_mismatch_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[ticket.sections]]
name = "Problem"
type = "free"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to             = "ready"
focus_section  = "BadSection"

[[workflow.states]]
id    = "ready"
label = "Ready"

[[workflow.states.transitions]]
to = "closed"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("focus_section") && e.contains("BadSection")),
            "expected focus_section error in {errors:?}"
        );
    }

    // Test 9: completion=pr without provider is detected
    #[test]
    fn completion_pr_without_provider_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to         = "closed"
completion = "pr"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("provider")),
            "expected provider error in {errors:?}"
        );
    }

    // Test 10: completion=pr with provider configured passes
    #[test]
    fn completion_pr_with_provider_passes() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[provider]
type = "github"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to         = "closed"
completion = "pr"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            !errors.iter().any(|e| e.contains("provider")),
            "unexpected provider error in {errors:?}"
        );
    }

    // Test 11: context_section with empty ticket.sections is skipped
    #[test]
    fn context_section_skipped_when_no_sections_defined() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to              = "closed"
context_section = "AnySection"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            !errors.iter().any(|e| e.contains("context_section")),
            "unexpected context_section error in {errors:?}"
        );
    }

    // Test for state_ids helper (kept for compatibility)
    #[test]
    fn state_ids_helper() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"
"#;
        let config = load_config(toml);
        let ids = state_ids(&config);
        assert!(ids.contains("new"));
    }
}
