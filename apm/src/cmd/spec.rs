use anyhow::{bail, Result};
use apm_core::{config::{Config, SectionType}, git, ticket};
use std::io::Read;
use std::path::Path;

const KNOWN_SECTIONS: &[&str] = &[
    "Problem",
    "Acceptance criteria",
    "Out of scope",
    "Approach",
    "Open questions",
];

pub fn run(
    root: &Path,
    id: u32,
    section: Option<String>,
    set: Option<String>,
    check: bool,
    mark: Option<String>,
) -> Result<()> {
    if set.is_some() && section.is_none() {
        bail!("--set requires --section");
    }
    if mark.is_some() && section.is_none() {
        bail!("--mark requires --section");
    }

    let config = Config::load(root)?;

    let prefix = format!("ticket/{id:04}-");
    let branches = git::ticket_branches(root)?;
    let branch = branches.into_iter().find(|b| b.starts_with(&prefix));
    let Some(branch) = branch else {
        bail!("ticket #{id} not found");
    };

    let suffix = branch.trim_start_matches("ticket/");
    let filename = format!("{suffix}.md");
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);
    let dummy_path = root.join(&rel_path);

    let content = git::read_from_branch(root, &branch, &rel_path)?;

    if let (Some(ref name), Some(ref item_text)) = (&section, &mark) {
        let new_content = mark_item(&content, name, item_text)?;
        git::commit_to_branch(
            root,
            &branch,
            &rel_path,
            &new_content,
            &format!("ticket({id}): mark \"{item_text}\" in {name}"),
        )?;
        println!("ticket #{id}: marked \"{item_text}\" in {name:?}");
        return Ok(());
    }

    let mut t = ticket::Ticket::parse(&dummy_path, &content)?;
    let mut doc = t.document()?;

    if check {
        let errors = doc.validate();
        if errors.is_empty() {
            println!("all required sections present");
            return Ok(());
        }
        for e in &errors {
            eprintln!("{e}");
        }
        std::process::exit(1);
    }

    let config_active = !config.ticket.sections.is_empty();

    if let Some(ref name) = section {
        if config_active {
            if !config.ticket.sections.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
                bail!("unknown section {:?}; not defined in [ticket.sections]", name);
            }
        } else if !KNOWN_SECTIONS.iter().any(|s| s.eq_ignore_ascii_case(name)) {
            bail!(
                "unknown section {:?}; valid sections: {}",
                name,
                KNOWN_SECTIONS.join(", ")
            );
        }

        if let Some(value) = set {
            let text = if value == "-" {
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf
            } else {
                value
            };

            let trimmed = text.trim().to_string();

            if config_active {
                let section_cfg = config.ticket.sections.iter()
                    .find(|s| s.name.eq_ignore_ascii_case(name))
                    .unwrap();
                let formatted = apply_section_type(&section_cfg.type_, trimmed);
                if is_doc_field(name) {
                    set_section_doc(&mut doc, name, formatted);
                    t.body = doc.serialize();
                } else {
                    set_section_body(&mut t.body, name, &formatted);
                }
            } else {
                let canon = canon_section(name);
                set_section(&mut doc, canon, trimmed);
                t.body = doc.serialize();
            }

            let new_content = t.serialize()?;
            git::commit_to_branch(
                root,
                &branch,
                &rel_path,
                &new_content,
                &format!("ticket({id}): set section {name}"),
            )?;
            println!("ticket #{id}: section {name:?} updated");
        } else {
            if config_active {
                let section_cfg = config.ticket.sections.iter()
                    .find(|s| s.name.eq_ignore_ascii_case(name))
                    .unwrap();
                if is_doc_field(&section_cfg.name) {
                    print_section(&doc, &section_cfg.name);
                } else {
                    print_section_body(&t.body, &section_cfg.name);
                }
            } else {
                let canon = canon_section(name);
                print_section(&doc, canon);
            }
        }
    } else {
        print_all(&doc);
    }

    Ok(())
}

fn is_doc_field(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(lower.as_str(),
        "problem" | "acceptance criteria" | "out of scope" | "approach"
        | "open questions" | "amendment requests")
}

