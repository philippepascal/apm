use anyhow::{bail, Result};
use apm_core::{config::Config, ticket};
use chrono::Local;

pub fn run(id: u32, field: String, value: String) -> Result<()> {
    let root = crate::repo_root()?;
    let config = Config::load(&root)?;
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
    t.save()?;
    println!("#{id}: {field} = {value}");
    Ok(())
}
