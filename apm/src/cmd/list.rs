use anyhow::Result;
use apm_core::identity;
use std::path::Path;
use crate::ctx::CmdContext;

pub fn run(root: &Path, state_filter: Option<String>, unassigned: bool, all: bool, supervisor_filter: Option<String>, actionable_filter: Option<String>, no_aggressive: bool, mine: bool, author: Option<String>, owner: Option<String>) -> Result<()> {
    let ctx = CmdContext::load(root, no_aggressive)?;

    let mine_user: Option<String> = if mine {
        Some(identity::resolve_current_user(root))
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
