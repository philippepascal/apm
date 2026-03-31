use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use crate::{config::Config, git, ticket::Ticket};

pub struct AcceptCandidate {
    pub ticket: Ticket,
}

pub struct CloseCandidate {
    pub ticket: Ticket,
    pub reason: &'static str,
}

pub struct Candidates {
    pub accept: Vec<AcceptCandidate>,
    pub close: Vec<CloseCandidate>,
}

pub fn detect(root: &Path, config: &Config) -> Result<Candidates> {
    let branches = git::ticket_branches(root)?;
    let merged = git::merged_into_main(root, &config.project.default_branch)?;

    let mut accept = Vec::new();
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
        accept.push(AcceptCandidate { ticket: t });
    }

    let branch_set: HashSet<&str> = branches.iter().map(|s| s.as_str()).collect();
    let close = detect_closeable(root, config, &branches, &branch_set)?;

    Ok(Candidates { accept, close })
}

fn detect_closeable(
    root: &Path,
    config: &Config,
    branches: &[String],
    branch_set: &HashSet<&str>,
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

pub fn apply(root: &Path, config: &Config, candidates: &Candidates, author: &str, aggressive: bool) -> Result<()> {
    for c in &candidates.accept {
        crate::ticket::accept(root, config, &c.ticket.frontmatter.id, author, aggressive)?;
    }
    for c in &candidates.close {
        let id = c.ticket.frontmatter.id.clone();
        if let Err(e) = crate::ticket::close(root, config, &id, None, author, aggressive) {
            eprintln!("warning: could not close {id:?}: {e:#}");
        }
    }
    Ok(())
}
