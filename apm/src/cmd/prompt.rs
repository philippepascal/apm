use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, id: Option<&str>, agent: Option<String>, role: Option<String>, explain: bool) -> Result<()> {
    let mut stdout = std::io::stdout();
    match id {
        None => apm_core::prompt::discover(root, &mut stdout),
        Some(id) => {
            if explain {
                apm_core::prompt::explain(root, id, agent.as_deref(), role.as_deref(), &mut stdout)
            } else {
                apm_core::prompt::run(root, id, agent.as_deref(), role.as_deref(), &mut stdout)
            }
        }
    }
}
