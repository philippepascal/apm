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
    pub epic_submit_hints: Vec<(String, String)>,
    pub epic_close_hints: Vec<(String, String)>,
}

pub struct ApplyOutput {
    pub closed: Vec<String>,
    pub closed_branches: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub messages: Vec<String>,
}

pub fn detect(root: &Path, config: &Config) -> Result<Candidates> {
    let branches = git::ticket_branches(root)?;
    let merged = git::merged_into_main(root, &config.project.default_branch)?;
    let mut merged_set: HashSet<String> = merged.into_iter().collect();

    let terminal = config.terminal_state_ids();
    let impl_states = config.implementation_state_ids();
    let eligible = |t: &Ticket| -> bool {
        impl_states.contains(t.frontmatter.state.as_str())
            || crate::ticket_fmt::history_target_states(&t.body)
                .iter().any(|s| impl_states.contains(s.as_str()))
    };

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
    let mut epic_submit_hints: Vec<(String, String)> = Vec::new();
    let mut epic_close_hints: Vec<(String, String)> = Vec::new();

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
        let state = t.frontmatter.state.as_str();
        if terminal.contains(state) || !eligible(&t) { continue; }
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
            let state = t.frontmatter.state.as_str();
            if !terminal.contains(state) && eligible(&t) {
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
        let state = t.frontmatter.state.as_str();
        if eligible(&t) && !terminal.contains(state) {
            let branch = t.frontmatter.branch.as_deref().unwrap_or("");
            if !branch.is_empty() && !branch_set.contains(branch) {
                close.push(CloseCandidate { ticket: t, reason: "implemented, branch gone" });
            }
        }
    }

    // Case 4: implemented tickets merged into their target_branch.
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
        let state = t.frontmatter.state.as_str();
        if !eligible(&t) || terminal.contains(state) { continue; }
        let target = match t.frontmatter.target_branch.as_deref() {
            Some(tb) if !tb.is_empty() => tb.to_string(),
            _ => continue,
        };
        if git::is_branch_merged_into(root, branch, &target)? {
            merged_set.insert(branch.clone());
            close.push(CloseCandidate { ticket: t, reason: "branch merged into target" });
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
        let state = t.frontmatter.state.as_str();
        if eligible(&t) && !terminal.contains(state) {
            let id = &t.frontmatter.id;
            let target = t.frontmatter.target_branch.as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(default_branch);
            hints.push(format!(
                "ticket #{id} is in `implemented` state but its branch was not detected as merged into \
                 {target}. If it was already merged, close it manually: apm state {id} closed"
            ));
        }
    }

    // Epic detection pass: scan local epic branches for submit/close hints.
    let epic_branches = crate::epic::epic_branches(root).unwrap_or_default();
    if !epic_branches.is_empty() {
        let all_tickets = crate::ticket::load_all_from_git(root, &config.tickets.dir)
            .unwrap_or_default();
        for branch in &epic_branches {
            let id = crate::epic::epic_id_from_branch(branch);
            let title = crate::epic::branch_to_title(branch);
            let epic_tickets: Vec<_> = all_tickets
                .iter()
                .filter(|t| t.frontmatter.epic.as_deref() == Some(id))
                .collect();
            let state_cfgs: Vec<&crate::config::StateConfig> = epic_tickets
                .iter()
                .filter_map(|t| config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
                .collect();
            let derived = crate::epic::derive_epic_state(&state_cfgs);
            // An epic branch with no commits beyond its merge-base with main was never
            // developed; is_ancestor returns true for such branches (their tip is literally
            // reachable from main), producing false positives in epic_close_hints.
            let has_own_commits = git::run(root, &["merge-base", &main_ref, branch])
                .ok()
                .and_then(|base| {
                    git::run(root, &["rev-list", "--count", &format!("{base}..{branch}")]).ok()
                })
                .and_then(|s| s.trim().parse::<usize>().ok())
                .map(|n| n > 0)
                .unwrap_or(false);

            let is_merged = has_own_commits
                && git::is_branch_content_merged(root, default_branch, branch).unwrap_or(false);
            if is_merged {
                epic_close_hints.push((id.to_string(), title));
            } else if derived == "done" {
                epic_submit_hints.push((id.to_string(), title));
            }
        }
    }

    Ok(Candidates { close, hints, epic_submit_hints, epic_close_hints })
}

pub fn apply(root: &Path, config: &Config, candidates: &Candidates, author: &str, aggressive: bool) -> Result<ApplyOutput> {
    let mut closed = Vec::new();
    let mut closed_branches = Vec::new();
    let mut failed = Vec::new();
    let mut messages = Vec::new();
    for c in &candidates.close {
        let id = c.ticket.frontmatter.id.clone();
        match crate::ticket::close(root, config, &id, None, author, aggressive) {
            Ok(msgs) => {
                let branch = c.ticket.frontmatter.branch.clone()
                    .or_else(|| crate::ticket_fmt::branch_name_from_path(&c.ticket.path))
                    .unwrap_or_else(|| format!("ticket/{}", c.ticket.frontmatter.id));
                closed.push(id);
                closed_branches.push(branch);
                messages.extend(msgs);
            }
            Err(e) => {
                failed.push((id, format!("{e:#}")));
            }
        }
    }
    Ok(ApplyOutput { closed, closed_branches, failed, messages })
}
