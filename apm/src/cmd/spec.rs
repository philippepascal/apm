use anyhow::{bail, Result};
use apm_core::{config::Config, git, spec, ticket};
use std::{io::Read, path::Path};
const KNOWN_SECTIONS: &[&str] = &["Problem", "Acceptance criteria", "Out of scope", "Approach", "Open questions"];
pub fn run(root: &Path, id_arg: &str, section: Option<String>, set: Option<String>, check: bool, mark: Option<String>, no_aggressive: bool) -> Result<()> {
    if set.is_some() && section.is_none() { bail!("--set requires --section"); }
    if mark.is_some() && section.is_none() { bail!("--mark requires --section"); }
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;
    let branches = git::ticket_branches(root)?;
    let branch = git::resolve_ticket_branch(&branches, id_arg)?;
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
        let errors = doc.validate();
        if errors.is_empty() { println!("all required sections present"); return Ok(()); }
        errors.iter().for_each(|e| eprintln!("{e}")); std::process::exit(1);
    }
    let config_active = !config.ticket.sections.is_empty();
    let Some(ref name) = section else {
        for (n, k) in [("Problem","problem"),("Acceptance criteria","acceptance criteria"),("Out of scope","out of scope"),("Approach","approach")] {
            println!("### {n}\n\n{}\n", spec::get_section(&doc, k).unwrap_or_default());
        }
        if let Some(oq) = spec::get_section(&doc, "open questions") { println!("### Open questions\n\n{oq}\n"); } return Ok(()); };
    if config_active { if !config.ticket.sections.iter().any(|s| s.name.eq_ignore_ascii_case(name)) { bail!("unknown section {:?}; not defined in [ticket.sections]", name); } }
    else if !KNOWN_SECTIONS.iter().any(|s| s.eq_ignore_ascii_case(name)) { bail!("unknown section {:?}; valid sections: {}", name, KNOWN_SECTIONS.join(", ")); }
    if let Some(value) = set {
        let text = if value == "-" { let mut b = String::new(); std::io::stdin().read_to_string(&mut b)?; b } else { value };
        let trimmed = text.trim().to_string();
        if config_active {
            let formatted = spec::apply_section_type(&config.ticket.sections.iter().find(|s| s.name.eq_ignore_ascii_case(name)).unwrap().type_, trimmed);
            if spec::is_doc_field(name) { spec::set_section(&mut doc, name, formatted); t.body = doc.serialize(); }
            else { spec::set_section_body(&mut t.body, name, &formatted); }
        } else { spec::set_section(&mut doc, name, trimmed); t.body = doc.serialize(); }
        git::commit_to_branch(root, &branch, &rel_path, &t.serialize()?, &format!("ticket({id}): set section {name}"))?;
        if aggressive {
            if let Err(e) = git::push_branch(root, &branch) {
                eprintln!("warning: push failed: {e:#}");
            }
        }
        println!("ticket #{id}: section {name:?} updated");
    } else {
        let display = if config_active { config.ticket.sections.iter().find(|s| s.name.eq_ignore_ascii_case(name)).unwrap().name.clone() } else { name.clone() };
        if spec::is_doc_field(&display) { if let Some(text) = spec::get_section(&doc, &display) { println!("{text}"); } }
        else if let Some(text) = spec::get_section_body(&t.body, &display) { println!("{text}"); }
    }
    Ok(())
}