fn apply_section_type(type_: &SectionType, value: String) -> String {
    match type_ {
        SectionType::Tasks => {
            value.lines()
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
                .join("\n")
        }
        SectionType::Qa => {
            value.lines()
                .map(|line| {
                    let l = line.trim();
                    if l.is_empty() {
                        String::new()
                    } else {
                        format!("**Q:** {l}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        SectionType::Free => value,
    }
}

fn set_section_doc(doc: &mut ticket::TicketDocument, name: &str, value: String) {
    match name.to_lowercase().as_str() {
        "problem" => doc.problem = value,
        "acceptance criteria" => {
            doc.acceptance_criteria = value.lines()
                .filter_map(|line| {
                    let l = line.trim();
                    if l.starts_with("- [ ] ") {
                        Some(ticket::ChecklistItem { checked: false, text: l[6..].to_string() })
                    } else if l.starts_with("- [x] ") || l.starts_with("- [X] ") {
                        Some(ticket::ChecklistItem { checked: true, text: l[6..].to_string() })
                    } else {
                        None
                    }
                })
                .collect();
        }
        "out of scope" => doc.out_of_scope = value,
        "approach" => doc.approach = value,
        "open questions" => doc.open_questions = Some(value),
        "amendment requests" => {
            doc.amendment_requests = Some(value.lines()
                .filter_map(|line| {
                    let l = line.trim();
                    if l.starts_with("- [ ] ") {
                        Some(ticket::ChecklistItem { checked: false, text: l[6..].to_string() })
                    } else if l.starts_with("- [x] ") || l.starts_with("- [X] ") {
                        Some(ticket::ChecklistItem { checked: true, text: l[6..].to_string() })
                    } else {
                        None
                    }
                })
                .collect());
        }
        _ => {}
    }
}

fn set_section_body(body: &mut String, name: &str, value: &str) {
    let heading = format!("### {name}\n");
    if let Some(pos) = body.find(&heading) {
        let after_heading = pos + heading.len();
        let skip = body[after_heading..].len() - body[after_heading..].trim_start_matches('\n').len();
        let content_start = after_heading + skip;
        let rest = &body[content_start..];
        let end = rest.find("\n## ")
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

fn print_section_body(body: &str, name: &str) {
    let heading = format!("### {name}\n");
    if let Some(pos) = body.find(&heading) {
        let after_heading = pos + heading.len();
        let skip = body[after_heading..].len() - body[after_heading..].trim_start_matches('\n').len();
        let content_start = after_heading + skip;
        let rest = &body[content_start..];
        let end = rest.find("\n## ")
            .or_else(|| rest.find("\n### "))
            .map(|p| p)
            .unwrap_or(rest.len());
        println!("{}", rest[..end].trim());
    }
}

fn canon_section<'a>(name: &'a str) -> &'a str {
    KNOWN_SECTIONS
        .iter()
        .find(|s| s.eq_ignore_ascii_case(name))
        .copied()
        .unwrap_or(name)
}

fn print_section(doc: &ticket::TicketDocument, name: &str) {
    match name.to_lowercase().as_str() {
        "problem" => println!("{}", doc.problem),
        "acceptance criteria" => {
            for item in &doc.acceptance_criteria {
                println!("- [{}] {}", if item.checked { "x" } else { " " }, item.text);
            }
        }
        "out of scope" => println!("{}", doc.out_of_scope),
        "approach" => println!("{}", doc.approach),
        "open questions" => {
            if let Some(oq) = &doc.open_questions {
                println!("{oq}");
            }
        }
        _ => {}
    }
}

fn print_all(doc: &ticket::TicketDocument) {
    let sections = [
        ("Problem", doc.problem.clone()),
        (
            "Acceptance criteria",
            doc.acceptance_criteria
                .iter()
                .map(|i| format!("- [{}] {}", if i.checked { "x" } else { " " }, i.text))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        ("Out of scope", doc.out_of_scope.clone()),
        ("Approach", doc.approach.clone()),
    ];
    for (name, body) in &sections {
        println!("### {name}\n");
        println!("{body}\n");
    }
    if let Some(oq) = &doc.open_questions {
        println!("### Open questions\n");
        println!("{oq}\n");
    }
}

fn mark_item(content: &str, section: &str, item_text: &str) -> Result<String> {
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

fn set_section(doc: &mut ticket::TicketDocument, name: &str, value: String) {
    match name {
        "Problem" => doc.problem = value,
        "Acceptance criteria" => {
            doc.acceptance_criteria = value
                .lines()
                .filter_map(|line| {
                    let l = line.trim();
                    if l.starts_with("- [ ] ") {
                        Some(ticket::ChecklistItem { checked: false, text: l[6..].to_string() })
                    } else if l.starts_with("- [x] ") || l.starts_with("- [X] ") {
                        Some(ticket::ChecklistItem { checked: true, text: l[6..].to_string() })
                    } else {
                        None
                    }
                })
                .collect();
        }
        "Out of scope" => doc.out_of_scope = value,
        "Approach" => doc.approach = value,
        "Open questions" => doc.open_questions = Some(value),
        _ => {}
    }
}
