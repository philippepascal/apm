use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::path::Path;

pub fn run(root: &Path, id: u32, reason: Option<String>) -> Result<()> {
    let config = Config::load(root)?;
    let agent = std::env::var("APM_AGENT_NAME").unwrap_or_else(|_| "apm".into());
    ticket::close(root, &config, id, reason.as_deref(), &agent)
}
