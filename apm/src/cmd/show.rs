use anyhow::{bail, Result};
use apm_core::{config::Config, ticket};
use std::path::Path;

pub fn run(root: &Path, id: u32) -> Result<()> {
    let config = Config::load(root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    let tickets = ticket::load_all(&tickets_dir)?;
    let Some(t) = tickets.iter().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found");
    };
    let fm = &t.frontmatter;
    println!("#{} — {}", fm.id, fm.title);
    println!("state:    {}", fm.state);
    println!("priority: {}  effort: {}  risk: {}", fm.priority, fm.effort, fm.risk);
    if let Some(a) = &fm.agent { println!("agent:    {a}"); }
    if let Some(b) = &fm.branch { println!("branch:   {b}"); }
    println!();
    print!("{}", t.body);
    Ok(())
}
