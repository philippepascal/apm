use std::path::Path;
use crate::config::Config;
use crate::ticket::Ticket;

const DEP_COMMIT_CAP: usize = 20;

const SCOPE_GUIDANCE: &str =
    "Use this to scope your ticket — do not duplicate or overreach into sibling tickets' territory.";

/// Build an epic context bundle for a spec worker prompt.
///
/// Returns a Markdown string prepended to the worker prompt when the ticket
/// belongs to an epic.  Returns an empty string only when the epic branch or
/// EPIC.md cannot be found (so callers can always prepend safely).
pub fn build_epic_bundle(
    root: &Path,
    epic_id: &str,
    current_ticket_id: &str,
    config: &Config,
) -> String {
    let epic_md = crate::epic::find_epic_branch(root, epic_id)
        .and_then(|branch| crate::git::read_from_branch(root, &branch, "EPIC.md").ok())
        .unwrap_or_default();

    let (epic_title, epic_body) = parse_epic_md(&epic_md, epic_id);

    let all_tickets = crate::ticket::load_all_from_git(root, &config.tickets.dir)
        .unwrap_or_default();

    let siblings: Vec<&Ticket> = all_tickets.iter()
        .filter(|t| {
            t.frontmatter.epic.as_deref() == Some(epic_id)
                && t.frontmatter.id != current_ticket_id
        })
        .collect();

    let terminal_ids = config.terminal_state_ids();

    let mut active: Vec<&Ticket> = siblings.iter()
        .filter(|t| !terminal_ids.contains(&t.frontmatter.state))
        .copied()
        .collect();
    let mut closed: Vec<&Ticket> = siblings.iter()
        .filter(|t| terminal_ids.contains(&t.frontmatter.state))
        .copied()
        .collect();

    // Active siblings sorted by state then id for deterministic grouping.
    active.sort_by(|a, b| {
        a.frontmatter.state.cmp(&b.frontmatter.state)
            .then(a.frontmatter.id.cmp(&b.frontmatter.id))
    });
    // Closed siblings: newest first so the most-recent ones are retained when capped.
    closed.sort_by(|a, b| b.frontmatter.created_at.cmp(&a.frontmatter.created_at));

    let sibling_cap = config.context.epic_sibling_cap;
    let byte_cap = config.context.epic_byte_cap;

    let active_take = active.len().min(sibling_cap);
    let included_active = &active[..active_take];
    let remaining = sibling_cap.saturating_sub(active_take);
    let closed_take = closed.len().min(remaining);
    let included_closed = &closed[..closed_take];
    let elided_count = closed.len().saturating_sub(closed_take);

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("**Epic Context Bundle**\n\n");
    out.push_str(&format!("**Epic:** {}\n", epic_title));
    if !epic_body.is_empty() {
        out.push('\n');
        out.push_str(&epic_body);
        out.push('\n');
    }
    out.push('\n');
    out.push_str(&format!("**Scope guidance:** {SCOPE_GUIDANCE}\n"));

    if !included_active.is_empty() || !included_closed.is_empty() || elided_count > 0 {
        out.push_str("\n### Sibling Tickets\n");

        let mut seen_states: Vec<&str> = Vec::new();
        for t in included_active {
            let state = t.frontmatter.state.as_str();
            if !seen_states.contains(&state) {
                seen_states.push(state);
                out.push_str(&format!("\n#### {}\n", state));
            }
            append_sibling_entry(&mut out, t);
        }

        if !included_closed.is_empty() {
            let mut seen_closed: Vec<&str> = Vec::new();
            for t in included_closed {
                let state = t.frontmatter.state.as_str();
                if !seen_closed.contains(&state) {
                    seen_closed.push(state);
                    out.push_str(&format!("\n#### {}\n", state));
                }
                append_sibling_entry(&mut out, t);
            }
        }

        if elided_count > 0 {
            let plural = if elided_count == 1 { "" } else { "s" };
            out.push_str(&format!(
                "\n*({elided_count} older closed sibling{plural} not shown)*\n"
            ));
        }
    }

    out.push_str("---\n");

    // Apply byte cap: truncate at a safe character boundary.
    if byte_cap > 0 && out.len() > byte_cap {
        let truncate_at = (0..=byte_cap)
            .rev()
            .find(|&i| out.is_char_boundary(i))
            .unwrap_or(0);
        let mut truncated = out[..truncate_at].to_string();
        truncated.push_str("\n*[bundle truncated at byte limit]*\n---\n");
        truncated
    } else {
        out
    }
}

