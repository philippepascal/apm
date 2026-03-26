use anyhow::Result;
use apm_core::{config::Config, git, ticket::Ticket};
use chrono::Local;
use std::path::Path;

pub fn run(root: &Path, offline: bool, quiet: bool) -> Result<()> {
    let config = Config::load(root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    std::fs::create_dir_all(&tickets_dir)?;

    if !offline {
        match git::fetch_all(root) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("warning: fetch failed (no remote configured?): {e:#}");
            }
        }
    }

    // Read each ticket/* branch and write to local cache.
    let branches = git::ticket_branches(root)?;
    let mut updated = 0usize;

    for branch in &branches {
        // Branch is ticket/0001-my-ticket; file is tickets/0001-my-ticket.md
        let suffix = branch.trim_start_matches("ticket/");
        let filename = format!("{suffix}.md");
        let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);

        match git::read_from_branch(root, branch, &rel_path) {
            Ok(content) => {
                let local_path = tickets_dir.join(&filename);
                std::fs::write(&local_path, &content)?;
                updated += 1;
            }
            Err(e) => {
                eprintln!("warning: could not read {branch}: {e:#}");
            }
        }
    }

    // Detect merged branches and fire implemented → accepted auto-transition.
    let merged = git::merged_into_main(root)?;
    let mut transitioned = 0usize;
    for branch in &merged {
        let suffix = branch.trim_start_matches("ticket/");
        let filename = format!("{suffix}.md");
        let local_path = tickets_dir.join(&filename);
        if !local_path.exists() { continue; }
        let mut t = match Ticket::load(&local_path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if t.frontmatter.state != "implemented" { continue; }

        t.frontmatter.state = "accepted".into();
        t.frontmatter.updated = Some(Local::now().date_naive());
        let today = Local::now().format("%Y-%m-%d");
        let row = format!("| {today} | sync | implemented → accepted | branch merged |");
        if t.body.contains("## History") {
            if !t.body.ends_with('\n') { t.body.push('\n'); }
            t.body.push_str(&row);
            t.body.push('\n');
        } else {
            t.body.push_str(&format!(
                "\n## History\n\n| Date | Actor | Transition | Note |\n|------|-------|------------|------|\n{row}\n"
            ));
        }

        let content = match t.serialize() {
            Ok(c) => c,
            Err(e) => { eprintln!("warning: ticket({}) serialize: {e:#}", t.frontmatter.id); continue; }
        };
        let id = t.frontmatter.id;
        let rel_path = format!("{}/{filename}", config.tickets.dir.to_string_lossy());
        match git::commit_to_branch(root, "main", &rel_path, &content,
            &format!("ticket({id}): implemented → accepted (branch merged)")) {
            Ok(_) => {
                if !quiet { println!("#{id}: implemented → accepted (branch merged)"); }
                transitioned += 1;
            }
            Err(e) => eprintln!("warning: ticket({id}) transition failed: {e:#}"),
        }
    }

    if !offline {
        git::push_ticket_branches(root);
    }

    if !quiet {
        println!(
            "sync: {} ticket branch{} refreshed, {} auto-transitioned",
            updated,
            if updated == 1 { "" } else { "es" },
            transitioned,
        );
    }
    Ok(())
}
