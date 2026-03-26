use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Local;
use std::path::Path;

pub fn run(root: &Path, id: u32, new_state: String) -> Result<()> {
    let config = Config::load(root)?;
    let valid_states: std::collections::HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();
    if !valid_states.is_empty() && !valid_states.contains(new_state.as_str()) {
        let list: Vec<&str> = config.workflow.states.iter().map(|s| s.id.as_str()).collect();
        bail!("unknown state {:?} — valid states: {}", new_state, list.join(", "));
    }
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
        &format!("ticket({id}): {old_state} → {new_state}"),
    )?;

    println!("#{id}: {old_state} → {new_state}");
    Ok(())
}

pub fn ensure_amendment_section(body: &mut String) {
    if body.contains("### Amendment requests") {
        return;
    }
    let placeholder = "\n### Amendment requests\n\n<!-- Add amendment requests below -->\n";
    if let Some(pos) = body.find("### Out of scope") {
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
