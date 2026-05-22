use anyhow::Result;
use apm_core::config::resolve_identity;
use std::path::Path;
use crate::ctx::CmdContext;

pub fn run(root: &Path, state_filter: Option<String>, unassigned: bool, all: bool, actionable_filter: Option<String>, no_aggressive: bool, mine: bool, author: Option<String>, owner: Option<String>) -> Result<()> {
    let ctx = CmdContext::load(root, no_aggressive)?;

    let mine_user: Option<String> = if mine {
        Some(resolve_identity(root))
    } else {
        None
    };
    let author_filter = if mine { None } else { author };

    let filtered = apm_core::ticket::list_filtered(
        &ctx.tickets,
        &ctx.config,
        state_filter.as_deref(),
        unassigned,
        all,
        actionable_filter.as_deref(),
        author_filter.as_deref(),
        owner.as_deref(),
        mine_user.as_deref(),
    );

    let mut stale_tickets: Vec<(&str, &str)> = Vec::new();
    let mut diverged_tickets: Vec<(&str, &str)> = Vec::new();

    for t in &filtered {
        let fm = &t.frontmatter;
        let owner = fm.owner.as_deref().unwrap_or("-");
        let base = match fm.target_branch.as_deref() {
            Some(branch) => apm_core::epic::epic_id_from_branch(branch).to_owned(),
            None => ctx.config.project.default_branch.clone(),
        };
        let id_display = if t.local_stale {
            format!("*{}", fm.id)
        } else {
            fm.id.clone()
        };
        println!("{:<9} [{:<12}] {:<16} {:<12} {}", id_display, fm.state, owner, base, fm.title);

        if t.local_stale {
            stale_tickets.push((&fm.id, &fm.title));
        }
        if t.local_diverged {
            diverged_tickets.push((&fm.id, &fm.title));
        }
    }

    if !diverged_tickets.is_empty() {
        eprintln!();
        eprintln!("warning: local ref has diverged from origin on {} ticket(s) — showing local content:", diverged_tickets.len());
        for (id, title) in &diverged_tickets {
            eprintln!("    {}  {}", id, title);
        }
    }

    if !stale_tickets.is_empty() {
        println!();
        println!("  * local ref behind origin — run `apm sync` to fast-forward:");
        for (id, title) in &stale_tickets {
            println!("      *{}  {}", id, title);
        }
    }

    Ok(())
}
