use anyhow::Result;
use apm_core::{
    config::Config,
    ticket::{self, slugify, Frontmatter, Ticket},
};
use chrono::Local;

pub fn run(title: String) -> Result<()> {
    let root = crate::repo_root()?;
    let config = Config::load(&root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    let id = ticket::next_id(&tickets_dir)?;
    let slug = slugify(&title);
    let filename = format!("{id:04}-{slug}.md");
    let path = tickets_dir.join(&filename);
    let today = Local::now().date_naive();
    let fm = Frontmatter {
        id,
        title: title.clone(),
        state: "new".into(),
        priority: 0,
        effort: 0,
        risk: 0,
        agent: None,
        branch: None,
        created: Some(today),
        updated: Some(today),
    };
    let body = "## Spec\n\n### Problem\n\n### Acceptance criteria\n\n### Out of scope\n\n## History\n\n| Date | Actor | Transition | Note |\n|------|-------|------------|------|\n";
    let t = Ticket { frontmatter: fm, body: body.into(), path };
    t.save()?;
    println!("Created ticket #{id}: {filename}");
    Ok(())
}

