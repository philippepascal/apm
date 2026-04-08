use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, new_state: String, no_aggressive: bool, force: bool) -> Result<()> {
    let out = apm_core::state::transition(root, id_arg, new_state, no_aggressive, force)?;
    println!("{}: {} → {}", out.id, out.old_state, out.new_state);
    if let Some(wt) = out.worktree_path {
        println!("{}", wt.display());
    }
    for msg in &out.messages {
        println!("{msg}");
    }
    for w in &out.warnings {
        eprintln!("{w}");
    }
    Ok(())
}
