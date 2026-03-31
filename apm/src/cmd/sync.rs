use anyhow::Result;
use apm_core::{config::Config, git, sync::{self, Candidates}};
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

pub fn run(root: &Path, offline: bool, quiet: bool, no_aggressive: bool, auto_close: bool, auto_accept: bool) -> Result<()> {
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

    for c in &candidates.accept {
        if !quiet {
            println!("#{}: branch merged — run `apm state {} accepted` to accept", c.ticket.frontmatter.id, c.ticket.frontmatter.id);
        }
    }

    if !offline || aggressive {
        git::push_ticket_branches(root);
    }

    let branches = git::ticket_branches(root)?;
    if !quiet {
        println!(
            "sync: {} ticket branch{} visible",
            branches.len(),
            if branches.len() == 1 { "" } else { "es" },
        );
    }

    let Candidates { accept: accept_cands, close: close_cands } = candidates;

    if !accept_cands.is_empty() {
        let confirmed = auto_accept || (!quiet && is_interactive() && prompt_accept(&accept_cands)?);
        if confirmed {
            sync::apply(root, &config, &Candidates { accept: accept_cands, close: vec![] }, "apm-sync")?;
        }
    }

    if !close_cands.is_empty() {
        let confirmed = auto_close || (!quiet && prompt_close(&close_cands)?);
        if confirmed {
            sync::apply(root, &config, &Candidates { accept: vec![], close: close_cands }, "apm-sync")?;
        }
    }

    Ok(())
}

fn is_interactive() -> bool {
    io::stdout().is_terminal()
}

fn prompt_accept(candidates: &[sync::AcceptCandidate]) -> Result<bool> {
    println!("\nTickets ready to accept:");
    for c in candidates {
        println!("  #{}  {}", c.ticket.frontmatter.id, c.ticket.frontmatter.title);
    }
    print!("\nAccept all? [y/N] ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().eq_ignore_ascii_case("y"))
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
