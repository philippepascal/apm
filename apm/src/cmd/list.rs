use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::path::Path;

pub fn run(root: &Path, state_filter: Option<String>, unassigned: bool, all: bool, supervisor_filter: Option<String>, actionable_filter: Option<String>) -> Result<()> {
    let config = Config::load(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;

    let filtered = ticket::list_filtered(
        &tickets,
        &config,
        state_filter.as_deref(),
        unassigned,
        all,
        supervisor_filter.as_deref(),
        actionable_filter.as_deref(),
    );

    for t in filtered {
        let fm = &t.frontmatter;
        let agent = fm.agent.as_deref().unwrap_or("-");
        println!("{:<8} [{:<12}] {:<40} agent={}", fm.id, fm.state, fm.title, agent);
    }
    Ok(())
}
