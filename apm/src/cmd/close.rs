use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, reason: Option<String>) -> Result<()> {
    let config = Config::load(root)?;
    let agent = std::env::var("APM_AGENT_NAME").unwrap_or_else(|_| "apm".into());
    ticket::close(root, &config, id_arg, reason.as_deref(), &agent)
}
