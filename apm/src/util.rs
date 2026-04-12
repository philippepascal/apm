use std::io::{self, BufRead, Write};
use std::path::Path;
use apm_core::git;

/// Run `git fetch --all` when `aggressive` is true; emit a warning on failure.
pub fn fetch_if_aggressive(root: &Path, aggressive: bool) {
    if aggressive {
        if let Err(e) = git::fetch_all(root) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }
}

/// Run `git fetch <branch>` when `aggressive` is true; emit a warning on failure.
pub fn fetch_branch_if_aggressive(root: &Path, branch: &str, aggressive: bool) {
    if aggressive {
        if let Err(e) = git::fetch_branch(root, branch) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }
}

/// Print `prompt`, flush stdout, read one line, return true iff the answer is "y".
pub fn prompt_yes_no(prompt: &str) -> io::Result<bool> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
}
