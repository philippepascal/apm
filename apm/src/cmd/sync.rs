use anyhow::Result;
use apm_core::{config::Config, git, sync};
use std::io::{self, BufRead, Write};
use std::path::Path;

pub fn run(root: &Path, offline: bool, quiet: bool, no_aggressive: bool, auto_close: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    if !offline {
        match git::fetch_all(root) {
            Ok(_) => {
                git::sync_local_ticket_refs(root);
            }
            Err(e) => {
                eprintln!("warning: fetch failed (no remote configured?): {e:#}");
            }
        }
    }

    let candidates = sync::detect(root, &config)?;

    let branches = git::ticket_branches(root)?;
    if !quiet {
        println!(
            "sync: {} ticket branch{} visible",
            branches.len(),
            if branches.len() == 1 { "" } else { "es" },
        );
    }

    if !candidates.close.is_empty() {
        let confirmed = auto_close || (!quiet && prompt_close(&candidates.close)?);
        if confirmed {
            sync::apply(root, &config, &candidates, "apm-sync", aggressive)?;
        }
    }

    Ok(())
}

fn prompt_close(candidates: &[sync::CloseCandidate]) -> Result<bool> {
    println!("\nTickets ready to close:");
    for c in candidates {
        println!("  #{}  {}  ({})", c.ticket.frontmatter.id, c.ticket.frontmatter.title, c.reason);
    }
    print!("\nClose all? [y/N] ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
}
