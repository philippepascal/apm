use anyhow::Result;
use apm_core::{config::Config, git, ticket};
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

pub fn run(root: &Path, fix: bool, json: bool) -> Result<()> {
    let config = Config::load(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;

    let mut errors: Vec<Issue> = Vec::new();
    let warnings: Vec<Issue> = Vec::new();

    // ── Config checks ──────────────────────────────────────────────────────

    let state_ids: HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();

    // At least one non-terminal state.
    let has_non_terminal = config.workflow.states.iter().any(|s| !s.terminal);
    if !has_non_terminal {
        errors.push(Issue {
            kind: "config".into(),
            subject: "workflow".into(),
            message: "no non-terminal state exists".into(),
        });
    }

    for state in &config.workflow.states {
        // Terminal state with outgoing transitions.
        if state.terminal && !state.transitions.is_empty() {
            errors.push(Issue {
                kind: "config".into(),
                subject: format!("state:{}", state.id),
                message: format!(
                    "state '{}' is terminal but has {} outgoing transition(s)",
                    state.id,
                    state.transitions.len()
                ),
            });
        }

        // Instructions path exists on disk (only when instructions is set).
        if let Some(instructions) = &state.instructions {
            let path = root.join(instructions);
            if !path.exists() {
                errors.push(Issue {
                    kind: "config".into(),
                    subject: format!("state:{}", state.id),
                    message: format!(
                        "state '{}' instructions path '{}' does not exist",
                        state.id, instructions
                    ),
                });
            }
        }

        for transition in &state.transitions {
            // Transition target must exist.
            if !state_ids.contains(transition.to.as_str()) {
                errors.push(Issue {
                    kind: "config".into(),
                    subject: format!("state:{}->{}", state.id, transition.to),
                    message: format!(
                        "transition from '{}' to '{}': target state does not exist",
                        state.id, transition.to
                    ),
                });
            }

            // Unknown preconditions.
            for precondition in &transition.preconditions {
                if !KNOWN_PRECONDITIONS.contains(&precondition.as_str()) {
                    errors.push(Issue {
                        kind: "config".into(),
                        subject: format!("state:{}->{}", state.id, transition.to),
                        message: format!(
                            "transition from '{}' to '{}': unknown precondition '{}'",
                            state.id, transition.to, precondition
                        ),
                    });
                }
            }

            // Unknown side effects.
            for side_effect in &transition.side_effects {
                if !KNOWN_SIDE_EFFECTS.contains(&side_effect.as_str()) {
                    errors.push(Issue {
                        kind: "config".into(),
                        subject: format!("state:{}->{}", state.id, transition.to),
                        message: format!(
                            "transition from '{}' to '{}': unknown side_effect '{}'",
                            state.id, transition.to, side_effect
                        ),
                    });
                }
            }
        }
    }

    // ── Ticket checks ──────────────────────────────────────────────────────

    let tickets_checked = tickets.len();
    let mut branch_fixes: Vec<(ticket::Ticket, String, String)> = Vec::new();

    for t in &tickets {
        let fm = &t.frontmatter;
        let ticket_subject = format!("#{}", fm.id);

        // State must be declared.
        if !state_ids.is_empty() && !state_ids.contains(fm.state.as_str()) {
            errors.push(Issue {
                kind: "ticket".into(),
                subject: ticket_subject.clone(),
                message: format!(
                    "ticket #{} has unknown state '{}'",
                    fm.id, fm.state
                ),
            });
        }

        // Branch field must match canonical name derived from filename.
        if let Some(branch) = &fm.branch {
            let canonical = git::branch_name_from_path(&t.path);
            if let Some(expected) = canonical {
                if branch != &expected {
                    errors.push(Issue {
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

    // ── Output ─────────────────────────────────────────────────────────────

    if json {
        let out = serde_json::json!({
            "tickets_checked": tickets_checked,
            "errors": errors,
            "warnings": warnings,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for e in &errors {
            println!("error [{}] {}: {}", e.kind, e.subject, e.message);
        }
        for w in &warnings {
            println!("warning [{}] {}: {}", w.kind, w.subject, w.message);
        }
        println!(
            "{} tickets checked, {} errors, {} warnings",
            tickets_checked,
            errors.len(),
            warnings.len()
        );
    }

    // ── Fix ────────────────────────────────────────────────────────────────

    if fix {
        apply_branch_fixes(root, &config, branch_fixes)?;
    }

    if !errors.is_empty() {
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
    use apm_core::config::Config;

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

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let ids = state_ids(&config);

        for state in &config.workflow.states {
            for t in &state.transitions {
                assert!(ids.contains(t.to.as_str()), "target '{}' missing", t.to);
            }
        }
        for state in &config.workflow.states {
            if state.terminal {
                assert!(state.transitions.is_empty());
            }
        }
        assert!(config.workflow.states.iter().any(|s| !s.terminal));
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
        let ids = state_ids(&config);

        let bad: Vec<_> = config.workflow.states.iter()
            .flat_map(|s| s.transitions.iter().map(move |t| (&s.id, &t.to)))
            .filter(|(_, to)| !ids.contains(to.as_str()))
            .collect();

        assert_eq!(bad.len(), 1);
        assert_eq!(bad[0].1, "ghost");
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
"#;
        let config = load_config(toml);

        let bad: Vec<_> = config.workflow.states.iter()
            .filter(|s| s.terminal && !s.transitions.is_empty())
            .collect();

        assert_eq!(bad.len(), 1);
        assert_eq!(bad[0].id, "closed");
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
"#;
        let config = load_config(toml);
        let known: std::collections::HashSet<&str> =
            super::KNOWN_PRECONDITIONS.iter().copied().collect();

        let bad: Vec<_> = config.workflow.states.iter()
            .flat_map(|s| s.transitions.iter())
            .flat_map(|t| t.preconditions.iter())
            .filter(|p| !known.contains(p.as_str()))
            .collect();

        assert_eq!(bad.len(), 1);
        assert_eq!(bad[0], "totally_made_up");
    }

    // Test 5: ticket with unknown state is detected
    #[test]
    fn ticket_with_unknown_state_detected() {
        use apm_core::ticket::Ticket;
        use std::path::Path;

        let raw = "+++\nid = 1\ntitle = \"Test\"\nstate = \"phantom\"\n+++\n\n## Spec\n";
        let ticket = Ticket::parse(Path::new("tickets/0001-test.md"), raw).unwrap();

        let known_states: std::collections::HashSet<&str> =
            ["new", "ready", "closed"].iter().copied().collect();

        assert!(!known_states.contains(ticket.frontmatter.state.as_str()));
    }
}
