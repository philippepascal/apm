use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, id: Option<&str>, agent: Option<String>, role: Option<String>) -> Result<()> {
    let mut stdout = std::io::stdout();
    match id {
        None => apm_core::prompt::discover(root, &mut stdout),
        Some(id) => apm_core::prompt::run(root, id, agent.as_deref(), role.as_deref(), &mut stdout),
    }
}
