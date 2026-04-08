use anyhow::{bail, Result};
use apm_core::{git, ticket};
use chrono::Utc;
use std::path::Path;
use crate::ctx::CmdContext;

pub fn run(root: &Path, id_arg: &str, field: String, value: String, no_aggressive: bool) -> Result<()> {
    let ctx = CmdContext::load(root, no_aggressive)?;
    let id = ticket::resolve_id_in_slice(&ctx.tickets, id_arg)?;
    let mut tickets = ctx.tickets;

    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket {id:?} not found");
    };
    if field == "owner" {
        ticket::check_owner(root, t)?;
    }
    ticket::set_field(&mut t.frontmatter, &field, &value)?;
    t.frontmatter.updated_at = Some(Utc::now());

    let content = t.serialize()?;
    let rel_path = format!(
        "{}/{}",
        ctx.config.tickets.dir.to_string_lossy(),
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

    if ctx.aggressive {
        if let Err(e) = git::push_branch(root, &branch) {
            eprintln!("warning: push failed: {e:#}");
        }
    }

    println!("{id}: {field} = {value}");
    Ok(())
}
