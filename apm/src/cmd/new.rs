use anyhow::Result;
use apm_core::{
    config::Config,
    git,
    ticket::{slugify, Frontmatter, Ticket},
};
use chrono::Utc;
use std::path::Path;

pub fn run(root: &Path, title: String, side_note: bool, context: Option<String>) -> Result<()> {
    let config = Config::load(root)?;
    if side_note && !config.agents.side_tickets {
        anyhow::bail!("side tickets are disabled in apm.toml (agents.side_tickets = false)");
    }
    let tickets_dir = root.join(&config.tickets.dir);
    std::fs::create_dir_all(&tickets_dir)?;

    let id = git::next_ticket_id(root, &tickets_dir)?;
    let slug = slugify(&title);
    let filename = format!("{id:04}-{slug}.md");
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);
    let branch = format!("ticket/{id:04}-{slug}");
    let now = Utc::now();
    let author = std::env::var("APM_AGENT_NAME")
        .ok()
        .unwrap_or_else(|| "apm".into());
    let fm = Frontmatter {
        id,
        title: title.clone(),
        state: "new".into(),
        priority: 0,
        effort: 0,
        risk: 0,
        author: Some(author.clone()),
        supervisor: None,
        agent: None,
        branch: Some(branch.clone()),
        created_at: Some(now),
        updated_at: Some(now),
    };
    let when = now.format("%Y-%m-%dT%H:%MZ");
    let problem_section = match &context {
        Some(ctx) => format!("### Problem\n\n{ctx}\n\n"),
        None => "### Problem\n\n".to_string(),
    };
    let body = format!(
        "## Spec\n\n{problem_section}### Acceptance criteria\n\n### Out of scope\n\n### Approach\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|\n| {when} | — | new | {author} |\n"
    );
    let path = tickets_dir.join(&filename);
    let t = Ticket { frontmatter: fm, body, path };
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
