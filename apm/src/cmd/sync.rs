use anyhow::Result;
use apm_core::{config::Config, git, ticket::Ticket};
use std::path::Path;

pub fn run(root: &Path, offline: bool, quiet: bool) -> Result<()> {
    let config = Config::load(root)?;

    if !offline {
        match git::fetch_all(root) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("warning: fetch failed (no remote configured?): {e:#}");
            }
        }
    }

    // Detect merged branches and fire implemented → accepted auto-transition.
    let branches = git::ticket_branches(root)?;
    let merged = git::merged_into_main(root)?;
    let mut transitioned = 0usize;

    for branch in &merged {
        let suffix = branch.trim_start_matches("ticket/");
        let filename = format!("{suffix}.md");
        let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);

        let content = match git::read_from_branch(root, branch, &rel_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let dummy_path = root.join(&rel_path);
        let mut t = match Ticket::parse(&dummy_path, &content) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if t.frontmatter.state != "implemented" { continue; }

        let now = chrono::Utc::now();
        t.frontmatter.state = "accepted".into();
        t.frontmatter.updated_at = Some(now);
        let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
        crate::cmd::state::append_history(&mut t.body, "implemented", "accepted", &when, "apm sync");

        let updated = match t.serialize() {
            Ok(c) => c,
            Err(e) => { eprintln!("warning: ticket({}) serialize: {e:#}", t.frontmatter.id); continue; }
        };
        let id = t.frontmatter.id;
        match git::commit_to_branch(root, "main", &rel_path, &updated,
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
            "sync: {} ticket branch{} visible, {} auto-transitioned",
            branches.len(),
            if branches.len() == 1 { "" } else { "es" },
            transitioned,
        );
    }
    Ok(())
}
