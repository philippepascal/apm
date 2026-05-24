use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, id: Option<&str>, agent: Option<String>, role: Option<String>, system: bool, message: bool, explain: bool) -> Result<()> {
    let mut stdout = std::io::stdout();
    match (id, agent.as_deref(), role.as_deref()) {
        (Some(id), agent_ov, role_ov) => {
            if explain {
                apm_core::prompt::explain(root, id, agent_ov, role_ov, &mut stdout)
            } else if system {
                apm_core::prompt::run(root, id, agent_ov, role_ov, &mut stdout)
            } else if message {
                apm_core::prompt::run_message(root, id, agent_ov, role_ov, &mut stdout)
            } else {
                apm_core::prompt::run_full(root, id, agent_ov, role_ov, &mut stdout)
            }
        }
        (None, Some(a), Some(r)) => {
            if explain {
                apm_core::prompt::explain_without_ticket(root, a, r, &mut stdout)
            } else {
                apm_core::prompt::run_without_ticket(root, a, r, &mut stdout)
            }
        }
        (None, _, _) => apm_core::prompt::discover(root, &mut stdout),
    }
}
