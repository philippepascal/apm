use std::path::Path;
use anyhow::Result;
use apm_core::{config::Config, ticket::Ticket};

pub struct CmdContext {
    pub config: Config,
    pub tickets: Vec<Ticket>,
    pub aggressive: bool,
}

impl CmdContext {
    pub fn load(root: &Path, no_aggressive: bool) -> Result<Self> {
        let config = Config::load(root)?;
        let aggressive = config.sync.aggressive && !no_aggressive;
        crate::util::fetch_if_aggressive(root, aggressive);
        let tickets = if aggressive {
            apm_core::ticket::load_all_from_git_classified(root, &config.tickets.dir)?
        } else {
            apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?
        };
        Ok(Self { config, tickets, aggressive })
    }

    pub fn load_config_only(root: &Path) -> Result<Config> {
        Config::load(root)
    }
}
