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
    if new_state == "ammend" {
        ensure_amendment_section(&mut t.body);
    }
    append_history(&mut t.body, &old_state, &new_state);
    t.save()?;
    println!("#{id}: {old_state} → {new_state}");
    Ok(())
}

fn ensure_amendment_section(body: &mut String) {
    if body.contains("### Amendment requests") {
        return;
    }
    let placeholder = "\n### Amendment requests\n\n<!-- Add amendment requests below -->\n";
    // Insert after ### Out of scope if present, otherwise before ## History
    if let Some(pos) = body.find("### Out of scope") {
        // Find the end of the Out of scope block (next ### or ## or end of string)
        let after = &body[pos..];
        let block_end = after[1..]
            .find("\n##")
            .map(|p| pos + 1 + p)
            .unwrap_or(body.len());
        body.insert_str(block_end, placeholder);
    } else if let Some(pos) = body.find("## History") {
        body.insert_str(pos, &format!("{}\n", placeholder));
    } else {
        body.push_str(placeholder);
    }
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
