use anyhow::Result;
use apm_core::{config::Config, git, ticket, ticket::Ticket};
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

struct CloseCandidate {
    ticket: Ticket,
    reason: &'static str,
}

struct AcceptCandidate {
    ticket: Ticket,
}

pub fn run(root: &Path, offline: bool, quiet: bool, no_aggressive: bool, auto_close: bool, auto_accept: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    if !offline {
        match git::fetch_all(root) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("warning: fetch failed (no remote configured?): {e:#}");
            }
        }
    }

    // Detect merged branches and suggest manual transition to accepted.
    let branches = git::ticket_branches(root)?;
    let merged = git::merged_into_main(root, &config.project.default_branch)?;

    let mut accept_candidates: Vec<AcceptCandidate> = Vec::new();
    for branch in &merged {
        let suffix = branch.trim_start_matches("ticket/");
        let filename = format!("{suffix}.md");
        let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);

        let content = match git::read_from_branch(root, branch, &rel_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let dummy_path = root.join(&rel_path);
        let t = match Ticket::parse(&dummy_path, &content) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if t.frontmatter.state != "implemented" { continue; }

        if !quiet {
            println!("#{}: branch merged — run `apm state {} accepted` to accept", t.frontmatter.id, t.frontmatter.id);
        }
        accept_candidates.push(AcceptCandidate { ticket: t });
    }

    if !offline || aggressive {
        git::push_ticket_branches(root);
    }

    if !quiet {
        println!(
            "sync: {} ticket branch{} visible",
            branches.len(),
            if branches.len() == 1 { "" } else { "es" },
        );
    }

    // Prompt to accept merged tickets.
    if !accept_candidates.is_empty() {
        let confirmed = auto_accept || (!quiet && is_interactive() && prompt_accept(&accept_candidates)?);
        if confirmed {
            for c in &accept_candidates {
                super::state::run(root, &c.ticket.frontmatter.id, "accepted".into(), no_aggressive, false)?;
            }
        }
    }

    // Detect tickets ready to close and close each via the shared close logic.
    let branch_set: std::collections::HashSet<&str> = branches.iter().map(|s| s.as_str()).collect();
    let candidates = detect_closeable(root, &config, &branches, &branch_set)?;
    if !candidates.is_empty() {
        let confirmed = auto_close || (!quiet && prompt_close(&candidates)?);
        if confirmed {
            for c in candidates {
                let id = c.ticket.frontmatter.id.clone();
                if let Err(e) = ticket::close(root, &config, &id, None, "apm-sync") {
                    eprintln!("warning: could not close {id:?}: {e:#}");
                }
            }
        }
    }

    Ok(())
}

fn is_interactive() -> bool {
    io::stdout().is_terminal()
}

fn prompt_accept(candidates: &[AcceptCandidate]) -> Result<bool> {
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

fn detect_closeable(
    root: &Path,
    config: &Config,
    branches: &[String],
    branch_set: &std::collections::HashSet<&str>,
) -> Result<Vec<CloseCandidate>> {
    let mut candidates = Vec::new();

    // Case 1: tickets in `accepted` state on any ticket branch.
    for branch in branches {
        let suffix = branch.trim_start_matches("ticket/");
        let rel_path = format!("{}/{suffix}.md", config.tickets.dir.to_string_lossy());
        let content = match git::read_from_branch(root, branch, &rel_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let t = match Ticket::parse(&root.join(&rel_path), &content) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if t.frontmatter.state == "accepted" {
            candidates.push(CloseCandidate { ticket: t, reason: "accepted" });
        }
    }

    // Case 2: tickets on main in `implemented` state with no surviving branch.
    let default_branch = &config.project.default_branch;
    let ticket_files = git::list_files_on_branch(root, default_branch, &config.tickets.dir.to_string_lossy()).unwrap_or_default();
    for rel_path in ticket_files {
        if !rel_path.ends_with(".md") { continue; }
        let content = match git::read_from_branch(root, default_branch, &rel_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let t = match Ticket::parse(&root.join(&rel_path), &content) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if t.frontmatter.state == "implemented" {
            let branch = t.frontmatter.branch.as_deref().unwrap_or("");
            if !branch.is_empty() && !branch_set.contains(branch) {
                candidates.push(CloseCandidate { ticket: t, reason: "implemented, branch gone" });
            }
        }
    }

    Ok(candidates)
}

fn prompt_close(candidates: &[CloseCandidate]) -> Result<bool> {
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

