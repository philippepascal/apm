use std::path::Path;
use crate::config::Config;

const DEP_COMMIT_CAP: usize = 20;

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
    out.push_str("# Dependency Context Bundle\n\n");

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

    out.push_str("***\n");
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

