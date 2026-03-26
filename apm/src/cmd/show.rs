use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use std::path::Path;

pub fn run(root: &Path, id: u32) -> Result<()> {
    let config = Config::load(root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    let tickets = ticket::load_all(&tickets_dir)?;

    // Try the local cache first; fall back to reading from the ticket branch.
    let t = if let Some(t) = tickets.iter().find(|t| t.frontmatter.id == id) {
        t.clone()
    } else {
        load_from_branch(root, &config, id)?
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

fn load_from_branch(root: &Path, config: &Config, id: u32) -> Result<apm_core::ticket::Ticket> {
    // Find a local or remote branch matching ticket/<id:04>-*.
    let prefix = format!("ticket/{id:04}-");
    let branches = git::ticket_branches(root)?;
    let branch = branches.into_iter().find(|b| b.starts_with(&prefix));

    let Some(branch) = branch else {
        bail!("ticket #{id} not found");
    };

    let suffix = branch.trim_start_matches("ticket/");
    let filename = format!("{suffix}.md");
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);

    let content = git::read_from_branch(root, &branch, &rel_path)?;
    let dummy_path = root.join(&config.tickets.dir).join(&filename);
    ticket::Ticket::parse(&dummy_path, &content)
}
