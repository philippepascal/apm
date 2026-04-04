use anyhow::Result;
use apm_core::{config::Config, git, identity, ticket};
use std::path::Path;

pub fn run(root: &Path, state_filter: Option<String>, unassigned: bool, all: bool, supervisor_filter: Option<String>, actionable_filter: Option<String>, no_aggressive: bool, mine: bool, author: Option<String>, owner: Option<String>) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    if aggressive {
        if let Err(e) = git::fetch_all(root) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }

    let mine_user: Option<String> = if mine {
        Some(identity::resolve_current_user(root))
    } else {
        None
    };
    let author_filter = if mine { None } else { author };

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;

    let filtered = ticket::list_filtered(
        &tickets,
        &config,
        state_filter.as_deref(),
        unassigned,
        all,
        supervisor_filter.as_deref(),
        actionable_filter.as_deref(),
        author_filter.as_deref(),
        owner.as_deref(),
        mine_user.as_deref(),
    );

    for t in filtered {
        let fm = &t.frontmatter;
        println!("{:<8} [{:<12}] {}", fm.id, fm.state, fm.title);
    }
    Ok(())
}
