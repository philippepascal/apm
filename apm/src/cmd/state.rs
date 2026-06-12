use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, new_state: String, no_aggressive: bool, force: bool) -> Result<()> {
    let ids: Vec<&str> = id_arg.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    if ids.is_empty() {
        return Ok(());
    }

    if ids.len() == 1 {
        let out = apm_core::state::transition(root, ids[0], new_state, no_aggressive, force)?;
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
        return Ok(());
    }

    let total = ids.len();
    let mut failures = 0usize;
    for id in &ids {
        match apm_core::state::transition(root, id, new_state.clone(), no_aggressive, force) {
            Ok(out) => {
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
            }
            Err(e) => {
                #[allow(clippy::print_stderr)]
                {
                    eprintln!("{id}: {e:#}");
                }
                failures += 1;
            }
        }
    }
    if failures > 0 {
        anyhow::bail!("{failures} of {total} transitions failed");
    }
    Ok(())
}
