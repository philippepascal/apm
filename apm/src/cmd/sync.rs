use anyhow::Result;
use apm_core::{config::Config, git, sync};
use std::io::{self, BufRead, Write};
use std::path::Path;

pub fn run(root: &Path, offline: bool, quiet: bool, no_aggressive: bool, auto_close: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    if !offline {
        let mut sync_warnings: Vec<String> = Vec::new();
        match git::fetch_all(root) {
            Ok(_) => {
                git::sync_local_ticket_refs(root, &mut sync_warnings);
            }
            Err(e) => {
                eprintln!("warning: fetch failed (no remote configured?): {e:#}");
            }
        }
        if let Err(e) = git::push_default_branch(root, &config.project.default_branch) {
            eprintln!("warning: push {branch} failed: {e:#}", branch = config.project.default_branch);
        }
        for w in &sync_warnings {
            eprintln!("{w}");
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
            let apply_out = sync::apply(root, &config, &candidates, "apm-sync", aggressive)?;
            for (id, err) in &apply_out.failed {
                eprintln!("warning: could not close {id:?}: {err}");
            }
            for msg in &apply_out.messages {
                println!("{msg}");
            }
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