fn parse_epic_md(content: &str, fallback_id: &str) -> (String, String) {
    let mut title = fallback_id.to_string();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut found_title = false;

    for line in content.lines() {
        if !found_title {
            if let Some(t) = line.strip_prefix("# ") {
                title = t.trim().to_string();
                found_title = true;
            }
        } else {
            body_lines.push(line);
        }
    }

    // Trim leading and trailing blank lines from body.
    while body_lines.first().map(|l| l.trim().is_empty()) == Some(true) {
        body_lines.remove(0);
    }
    while body_lines.last().map(|l| l.trim().is_empty()) == Some(true) {
        body_lines.pop();
    }

    (title, body_lines.join("\n"))
}

fn append_sibling_entry(out: &mut String, t: &Ticket) {
    out.push_str(&format!("- **{}:** {}\n", t.frontmatter.id, t.frontmatter.title));

    let doc = match t.document() {
        Ok(d) => d,
        Err(_) => return,
    };

    // One-line Problem summary.
    let problem = crate::spec::get_section(&doc, "Problem").unwrap_or_default();
    if let Some(one_liner) = problem.lines().find(|l| !l.trim().is_empty()) {
        out.push_str(&format!("  *Problem:* {}\n", one_liner.trim()));
    }

    // Full "Out of scope" section if present.
    if let Some(oos) = crate::spec::get_section(&doc, "Out of scope").filter(|s| !s.is_empty()) {
        out.push_str("  *Out of scope:*\n");
        for line in oos.lines() {
            if line.trim().is_empty() {
                out.push_str("  \n");
            } else {
                out.push_str(&format!("  {}\n", line));
            }
        }
    }
}

/// Build a dependency context bundle for a worker prompt.
///
/// Returns a Markdown string to prepend to the worker prompt when the ticket
/// has `depends_on` set.  Returns an empty string when `depends_on` is empty.
///
/// Direct dependencies include: ticket id + title, full Approach section, and
/// a capped commit-subject list.  If a dependency is not yet in a terminal
/// state, a warning is appended.
///
/// Transitive dependencies (deps-of-deps, one level deep) include only
/// title + one-line Problem summary.
pub fn build_dependency_bundle(root: &Path, depends_on: &[String], config: &Config) -> String {
    if depends_on.is_empty() {
        return String::new();
    }

    let all_tickets = crate::ticket::load_all_from_git(root, &config.tickets.dir)
        .unwrap_or_default();

    let terminal_ids = config.terminal_state_ids();
    let default_branch = config.project.default_branch.clone();

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("**Dependency Context Bundle**\n\n");

    let mut any = false;

    for dep_id in depends_on {
        let Some(dep) = all_tickets.iter().find(|t| &t.frontmatter.id == dep_id) else {
            out.push_str(&format!("### Dependency: {dep_id}\n*Ticket not found.*\n\n"));
            any = true;
            continue;
        };

        any = true;
        let is_terminal = terminal_ids.contains(&dep.frontmatter.state);

        out.push_str(&format!(
            "### Dependency: {} — {}\n",
            dep_id, dep.frontmatter.title
        ));
        out.push_str(&format!("**State:** {}", dep.frontmatter.state));
        if !is_terminal {
            out.push_str(
                " ⚠️ *This dependency is not yet closed — its API may still change. Tread carefully.*",
            );
        }
        out.push('\n');

        // Full Approach section.
        if let Ok(doc) = dep.document() {
            if let Some(approach) = crate::spec::get_section(&doc, "Approach")
                .filter(|s| !s.is_empty())
            {
                out.push_str("\n**Approach:**\n");
                out.push_str(&approach);
                out.push('\n');
            }
        }

        // Commit subjects landed on the dep branch.
        let dep_branch = dep
            .frontmatter
            .branch
            .clone()
            .or_else(|| crate::ticket_fmt::branch_name_from_path(&dep.path))
            .unwrap_or_else(|| format!("ticket/{dep_id}"));
        let target = dep
            .frontmatter
            .target_branch
            .as_deref()
            .unwrap_or(default_branch.as_str());
        let subjects = commit_subjects(root, target, &dep_branch, DEP_COMMIT_CAP);
        if !subjects.is_empty() {
            out.push_str("\n**Commits landed:**\n");
            for s in &subjects {
                out.push_str(&format!("- {s}\n"));
            }
        }

        // Transitive dependencies — one level deep, title + one-line Problem.
        if let Some(ref trans_ids) = dep.frontmatter.depends_on {
            if !trans_ids.is_empty() {
                out.push_str("\n**Transitive dependencies:**\n");
                for trans_id in trans_ids {
                    if let Some(trans) = all_tickets.iter().find(|t| &t.frontmatter.id == trans_id) {
                        out.push_str(&format!("- **{}:** {}", trans_id, trans.frontmatter.title));
                        if let Ok(doc) = trans.document() {
                            if let Some(problem) = crate::spec::get_section(&doc, "Problem") {
                                if let Some(line) =
                                    problem.lines().find(|l| !l.trim().is_empty())
                                {
                                    out.push_str(&format!(" — {}", line.trim()));
                                }
                            }
                        }
                        out.push('\n');
                    } else {
                        out.push_str(&format!("- **{trans_id}:** *(not found)*\n"));
                    }
                }
            }
        }

        out.push('\n');
    }

    if !any {
        return String::new();
    }

    out.push_str("---\n");
    out
}

