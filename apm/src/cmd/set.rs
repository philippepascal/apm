use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Local;
use std::path::Path;

pub fn run(root: &Path, id: u32, field: String, value: String) -> Result<()> {
    let config = Config::load(root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    let mut tickets = ticket::load_all(&tickets_dir)?;
    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found");
    };
    let fm = &mut t.frontmatter;
    match field.as_str() {
        "priority" => fm.priority = value.parse().map_err(|_| anyhow::anyhow!("priority must be 0–255"))?,
        "effort"   => fm.effort   = value.parse().map_err(|_| anyhow::anyhow!("effort must be 0–255"))?,
        "risk"     => fm.risk     = value.parse().map_err(|_| anyhow::anyhow!("risk must be 0–255"))?,
        "agent"    => fm.agent    = if value == "-" { None } else { Some(value.clone()) },
        "branch"   => fm.branch   = if value == "-" { None } else { Some(value.clone()) },
        "title"    => fm.title    = value.clone(),
        other => bail!("unknown field: {other}"),
    }
    fm.updated = Some(Local::now().date_naive());

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
        .unwrap_or_else(|| format!("ticket/{id:04}"));

    git::commit_to_branch(
        root,
        &branch,
        &rel_path,
        &content,
        &format!("ticket({id}): set {field} = {value}"),
    )?;

    println!("#{id}: {field} = {value}");
    Ok(())
}
