use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use crate::{config::Config, git, ticket::Ticket};

pub struct CloseCandidate {
    pub ticket: Ticket,
    pub reason: &'static str,
}

pub struct Candidates {
    pub close: Vec<CloseCandidate>,
    pub hints: Vec<String>,
}

pub struct ApplyOutput {
    pub closed: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub messages: Vec<String>,
}

pub fn detect(root: &Path, config: &Config) -> Result<Candidates> {
    let branches = git::ticket_branches(root)?;
    let merged = git::merged_into_main(root, &config.project.default_branch)?;
    let mut merged_set: HashSet<String> = merged.into_iter().collect();

    let terminal = config.terminal_state_ids();

    let branch_set: HashSet<&str> = branches.iter().map(|s| s.as_str()).collect();

    let default_branch = &config.project.default_branch;
    let tickets_dir = config.tickets.dir.to_string_lossy().to_string();

    // Mirrors `merged_into_main`'s own preference: prefer origin/<default> when available.
    let remote_ref = format!("refs/remotes/origin/{default_branch}");
    let main_ref = if git::run(root, &["rev-parse", "--verify", &remote_ref]).is_ok() {
        format!("origin/{default_branch}")
    } else {
        default_branch.clone()
    };

    let mut close = Vec::new();
    let mut hints = Vec::new();

    // Case 1: non-terminal tickets on merged branches.
    for branch in &branches {
        if !merged_set.contains(branch.as_str()) { continue; }
        let suffix = branch.trim_start_matches("ticket/");
        let rel_path = format!("{tickets_dir}/{suffix}.md");
        let content = match git::read_from_branch(root, branch, &rel_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let t = match Ticket::parse(&root.join(&rel_path), &content) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if terminal.contains(t.frontmatter.state.as_str()) { continue; }
        close.push(CloseCandidate { ticket: t, reason: "branch merged" });
    }

    // Case 3: tickets whose branch tip has only state-transition commits after the merge.
    // Walk from the tip skipping ticket-file-only commits; squash-check the last real commit.
    for branch in &branches {
        if merged_set.contains(branch.as_str()) { continue; }
        if git::content_merged_into_main(root, &main_ref, branch, &tickets_dir)? {
            let suffix = branch.trim_start_matches("ticket/");
            let rel_path = format!("{tickets_dir}/{suffix}.md");
            let content = match git::read_from_branch(root, branch, &rel_path) {
                Ok(c) => c,
                Err(_) => {
                    merged_set.insert(branch.clone());
                    continue;
                }
            };
            let t = match Ticket::parse(&root.join(&rel_path), &content) {
                Ok(t) => t,
                Err(_) => {
                    merged_set.insert(branch.clone());
                    continue;
                }
            };
            merged_set.insert(branch.clone());
            if !terminal.contains(t.frontmatter.state.as_str()) {
                close.push(CloseCandidate { ticket: t, reason: "branch content merged" });
            }
        }
    }

    // Case 2: tickets on main in `implemented` state with no surviving branch.
    let ticket_files = git::list_files_on_branch(root, default_branch, &tickets_dir).unwrap_or_default();
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
                close.push(CloseCandidate { ticket: t, reason: "implemented, branch gone" });
            }
        }
    }

    // Hint generation: implemented tickets whose branch was not detected by any pass.
    for branch in &branches {
        if merged_set.contains(branch.as_str()) { continue; }
        let suffix = branch.trim_start_matches("ticket/");
        let rel_path = format!("{tickets_dir}/{suffix}.md");
        let content = match git::read_from_branch(root, branch, &rel_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let t = match Ticket::parse(&root.join(&rel_path), &content) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if t.frontmatter.state == "implemented" {
            let id = &t.frontmatter.id;
            hints.push(format!(
                "ticket #{id} is in `implemented` state but its branch was not detected as merged into \
                 main. If it was already merged, close it manually: apm state {id} closed"
            ));
        }
    }

    Ok(Candidates { close, hints })
}

pub fn apply(root: &Path, config: &Config, candidates: &Candidates, author: &str, aggressive: bool) -> Result<ApplyOutput> {
    let mut closed = Vec::new();
    let mut failed = Vec::new();
    let mut messages = Vec::new();
    for c in &candidates.close {
        let id = c.ticket.frontmatter.id.clone();
        match crate::ticket::close(root, config, &id, None, author, aggressive) {
            Ok(msgs) => {
                closed.push(id);
                messages.extend(msgs);
            }
            Err(e) => {
                failed.push((id, format!("{e:#}")));
            }
        }
    }
    Ok(ApplyOutput { closed, failed, messages })
}
