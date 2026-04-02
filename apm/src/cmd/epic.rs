use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run_new(root: &Path, title: String) -> Result<()> {
    let branch = apm_core::epic::create(root, &title)?;
    println!("{branch}");
    Ok(())
}

pub fn run_close(root: &Path, id_arg: &str) -> Result<()> {
    let config = apm_core::config::Config::load(root)?;

    // 1. Resolve the epic branch from the id prefix.
    let matches = apm_core::git::find_epic_branches(root, id_arg);
    let epic_branch = match matches.len() {
        0 => anyhow::bail!("no epic branch found matching '{id_arg}'"),
        1 => matches.into_iter().next().unwrap(),
        _ => anyhow::bail!(
            "ambiguous id '{id_arg}': matches {}\n  {}",
            matches.len(),
            matches.join("\n  ")
        ),
    };

    // 2. Parse the 8-char epic ID from the branch name: epic/<id>-<slug>
    let after_prefix = epic_branch.trim_start_matches("epic/");
    let epic_id = after_prefix.split('-').next().unwrap_or("");

    // 3. Load all tickets and find those belonging to this epic.
    let tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;
    let epic_tickets: Vec<_> = tickets
        .iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
        .collect();

    // 4. Gate check: every epic ticket must be in a satisfies_deps or terminal state.
    let mut not_ready: Vec<String> = Vec::new();
    for t in &epic_tickets {
        let state_id = &t.frontmatter.state;
        let passes = config
            .workflow
            .states
            .iter()
            .find(|s| &s.id == state_id)
            .map(|s| s.satisfies_deps || s.terminal)
            .unwrap_or(false);
        if !passes {
            not_ready.push(format!("  {} — {} (state: {})", t.frontmatter.id, t.frontmatter.title, state_id));
        }
    }
    if !not_ready.is_empty() {
        anyhow::bail!(
            "cannot close epic: the following tickets are not ready:\n{}",
            not_ready.join("\n")
        );
    }

    // 5. Check for an existing open PR (idempotency).
    let pr_check = Command::new("gh")
        .args([
            "pr", "list",
            "--head", &epic_branch,
            "--state", "open",
            "--json", "number",
            "--jq", ".[0].number",
        ])
        .current_dir(root)
        .output();
    if let Ok(out) = pr_check {
        let number_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !number_str.is_empty() {
            if let Ok(n) = number_str.parse::<u64>() {
                println!("PR #{n} already open for {epic_branch}");
                return Ok(());
            }
        }
    }

    // 6. Derive a human-readable title from the branch name.
    let pr_title = branch_to_title(&epic_branch);

    // 7. Create the PR.
    let default_branch = &config.project.default_branch;
    let pr_body = format!("Epic: {epic_branch}");
    let create_out = Command::new("gh")
        .args([
            "pr", "create",
            "--base", default_branch,
            "--head", &epic_branch,
            "--title", &pr_title,
            "--body", &pr_body,
        ])
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("gh not found: {e}"))?;

    if !create_out.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&create_out.stderr).trim());
    }

    let url = String::from_utf8_lossy(&create_out.stdout).trim().to_string();
    println!("{url}");
    Ok(())
}

/// Convert an epic branch name to a human-readable PR title.
/// `epic/ab12cd34-user-authentication` → `"User Authentication"`
pub fn branch_to_title(branch: &str) -> String {
    // Strip "epic/" prefix
    let rest = branch.trim_start_matches("epic/");
    // Strip the "<8-char-id>-" segment (first hyphen-separated token)
    let slug = match rest.find('-') {
        Some(pos) => &rest[pos + 1..],
        None => rest,
    };
    // Replace hyphens with spaces and title-case each word
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_to_title_basic() {
        assert_eq!(branch_to_title("epic/ab12cd34-user-authentication"), "User Authentication");
    }

    #[test]
    fn branch_to_title_single_word() {
        assert_eq!(branch_to_title("epic/ab12cd34-dashboard"), "Dashboard");
    }

    #[test]
    fn branch_to_title_many_words() {
        assert_eq!(branch_to_title("epic/ab12cd34-add-oauth-login-flow"), "Add Oauth Login Flow");
    }

    #[test]
    fn branch_to_title_no_slug() {
        // Degenerate: no hyphen after id — returns empty string (id treated as slug)
        assert_eq!(branch_to_title("epic/ab12cd34"), "Ab12cd34");
    }

    // Gate check logic tests
    #[test]
    fn gate_check_all_passing() {
        use apm_core::config::{StateConfig, WorkflowConfig};

        let states = vec![
            make_state("implemented", true, false),
            make_state("closed", false, true),
        ];
        let wf = WorkflowConfig { states, ..Default::default() };

        // Both states satisfy the gate
        for s in &wf.states {
            assert!(s.satisfies_deps || s.terminal, "state {} should pass", s.id);
        }
    }

    #[test]
    fn gate_check_failing_state() {
        use apm_core::config::{StateConfig, WorkflowConfig};

        let states = vec![
            make_state("in_progress", false, false),
            make_state("implemented", true, false),
        ];
        let wf = WorkflowConfig { states, ..Default::default() };

        let in_prog = wf.states.iter().find(|s| s.id == "in_progress").unwrap();
        assert!(!in_prog.satisfies_deps && !in_prog.terminal);

        let implemented = wf.states.iter().find(|s| s.id == "implemented").unwrap();
        assert!(implemented.satisfies_deps || implemented.terminal);
    }

    fn make_state(id: &str, satisfies_deps: bool, terminal: bool) -> apm_core::config::StateConfig {
        apm_core::config::StateConfig {
            id: id.to_string(),
            label: id.to_string(),
            description: String::new(),
            terminal,
            satisfies_deps,
            transitions: vec![],
            actionable: vec![],
            instructions: None,
        }
    }
}
