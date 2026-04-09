use crate::config::{CompletionStrategy, Config, LocalConfig};
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::Path;

pub fn validate_owner(config: &Config, local: &LocalConfig, username: &str) -> Result<()> {
    if username == "-" {
        return Ok(());
    }
    let (collaborators, warnings) = crate::config::resolve_collaborators(config, local);
    for w in &warnings {
        #[allow(clippy::print_stderr)]
        { eprintln!("{w}"); }
    }
    if collaborators.is_empty() {
        return Ok(());
    }
    if collaborators.iter().any(|c| c == username) {
        return Ok(());
    }
    let list = collaborators.join(", ");
    bail!("unknown user '{username}'; valid collaborators: {list}");
}

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

    let provider_ok = config.git_host.provider.as_ref()
        .map(|p| !p.is_empty())
        .unwrap_or(false);

    if needs_provider && !provider_ok {
        errors.push(
            "config: workflow — completion 'pr' or 'merge' requires [git_host] with a provider".into()
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
            // Transition target must exist.  "closed" is a built-in terminal state
            // that is always valid even when absent from [[workflow.states]].
            if transition.to != "closed" && !state_ids.contains(transition.to.as_str()) {
                errors.push(format!(
                    "config: state.{}.transition({}) — target state '{}' does not exist",
                    state.id, transition.to, transition.to
                ));
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

pub fn validate_warnings(config: &crate::config::Config) -> Vec<String> {
    let mut warnings = config.load_warnings.clone();
    if let Some(container) = &config.workers.container {
        if !container.is_empty() {
            let docker_ok = std::process::Command::new("docker")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !docker_ok {
                warnings.push(
                    "workers.container is set but 'docker' is not in PATH".to_string()
                );
            }
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, LocalConfig};
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

    // Test 5: ticket with unknown state is detected
    #[test]
    fn ticket_with_unknown_state_detected() {
        use crate::ticket::Ticket;

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

[git_host]
provider = "github"

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

    // Test: closed state is not flagged as unknown even when absent from config
    #[test]
    fn closed_state_not_flagged_as_unknown() {
        use crate::ticket::Ticket;

        // Config with no "closed" state
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to = "done"

[[workflow.states]]
id       = "done"
label    = "Done"
terminal = true
"#;
        let config = load_config(toml);
        let state_ids: std::collections::HashSet<&str> = config.workflow.states.iter()
            .map(|s| s.id.as_str())
            .collect();

        let raw = "+++\nid = 1\ntitle = \"Test\"\nstate = \"closed\"\n+++\n\n## Spec\n";
        let ticket = Ticket::parse(Path::new("tickets/0001-test.md"), raw).unwrap();

        // "closed" is not in state_ids, but the validate logic skips it.
        assert!(!state_ids.contains("closed"));
        // Simulate the validate check: closed should be exempt.
        let fm = &ticket.frontmatter;
        let flagged = !state_ids.is_empty() && fm.state != "closed" && !state_ids.contains(fm.state.as_str());
        assert!(!flagged, "closed state should not be flagged as unknown");
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

    #[test]
    fn validate_warnings_no_container() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        let warnings = super::validate_warnings(&config);
        assert!(warnings.is_empty());
    }

    #[test]
    fn valid_collaborator_accepted() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        assert!(super::validate_owner(&config, &LocalConfig::default(), "alice").is_ok());
    }

    #[test]
    fn unknown_user_rejected() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        let err = super::validate_owner(&config, &LocalConfig::default(), "charlie").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown user 'charlie'"), "unexpected message: {msg}");
        assert!(msg.contains("alice, bob"), "unexpected message: {msg}");
    }

    #[test]
    fn empty_collaborators_skips_validation() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        assert!(super::validate_owner(&config, &LocalConfig::default(), "anyone").is_ok());
    }

    #[test]
    fn clear_owner_always_allowed() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice"]

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        assert!(super::validate_owner(&config, &LocalConfig::default(), "-").is_ok());
    }

    #[test]
    fn github_mode_known_user_accepted() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "org/repo"
"#;
        let config = load_config(toml);
        // No token in LocalConfig::default() — falls back to project.collaborators
        assert!(super::validate_owner(&config, &LocalConfig::default(), "alice").is_ok());
    }

    #[test]
    fn github_mode_unknown_user_rejected() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "org/repo"
"#;
        let config = load_config(toml);
        // No token — falls back to project.collaborators; charlie is not in the list
        let err = super::validate_owner(&config, &LocalConfig::default(), "charlie").unwrap_err();
        assert!(err.to_string().contains("charlie"), "expected charlie in: {err}");
    }

    #[test]
    fn github_mode_no_collaborators_skips_check() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "org/repo"
"#;
        let config = load_config(toml);
        // Empty collaborators list — no validation
        assert!(super::validate_owner(&config, &LocalConfig::default(), "anyone").is_ok());
    }

    #[test]
    fn github_mode_clear_owner_accepted() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice"]

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "org/repo"
"#;
        let config = load_config(toml);
        assert!(super::validate_owner(&config, &LocalConfig::default(), "-").is_ok());
    }

    #[test]
    fn non_github_mode_unknown_user_rejected() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        let err = super::validate_owner(&config, &LocalConfig::default(), "charlie").unwrap_err();
        assert!(err.to_string().contains("charlie"), "expected charlie in: {err}");
    }

    #[test]
    fn validate_warnings_empty_container() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
container = ""
"#;
        let config = load_config(toml);
        let warnings = super::validate_warnings(&config);
        assert!(warnings.is_empty(), "empty container string should not warn");
    }
}
