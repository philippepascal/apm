use crate::config::Config;
use crate::ticket::Ticket;
use std::collections::HashSet;

pub fn verify_tickets(
    config: &Config,
    tickets: &[Ticket],
    merged: &HashSet<String>,
) -> Vec<String> {
    let valid_states: HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();
    let terminal = config.terminal_state_ids();

    let in_progress_states: HashSet<&str> =
        ["in_progress", "implemented"].iter().copied().collect();

    let mut issues: Vec<String> = Vec::new();

    for t in tickets {
        let fm = &t.frontmatter;

        // Skip terminal-state tickets.
        if terminal.contains(fm.state.as_str()) { continue; }

        let prefix = format!("#{} [{}]", fm.id, fm.state);

        // State value not in config.
        if !valid_states.is_empty() && !valid_states.contains(fm.state.as_str()) {
            issues.push(format!("{prefix}: unknown state {:?}", fm.state));
        }

        // Frontmatter id doesn't match filename numeric prefix.
        if let Some(name) = t.path.file_name().and_then(|n| n.to_str()) {
            let expected_prefix = format!("{:04}", fm.id);
            if !name.starts_with(&expected_prefix) {
                issues.push(format!("{prefix}: id {} does not match filename {name}", fm.id));
            }
        }

        // in_progress/implemented with no branch.
        if in_progress_states.contains(fm.state.as_str()) && fm.branch.is_none() {
            issues.push(format!("{prefix}: state requires branch but none set"));
        }

        // Branch merged but ticket not yet closed.
        if let Some(branch) = &fm.branch {
            if (fm.state == "in_progress" || fm.state == "implemented")
                && merged.contains(branch.as_str())
            {
                issues.push(format!("{prefix}: branch {branch} is merged but ticket not closed"));
            }
        }

        // Missing ## Spec section.
        if !t.body.contains("## Spec") {
            issues.push(format!("{prefix}: missing ## Spec section"));
        }

        // Missing ## History section.
        if !t.body.contains("## History") {
            issues.push(format!("{prefix}: missing ## History section"));
        }

        // Validate document structure (required sections non-empty, AC items present).
        if let Ok(doc) = t.document() {
            for err in doc.validate(&config.ticket.sections) {
                issues.push(format!("{prefix}: {err}"));
            }
        }
    }

    issues
}
