use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use std::path::Path;

pub fn run(root: &Path, id: u32) -> Result<()> {
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
    let t = ticket::Ticket::parse(&dummy_path, &content)?;

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
