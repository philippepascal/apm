use anyhow::Result;
use apm_core::{config::Config, git, ticket::Ticket};
use std::path::Path;

pub fn run(root: &Path) -> Result<()> {
    let config = Config::load(root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    std::fs::create_dir_all(&tickets_dir)?;

    // Fetch all remote refs.
    match git::fetch_all(root) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("warning: fetch failed (no remote configured?): {e:#}");
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

    // Detect merged branches and report (auto-transitions are a future enhancement).
    let merged = git::merged_into_main(root)?;
    for branch in &merged {
        let suffix = branch.trim_start_matches("ticket/");
        let filename = format!("{suffix}.md");
        let local_path = tickets_dir.join(&filename);
        if local_path.exists() {
            if let Ok(t) = Ticket::load(&local_path) {
                if t.frontmatter.state == "implemented" {
                    eprintln!(
                        "info: ticket #{} branch merged → consider `apm state {} accepted`",
                        t.frontmatter.id, t.frontmatter.id
                    );
                }
            }
        }
    }

    println!(
        "sync: {} ticket branch{} refreshed",
        updated,
        if updated == 1 { "" } else { "es" }
    );
    Ok(())
}
