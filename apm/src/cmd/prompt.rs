use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, id: &str, agent: Option<String>, role: Option<String>) -> Result<()> {
    let mut stdout = std::io::stdout();
    apm_core::prompt::run(root, id, agent.as_deref(), role.as_deref(), &mut stdout)
}
