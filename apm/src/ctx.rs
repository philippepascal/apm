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
        let mut tickets = if aggressive {
            apm_core::ticket::load_all_from_git_classified(root, &config.tickets.dir)?
        } else {
            apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?
        };
        let branchless = apm_core::ticket::load_from_default_branch(
            root,
            &config.tickets.dir,
            &config.project.default_branch,
        )?;
        if !branchless.is_empty() {
            let seen: std::collections::HashSet<String> =
                tickets.iter().map(|t| t.frontmatter.id.clone()).collect();
            for t in branchless {
                if !seen.contains(&t.frontmatter.id) {
                    tickets.push(t);
                }
            }
            tickets.sort_by_key(|t| t.frontmatter.created_at);
        }
        Ok(Self { config, tickets, aggressive })
    }

    pub fn load_config_only(root: &Path) -> Result<Config> {
        Config::load(root)
    }
}
