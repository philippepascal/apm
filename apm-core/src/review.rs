use crate::config::Config;

pub const SENTINEL: &str = "# --- edit the ticket spec below this line ---";

/// Split ticket body into (spec_part, history_section).
/// `history_section` starts with `\n## History` so it can be spliced back directly.
pub fn split_body(body: &str) -> (String, String) {
    if let Some(idx) = body.find("\n## History") {
        (body[..idx].to_string(), body[idx..].to_string())
    } else if body.starts_with("## History") {
        (String::new(), body.to_string())
    } else {
        (body.to_string(), String::new())
    }
}

/// Extract the editable spec from the saved temp file.
/// Everything after the sentinel line (or after leading `# ` comment lines
/// if the sentinel was deleted) is the spec content.
pub fn extract_spec(content: &str) -> String {
    if let Some(idx) = content.find(SENTINEL) {
        let after = &content[idx + SENTINEL.len()..];
        after.trim_start_matches('\n').to_string()
    } else {
        // Sentinel was deleted — strip leading comment lines as fallback.
        let mut out = Vec::new();
        let mut past_header = false;
        for line in content.lines() {
            if !past_header && (line == "#" || line.starts_with("# ")) {
                continue;
            }
            past_header = true;
            out.push(line);
        }
        out.join("\n")
    }
}

/// Returns the manual (non-auto) transitions available from the current state
/// as `(to, label, hint)` tuples.
pub fn available_transitions(config: &Config, current_state: &str) -> Vec<(String, String, String)> {
    let terminal_ids = config.terminal_state_ids();

    let state_cfg = config.workflow.states.iter().find(|s| s.id == current_state);

    if let Some(sc) = state_cfg {
        if !sc.transitions.is_empty() {
            return sc.transitions.iter()
                .filter(|tr| !tr.trigger.starts_with("event:"))
                .map(|tr| (tr.to.clone(), tr.label.clone(), tr.hint.clone()))
                .collect();
        }
    }

    // No explicit transitions: all non-terminal, non-current states are valid.
    config.workflow.states.iter()
        .filter(|s| s.id != current_state && !terminal_ids.contains(s.id.as_str()))
        .map(|s| (s.id.clone(), s.label.clone(), String::new()))
        .collect()
}

/// Convert plain `- ` bullets in `### Amendment requests` to `- [ ] ` checkboxes.
/// Lines already formatted as `- [ ]`, `- [x]`, or `- [X]` are left unchanged.
/// Only lines inside the section (up to the next `##` heading) are affected.
pub fn normalize_amendments(spec: String) -> String {
    const SECTION: &str = "### Amendment requests";

    let parts: Vec<&str> = spec.split('\n').collect();
    let Some(sec_pos) = parts.iter().position(|l| *l == SECTION) else {
        return spec;
    };

    let mut result: Vec<String> = Vec::with_capacity(parts.len());
    let mut in_section = false;

    for (i, line) in parts.iter().enumerate() {
        if i < sec_pos {
            result.push((*line).to_string());
        } else if i == sec_pos {
            in_section = true;
            result.push((*line).to_string());
        } else if in_section && line.starts_with("##") {
            in_section = false;
            result.push((*line).to_string());
        } else if in_section
            && line.starts_with("- ")
            && !line.starts_with("- [ ]")
            && !line.starts_with("- [x]")
            && !line.starts_with("- [X]")
        {
            result.push(format!("- [ ]{}", &line[1..]));
        } else {
            result.push((*line).to_string());
        }
    }

    result.join("\n")
}

/// Splice trimmed new spec with the original history section.
pub fn apply_review(new_spec: &str, history_section: &str) -> String {
    format!("{}{}", new_spec.trim_end(), history_section)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_body_with_history() {
        let body = "## Spec\n\nsome content\n## History\n\n| row |";
        let (spec, hist) = split_body(body);
        assert_eq!(spec, "## Spec\n\nsome content");
        assert_eq!(hist, "\n## History\n\n| row |");
    }

    #[test]
    fn split_body_no_history() {
        let body = "## Spec\n\nsome content";
        let (spec, hist) = split_body(body);
        assert_eq!(spec, body);
        assert_eq!(hist, "");
    }

    #[test]
    fn split_body_history_at_start() {
        let body = "## History\n\n| row |";
        let (spec, hist) = split_body(body);
        assert_eq!(spec, "");
        assert_eq!(hist, body);
    }

    #[test]
    fn extract_spec_with_sentinel() {
        let content = format!("# comment\n{SENTINEL}\n\nmy spec here");
        assert_eq!(extract_spec(&content), "my spec here");
    }

    #[test]
    fn extract_spec_without_sentinel_strips_comments() {
        let content = "# comment line\n# another comment\nactual spec\nmore spec";
        assert_eq!(extract_spec(content), "actual spec\nmore spec");
    }

    fn make_config(toml_states: &str) -> Config {
        let full = format!(
            "[project]\nname = \"test\"\n\n[workflow]\n{toml_states}"
        );
        toml::from_str(&full).expect("config parse")
    }

    #[test]
    fn available_transitions_filters_event_triggers() {
        let config = make_config(r#"
[[workflow.states]]
id = "ready"
label = "Ready"
[[workflow.states.transitions]]
to = "in_progress"
label = "Start"
trigger = "command:start"
[[workflow.states.transitions]]
to = "closed"
label = "Auto-close"
trigger = "event:pr_merged"

[[workflow.states]]
id = "in_progress"
label = "In Progress"

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#);

        let transitions = available_transitions(&config, "ready");
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].0, "in_progress");
    }

    #[test]
    fn available_transitions_fallback_excludes_terminal_and_current() {
        let config = make_config(r#"
[[workflow.states]]
id = "new"
label = "New"

[[workflow.states]]
id = "ready"
label = "Ready"

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#);

        let transitions = available_transitions(&config, "new");
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].0, "ready");
    }

    #[test]
    fn normalize_amendments_converts_plain_bullets() {
        let input = "### Amendment requests\n- fix this\n- also fix that\n\n## Other".to_string();
        let output = normalize_amendments(input);
        assert!(output.contains("- [ ] fix this"));
        assert!(output.contains("- [ ] also fix that"));
    }

    #[test]
    fn normalize_amendments_leaves_checkboxes_unchanged() {
        let input = "### Amendment requests\n- [ ] already checkbox\n- [x] done\n- [X] done cap".to_string();
        let output = normalize_amendments(input);
        assert!(output.contains("- [ ] already checkbox"));
        assert!(output.contains("- [x] done"));
        assert!(output.contains("- [X] done cap"));
    }

    #[test]
    fn normalize_amendments_leaves_outside_section_unchanged() {
        let input = "## Spec\n- plain bullet\n### Amendment requests\n- fix this".to_string();
        let output = normalize_amendments(input);
        assert!(output.contains("## Spec\n- plain bullet"));
        assert!(output.contains("- [ ] fix this"));
    }

    #[test]
    fn normalize_amendments_no_section_returns_unchanged() {
        let input = "## Spec\n- plain bullet".to_string();
        let output = normalize_amendments(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn apply_review_trims_trailing_whitespace() {
        let result = apply_review("spec content   \n\n", "\n## History\n| row |");
        assert_eq!(result, "spec content\n## History\n| row |");
    }
}
