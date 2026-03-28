use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::collections::HashSet;
use std::path::Path;

pub fn run(root: &Path, state_filter: Option<String>, unassigned: bool, all: bool, supervisor_filter: Option<String>, actionable_filter: Option<String>) -> Result<()> {
    let config = Config::load(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;

    let terminal: HashSet<&str> = config.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();

    // Build a map from state id → actionable actors for fast lookup.
    let actionable_map: std::collections::HashMap<&str, &Vec<String>> = config.workflow.states.iter()
        .map(|s| (s.id.as_str(), &s.actionable))
        .collect();

    let filtered = tickets.iter().filter(|t| {
        let fm = &t.frontmatter;
        let state_ok = state_filter.as_deref().map_or(true, |s| fm.state == s);
        let agent_ok = !unassigned || fm.agent.is_none();
        let state_is_terminal = state_filter.as_deref().map_or(false, |s| terminal.contains(s));
        let terminal_ok = all || state_is_terminal || !terminal.contains(fm.state.as_str());
        let supervisor_ok = supervisor_filter.as_deref().map_or(true, |s| fm.supervisor.as_deref() == Some(s));
        let actionable_ok = actionable_filter.as_deref().map_or(true, |actor| {
            actionable_map.get(fm.state.as_str())
                .map_or(false, |actors| actors.iter().any(|a| a == actor || a == "any"))
        });
        state_ok && agent_ok && terminal_ok && supervisor_ok && actionable_ok
    });

    for t in filtered {
        let fm = &t.frontmatter;
        let agent = fm.agent.as_deref().unwrap_or("-");
        println!("#{:<4} [{:<12}] {:<40} agent={}", fm.id, fm.state, fm.title, agent);
    }
    Ok(())
}
