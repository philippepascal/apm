use anyhow::Result;
use apm_core::{
    config::Config,
    git,
    ticket::{slugify, Frontmatter, Ticket},
};
use chrono::Local;
use std::path::Path;

pub fn run(root: &Path, title: String) -> Result<()> {
    let config = Config::load(root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    std::fs::create_dir_all(&tickets_dir)?;

    let id = git::next_ticket_id(root, &tickets_dir)?;
    let slug = slugify(&title);
    let filename = format!("{id:04}-{slug}.md");
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);
    let branch = format!("ticket/{id:04}-{slug}");
    let today = Local::now().date_naive();

    let fm = Frontmatter {
        id,
        title: title.clone(),
        state: "new".into(),
        priority: 0,
        effort: 0,
        risk: 0,
        agent: None,
        branch: Some(branch.clone()),
        created: Some(today),
        updated: Some(today),
    };
    let body = "## Spec\n\n### Problem\n\n### Acceptance criteria\n\n### Out of scope\n\n## History\n\n| Date | Actor | Transition | Note |\n|------|-------|------------|------|\n";
    let path = tickets_dir.join(&filename);
    let t = Ticket { frontmatter: fm, body: body.into(), path };
    let content = t.serialize()?;

    git::commit_to_branch(
        root,
        &branch,
        &rel_path,
        &content,
        &format!("ticket({id}): create {title}"),
    )?;

    println!("Created ticket #{id}: {filename} (branch: {branch})");
    Ok(())
}
