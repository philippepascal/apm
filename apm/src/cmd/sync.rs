use anyhow::Result;
use apm_core::{config::Config, git, ticket::Ticket};
use std::io::{self, BufRead, Write};
use std::path::Path;

struct CloseCandidate {
    ticket: Ticket,
    rel_path: String,
    reason: &'static str,
}

pub fn run(root: &Path, offline: bool, quiet: bool, no_aggressive: bool, auto_close: bool) -> Result<()> {
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

    // Detect tickets ready to close and batch-commit them to main.
    let branch_set: std::collections::HashSet<&str> = branches.iter().map(|s| s.as_str()).collect();
    let candidates = detect_closeable(root, &config, &branches, &branch_set)?;
    if !candidates.is_empty() {
        let confirmed = auto_close || (!quiet && prompt_close(&candidates)?);
        if confirmed {
            batch_close(root, &config, candidates, quiet)?;
        }
    }

    Ok(())
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
            candidates.push(CloseCandidate { ticket: t, rel_path, reason: "accepted" });
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
                candidates.push(CloseCandidate { ticket: t, rel_path, reason: "implemented, branch gone" });
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

fn batch_close(root: &Path, config: &Config, candidates: Vec<CloseCandidate>, quiet: bool) -> Result<()> {
    let now = chrono::Utc::now();
    let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
    let mut files: Vec<(String, String)> = Vec::new();
    let mut ids: Vec<u32> = Vec::new();

    for c in candidates {
        let mut t = c.ticket;
        let prev = t.frontmatter.state.clone();
        t.frontmatter.state = "closed".into();
        t.frontmatter.updated_at = Some(now);
        crate::cmd::state::append_history(&mut t.body, &prev, "closed", &when, "apm-sync");
        match t.serialize() {
            Ok(content) => {
                ids.push(t.frontmatter.id);
                files.push((c.rel_path, content));
            }
            Err(e) => eprintln!("warning: ticket({}) serialize: {e:#}", t.frontmatter.id),
        }
    }

    if files.is_empty() { return Ok(()); }

    let file_refs: Vec<(&str, String)> = files.iter().map(|(p, c)| (p.as_str(), c.clone())).collect();
    let ids_str = ids.iter().map(|id| format!("#{id}")).collect::<Vec<_>>().join(", ");
    let message = format!("apm sync: close tickets {ids_str}");

    git::commit_files_to_branch(root, &config.project.default_branch, &file_refs, &message)?;
    if !quiet { println!("Closed: {ids_str}"); }
    Ok(())
}
