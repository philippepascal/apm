use anyhow::{bail, Result};
use crate::config::SectionType;
use crate::ticket::{ChecklistItem, TicketDocument};

pub fn get_section(doc: &TicketDocument, name: &str) -> Option<String> {
    match name.to_lowercase().as_str() {
        "problem" => Some(doc.problem.clone()),
        "acceptance criteria" => Some(
            doc.acceptance_criteria
                .iter()
                .map(|i| format!("- [{}] {}", if i.checked { "x" } else { " " }, i.text))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        "out of scope" => Some(doc.out_of_scope.clone()),
        "approach" => Some(doc.approach.clone()),
        "open questions" => doc.open_questions.clone(),
        "amendment requests" => doc.amendment_requests.as_ref().map(|items| {
            items
                .iter()
                .map(|i| format!("- [{}] {}", if i.checked { "x" } else { " " }, i.text))
                .collect::<Vec<_>>()
                .join("\n")
        }),
        _ => None,
    }
}

pub fn set_section(doc: &mut TicketDocument, name: &str, value: String) {
    match name.to_lowercase().as_str() {
        "problem" => doc.problem = value,
        "acceptance criteria" => {
            doc.acceptance_criteria = parse_checklist(&value);
        }
        "out of scope" => doc.out_of_scope = value,
        "approach" => doc.approach = value,
        "open questions" => doc.open_questions = Some(value),
        "amendment requests" => {
            doc.amendment_requests = Some(parse_checklist(&value));
        }
        _ => {}
    }
}

pub fn apply_section_type(type_: &SectionType, value: String) -> String {
    match type_ {
        SectionType::Tasks => value
            .lines()
            .map(|line| {
                let l = line.trim();
                if l.is_empty() {
                    String::new()
                } else if l.starts_with("- [ ] ") || l.starts_with("- [x] ") || l.starts_with("- [X] ") {
                    l.to_string()
                } else {
                    format!("- [ ] {l}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        SectionType::Qa => value
            .lines()
            .map(|line| {
                let l = line.trim();
                if l.is_empty() {
                    String::new()
                } else {
                    format!("**Q:** {l}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        SectionType::Free => value,
    }
}

pub fn mark_item(content: &str, section: &str, item_text: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let section_lower = section.to_lowercase();

    let header_idx = lines.iter().position(|line| {
        line.strip_prefix("### ")
            .map(|rest| rest.to_lowercase() == section_lower)
            .unwrap_or(false)
    });

    let Some(header_idx) = header_idx else {
        bail!("section {:?} not found", section);
    };

    let mut matches: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate().skip(header_idx + 1) {
        if line.starts_with("##") {
            break;
        }
        if let Some(text) = line.strip_prefix("- [ ] ") {
            if text.to_lowercase().contains(&item_text.to_lowercase()) {
                matches.push(i);
            }
        }
    }

    match matches.len() {
        0 => bail!(
            "no unchecked item matching {:?} found in section {:?}",
            item_text,
            section
        ),
        1 => {
            let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
            new_lines[matches[0]] = new_lines[matches[0]].replacen("- [ ] ", "- [x] ", 1);
            let joined = new_lines.join("\n");
            if content.ends_with('\n') {
                Ok(joined + "\n")
            } else {
                Ok(joined)
            }
        }
        _ => {
            let mut msg = format!(
                "ambiguous: {} unchecked items match {:?} in section {:?}:",
                matches.len(),
                item_text,
                section
            );
            for i in &matches {
                msg.push_str(&format!("\n  {}", lines[*i]));
            }
            bail!("{}", msg);
        }
    }
}

pub fn is_doc_field(name: &str) -> bool {
    matches!(name.to_lowercase().as_str(),
        "problem" | "acceptance criteria" | "out of scope" | "approach"
        | "open questions" | "amendment requests")
}

pub fn get_section_body(body: &str, name: &str) -> Option<String> {
    let heading = format!("### {name}\n");
    let pos = body.find(&heading)?;
    let after_heading = pos + heading.len();
    let skip = body[after_heading..].len()
        - body[after_heading..].trim_start_matches('\n').len();
    let content_start = after_heading + skip;
    let rest = &body[content_start..];
    let end = rest
        .find("\n## ")
        .or_else(|| rest.find("\n### "))
        .unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

pub fn set_section_body(body: &mut String, name: &str, value: &str) {
    let heading = format!("### {name}\n");
    if let Some(pos) = body.find(&heading) {
        let after_heading = pos + heading.len();
        let skip = body[after_heading..].len()
            - body[after_heading..].trim_start_matches('\n').len();
        let content_start = after_heading + skip;
        let rest = &body[content_start..];
        let end = rest
            .find("\n## ")
            .or_else(|| rest.find("\n### "))
            .map(|p| content_start + p + 1)
            .unwrap_or(body.len());
        let before = body[..after_heading].to_string();
        let after = body[end..].to_string();
        if value.is_empty() {
            *body = format!("{}\n{}", before, after);
        } else {
            *body = format!("{}\n{}\n\n{}", before, value, after);
        }
    }
}

fn parse_checklist(value: &str) -> Vec<ChecklistItem> {
    value
        .lines()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ticket::TicketDocument;
    use crate::config::SectionType;

    fn base_doc() -> TicketDocument {
        TicketDocument::parse(
            "## Spec\n\n### Problem\n\nA bug exists\n\
             \n### Acceptance criteria\n\n- [ ] Fix the bug\n- [x] Write tests\n\
             \n### Out of scope\n\nNothing\n\
             \n### Approach\n\nUse a hammer\n\
             \n### Open questions\n\nWhy?\n",
        )
        .unwrap()
    }

    #[test]
    fn get_section_problem() {
        let doc = base_doc();
        assert_eq!(get_section(&doc, "Problem"), Some("A bug exists".to_string()));
    }

    #[test]
    fn get_section_acceptance_criteria_markdown() {
        let doc = base_doc();
        let result = get_section(&doc, "Acceptance criteria").unwrap();
        assert!(result.contains("- [ ] Fix the bug"));
        assert!(result.contains("- [x] Write tests"));
    }

    #[test]
    fn get_section_unknown_returns_none() {
        let doc = base_doc();
        assert_eq!(get_section(&doc, "Nonexistent"), None);
    }

    #[test]
    fn set_section_problem_case_insensitive() {
        let mut doc = base_doc();
        set_section(&mut doc, "problem", "New problem".to_string());
        assert_eq!(doc.problem, "New problem");
    }

    #[test]
    fn set_section_acceptance_criteria_parses_checklist() {
        let mut doc = base_doc();
        set_section(&mut doc, "acceptance criteria", "- [ ] Item one\n- [x] Item two".to_string());
        assert_eq!(doc.acceptance_criteria.len(), 2);
        assert!(!doc.acceptance_criteria[0].checked);
        assert_eq!(doc.acceptance_criteria[0].text, "Item one");
        assert!(doc.acceptance_criteria[1].checked);
    }

    #[test]
    fn set_section_amendment_requests_parses_checklist() {
        let mut doc = base_doc();
        set_section(&mut doc, "amendment requests", "- [ ] Fix docs".to_string());
        let ar = doc.amendment_requests.unwrap();
        assert_eq!(ar.len(), 1);
        assert_eq!(ar[0].text, "Fix docs");
    }

    #[test]
    fn apply_section_type_tasks_wraps_bare_line() {
        let result = apply_section_type(&SectionType::Tasks, "Do something".to_string());
        assert_eq!(result, "- [ ] Do something");
    }

    #[test]
    fn apply_section_type_tasks_leaves_formatted_unchanged() {
        let result = apply_section_type(&SectionType::Tasks, "- [ ] Already formatted".to_string());
        assert_eq!(result, "- [ ] Already formatted");
    }

    #[test]
    fn apply_section_type_qa_prefixes_line() {
        let result = apply_section_type(&SectionType::Qa, "What is it?".to_string());
        assert_eq!(result, "**Q:** What is it?");
    }

    #[test]
    fn apply_section_type_free_unchanged() {
        let result = apply_section_type(&SectionType::Free, "Some text".to_string());
        assert_eq!(result, "Some text");
    }

    #[test]
    fn mark_item_replaces_unchecked() {
        let content = "### Acceptance criteria\n- [ ] Fix the bug\n- [ ] Write tests\n";
        let result = mark_item(content, "Acceptance criteria", "Fix the bug").unwrap();
        assert!(result.contains("- [x] Fix the bug"));
        assert!(result.contains("- [ ] Write tests"));
    }

    #[test]
    fn mark_item_error_no_match() {
        let content = "### Acceptance criteria\n- [ ] Fix the bug\n";
        let err = mark_item(content, "Acceptance criteria", "nonexistent").unwrap_err();
        assert!(err.to_string().contains("no unchecked item"));
    }

    #[test]
    fn mark_item_error_ambiguous() {
        let content = "### Acceptance criteria\n- [ ] Fix the bug now\n- [ ] Fix the bug later\n";
        let err = mark_item(content, "Acceptance criteria", "Fix the bug").unwrap_err();
        assert!(err.to_string().contains("ambiguous"));
    }
}