/// Return commit subjects on `dep_branch` not reachable from `target`.
/// Caps at `max_count`.  Returns an empty vec on any git error.
fn commit_subjects(root: &Path, target: &str, dep_branch: &str, max_count: usize) -> Vec<String> {
    let dep_ref = if crate::git_util::remote_branch_tip(root, dep_branch).is_some() {
        format!("origin/{dep_branch}")
    } else {
        dep_branch.to_string()
    };
    let target_ref = if crate::git_util::remote_branch_tip(root, target).is_some() {
        format!("origin/{target}")
    } else {
        target.to_string()
    };
    let range = format!("{target_ref}..{dep_ref}");
    let max_str = max_count.to_string();
    let output = crate::git_util::run(
        root,
        &["log", "--pretty=%s", &range, &format!("--max-count={max_str}")],
    )
    .unwrap_or_default();
    output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_epic_md_extracts_title_and_body() {
        let md = "# My Epic\n\n## Goal\nDo great things.\n\n## Non-goals\nNot everything.\n";
        let (title, body) = parse_epic_md(md, "fallback");
        assert_eq!(title, "My Epic");
        assert!(body.contains("Goal"));
        assert!(body.contains("Do great things"));
        assert!(body.contains("Non-goals"));
    }

    #[test]
    fn parse_epic_md_fallback_when_no_heading() {
        let md = "Just some text without a heading.";
        let (title, body) = parse_epic_md(md, "epic-id");
        assert_eq!(title, "epic-id");
        assert!(body.is_empty());
    }

    #[test]
    fn parse_epic_md_trims_leading_blank_lines_from_body() {
        let md = "# Title\n\n\nFirst non-blank line.\n";
        let (_, body) = parse_epic_md(md, "id");
        assert!(!body.starts_with('\n'));
        assert!(body.starts_with("First"));
    }

    #[test]
    fn build_epic_bundle_returns_empty_string_when_no_epic_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path();
        std::process::Command::new("git")
            .args(["-c", "init.defaultBranch=main", "init", "-q"])
            .current_dir(p)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t.com")
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(p)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "t"])
            .current_dir(p)
            .status()
            .unwrap();
        std::fs::write(p.join("README.md"), "init").unwrap();
        std::process::Command::new("git")
            .args(["add", "README.md"])
            .current_dir(p)
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["-c", "commit.gpgsign=false", "commit", "-m", "init"])
            .current_dir(p)
            .status()
            .unwrap();
        std::fs::write(
            p.join("apm.toml"),
            "[project]\nname = \"test\"\n\n[tickets]\ndir = \"tickets\"\n",
        )
        .unwrap();
        let config = crate::config::Config::load(p).unwrap();
        // No epic branch exists → bundle is just the header/footer with minimal content
        // (epic_id used as fallback title)
        let bundle = build_epic_bundle(p, "deadbeef", "aabb1234", &config);
        assert!(bundle.contains("deadbeef"), "fallback title should appear");
        assert!(bundle.contains("Scope guidance"), "guidance should always appear");
    }
}
