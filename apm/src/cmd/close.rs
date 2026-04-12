use anyhow::Result;
use apm_core::{config::Config, git, ticket, ticket_fmt};
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, reason: Option<String>, no_aggressive: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;
    let agent = std::env::var("APM_AGENT_NAME").unwrap_or_else(|_| "apm".into());

    let branches = git::ticket_branches(root).unwrap_or_default();
    let branch = ticket_fmt::resolve_ticket_branch(&branches, id_arg).ok();

    if let Some(ref b) = branch {
        crate::util::fetch_branch_if_aggressive(root, b, aggressive);
    }

    let msgs = ticket::close(root, &config, id_arg, reason.as_deref(), &agent, aggressive)?;
    for msg in &msgs {
        println!("{msg}");
    }

    Ok(())
}
