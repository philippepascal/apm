use anyhow::{bail, Result};
use apm_core::{config::Config, ticket};
use chrono::Local;

pub fn run(id: u32, new_state: String) -> Result<()> {
    let root = crate::repo_root()?;
    let config = Config::load(&root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    let mut tickets = ticket::load_all(&tickets_dir)?;
    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found");
    };
    let old_state = t.frontmatter.state.clone();
    t.frontmatter.state = new_state.clone();
    t.frontmatter.updated = Some(Local::now().date_naive());
    append_history(&mut t.body, &old_state, &new_state);
    t.save()?;
    println!("#{id}: {old_state} → {new_state}");
    Ok(())
}

fn append_history(body: &mut String, from: &str, to: &str) {
    let today = Local::now().format("%Y-%m-%d");
    let row = format!("| {today} | manual | {from} → {to} | |");
    if body.contains("## History") {
        if !body.ends_with('\n') {
            body.push('\n');
        }
        body.push_str(&row);
        body.push('\n');
    } else {
        body.push_str(&format!(
            "\n## History\n\n| Date | Actor | Transition | Note |\n|------|-------|------------|------|\n{row}\n"
        ));
    }
}
