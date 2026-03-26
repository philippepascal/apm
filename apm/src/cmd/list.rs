use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::collections::HashSet;
use std::path::Path;

pub fn run(root: &Path, state_filter: Option<String>, unassigned: bool, all: bool) -> Result<()> {
    let config = Config::load(root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    let tickets = ticket::load_all(&tickets_dir)?;

    let terminal: HashSet<&str> = config.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();

    let filtered = tickets.iter().filter(|t| {
        let fm = &t.frontmatter;
        let state_ok = state_filter.as_deref().map_or(true, |s| fm.state == s);
        let agent_ok = !unassigned || fm.agent.is_none();
        let terminal_ok = all || !terminal.contains(fm.state.as_str());
        state_ok && agent_ok && terminal_ok
    });

    for t in filtered {
        let fm = &t.frontmatter;
        let agent = fm.agent.as_deref().unwrap_or("-");
        println!("#{:<4} [{:<12}] {:<40} agent={}", fm.id, fm.state, fm.title, agent);
    }
    Ok(())
}
