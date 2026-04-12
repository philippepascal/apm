use anyhow::{bail, Result};
use apm_core::{config::Config, git, spec, ticket, ticket_fmt};
use std::{io::Read, path::Path};

pub fn run(root: &Path, id_arg: &str, section: Option<String>, set: Option<String>, set_file: Option<String>, check: bool, mark: Option<String>, no_aggressive: bool) -> Result<()> {
    if set.is_some() && section.is_none() { bail!("--set requires --section"); }
    if set_file.is_some() && section.is_none() { bail!("--set-file requires --section"); }
    if mark.is_some() && section.is_none() { bail!("--mark requires --section"); }
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;
    let branches = git::ticket_branches(root)?;
    let branch = ticket_fmt::resolve_ticket_branch(&branches, id_arg)?;
    let id = branch.strip_prefix("ticket/").and_then(|s| s.split('-').next()).unwrap_or(id_arg).to_string();
    let rel_path = format!("{}/{}.md", config.tickets.dir.to_string_lossy(), branch.trim_start_matches("ticket/"));

    if aggressive {
        if let Err(e) = git::fetch_branch(root, &branch) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }

    let content = git::read_from_branch(root, &branch, &rel_path)?;
    if let (Some(ref name), Some(ref item)) = (&section, &mark) {
        let new = spec::mark_item(&content, name, item)?;
        git::commit_to_branch(root, &branch, &rel_path, &new, &format!("ticket({id}): mark \"{item}\" in {name}"))?;
        if aggressive {
            if let Err(e) = git::push_branch(root, &branch) {
                eprintln!("warning: push failed: {e:#}");
            }
        }
        println!("ticket #{id}: marked \"{item}\" in {name:?}"); return Ok(());
    }
    let mut t = ticket::Ticket::parse(&root.join(&rel_path), &content)?;
    let mut doc = t.document()?;
    if check {
        let errors = doc.validate(&config.ticket.sections);
        if errors.is_empty() { println!("all required sections present"); return Ok(()); }
        errors.iter().for_each(|e| eprintln!("{e}")); std::process::exit(1);
    }
    let config_active = !config.ticket.sections.is_empty();
    let Some(ref name) = section else {
        for (section_name, value) in &doc.sections {
            println!("### {section_name}\n\n{value}\n");
        }
        return Ok(());
    };
    if config_active && !config.has_section(name) {
        bail!("unknown section {:?}; not defined in [ticket.sections]", name);
    }
    let set_resolved = match (set, set_file) {
        (Some(v), _) => Some(v),
        (None, Some(path)) => Some(std::fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("--set-file: {}: {e}", path))?),
        (None, None) => None,
    };
    if let Some(value) = set_resolved {
        let text = if value == "-" { let mut b = String::new(); std::io::stdin().read_to_string(&mut b)?; b } else { value };
        let trimmed = text.trim().to_string();
        let formatted = if config_active {
            let section_config = config.find_section(name).unwrap();
            spec::apply_section_type(&section_config.type_, trimmed)
        } else {
            trimmed
        };
        spec::set_section(&mut doc, name, formatted);
        t.body = doc.serialize();
        git::commit_to_branch(root, &branch, &rel_path, &t.serialize()?, &format!("ticket({id}): set section {name}"))?;
        if aggressive {
            if let Err(e) = git::push_branch(root, &branch) {
                eprintln!("warning: push failed: {e:#}");
            }
        }
        println!("ticket #{id}: section {name:?} updated");
    } else {
        if let Some(text) = spec::get_section(&doc, name) { println!("{text}"); }
    }
    Ok(())
}
