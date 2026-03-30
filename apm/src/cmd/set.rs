use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, field: String, value: String, no_aggressive: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;
    let mut tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let id = ticket::resolve_id_in_slice(&tickets, id_arg)?;

    if aggressive {
        let branches = git::ticket_branches(root).unwrap_or_default();
        if let Some(b) = branches.iter().find(|b| {
            b.strip_prefix("ticket/")
                .and_then(|s| s.split('-').next())
                .map(|bid| bid == id.as_str())
                .unwrap_or(false)
        }) {
            if let Err(e) = git::fetch_branch(root, b) {
                eprintln!("warning: fetch failed: {e:#}");
            }
        }
    }

    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket {id:?} not found");
    };
    ticket::set_field(&mut t.frontmatter, &field, &value)?;
    t.frontmatter.updated_at = Some(Utc::now());

    let content = t.serialize()?;
    let rel_path = format!(
        "{}/{}",
        config.tickets.dir.to_string_lossy(),
        t.path.file_name().unwrap().to_string_lossy()
    );
    let branch = t
        .frontmatter
        .branch
        .clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));

    git::commit_to_branch(
        root,
        &branch,
        &rel_path,
        &content,
        &format!("ticket({id}): set {field} = {value}"),
    )?;

    if aggressive {
        if let Err(e) = git::push_branch(root, &branch) {
            eprintln!("warning: push failed: {e:#}");
        }
    }

    println!("{id}: {field} = {value}");
    Ok(())
}
