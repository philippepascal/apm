use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
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
) -> Result<()> {
    if set.is_some() && section.is_none() {
        bail!("--set requires --section");
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

    if let Some(ref name) = section {
        if !KNOWN_SECTIONS.iter().any(|s| s.eq_ignore_ascii_case(name)) {
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
            let canon = canon_section(name);
            set_section(&mut doc, canon, trimmed);

            t.body = doc.serialize();
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
            let canon = canon_section(name);
            print_section(&doc, canon);
        }
    } else {
        print_all(&doc);
    }

    Ok(())
}

fn canon_section<'a>(name: &'a str) -> &'a str {
    // Return the matching known section with its canonical casing.
    KNOWN_SECTIONS
        .iter()
        .find(|s| s.eq_ignore_ascii_case(name))
        .copied()
        .unwrap_or(name)
}

fn print_section(doc: &ticket::TicketDocument, name: &str) {
    match name {
        "Problem" => println!("{}", doc.problem),
        "Acceptance criteria" => {
            for item in &doc.acceptance_criteria {
                println!("- [{}] {}", if item.checked { "x" } else { " " }, item.text);
            }
        }
        "Out of scope" => println!("{}", doc.out_of_scope),
        "Approach" => println!("{}", doc.approach),
        "Open questions" => {
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
