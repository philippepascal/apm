use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::StateConfig;

/// Derive the display state of an epic from the `StateConfig`s of its tickets.
///
/// Rules (evaluated in order):
/// 1. Empty slice → "empty"
/// 2. Any state has `actionable` containing "agent" → "active"
/// 3. All states have `terminal = true` → "done"
/// 4. All states have `satisfies_deps = true` or `terminal = true`, and at
///    least one has `satisfies_deps = true` → "complete"
/// 5. Otherwise → "active"
pub fn derive_epic_state(states: &[&StateConfig]) -> &'static str {
    if states.is_empty() {
        return "empty";
    }
    if states.iter().any(|s| s.actionable.iter().any(|a| a == "agent")) {
        return "active";
    }
    if states.iter().all(|s| s.terminal) {
        return "done";
    }
    if states.iter().all(|s| s.satisfies_deps || s.terminal)
        && states.iter().any(|s| s.satisfies_deps)
    {
        return "complete";
    }
    "active"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StateConfig;

    fn make_state(terminal: bool, satisfies_deps: bool, actionable: Vec<&str>) -> StateConfig {
        StateConfig {
            id: "x".to_string(),
            label: "x".to_string(),
            description: String::new(),
            terminal,
            satisfies_deps,
            transitions: vec![],
            actionable: actionable.into_iter().map(|s| s.to_string()).collect(),
            instructions: None,
        }
    }

    #[test]
    fn empty_slice_is_empty() {
        assert_eq!(derive_epic_state(&[]), "empty");
    }

    #[test]
    fn all_terminal_is_done() {
        let a = make_state(true, false, vec![]);
        let b = make_state(true, false, vec![]);
        assert_eq!(derive_epic_state(&[&a, &b]), "done");
    }

    #[test]
    fn all_satisfies_deps_or_terminal_with_at_least_one_satisfies_is_complete() {
        let a = make_state(false, true, vec![]);
        let b = make_state(true, false, vec![]);
        assert_eq!(derive_epic_state(&[&a, &b]), "complete");
    }

    #[test]
    fn any_agent_actionable_is_active() {
        let a = make_state(false, false, vec!["agent"]);
        let b = make_state(true, false, vec![]);
        assert_eq!(derive_epic_state(&[&a, &b]), "active");
    }

    #[test]
    fn mixed_non_terminal_non_satisfies_is_active() {
        let a = make_state(false, false, vec!["supervisor"]);
        let b = make_state(true, false, vec![]);
        assert_eq!(derive_epic_state(&[&a, &b]), "active");
    }

    #[test]
    fn agent_actionable_checked_before_terminal() {
        // Even if all are terminal, if any is agent-actionable → active
        let a = make_state(true, false, vec!["agent"]);
        assert_eq!(derive_epic_state(&[&a]), "active");
    }
}

pub fn create(root: &Path, title: &str) -> Result<String> {
    let id = crate::git::gen_hex_id();
    let slug = crate::ticket::slugify(title);
    let branch = format!("epic/{id}-{slug}");

    // Fetch origin/main; propagate error if it doesn't exist.
    let fetch_out = Command::new("git")
        .current_dir(root)
        .args(["fetch", "origin", "main"])
        .output()
        .map_err(|e| anyhow::anyhow!("git not found: {e}"))?;
    if !fetch_out.status.success() {
        anyhow::bail!(
            "{}",
            String::from_utf8_lossy(&fetch_out.stderr).trim()
        );
    }

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let wt_path = std::env::temp_dir().join(format!(
        "apm-{}-{}-{}",
        std::process::id(),
        unique,
        branch.replace('/', "-"),
    ));

    let add_out = Command::new("git")
        .current_dir(root)
        .args([
            "worktree",
            "add",
            "-b",
            &branch,
            &wt_path.to_string_lossy(),
            "origin/main",
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("git not found: {e}"))?;
    if !add_out.status.success() {
        anyhow::bail!(
            "{}",
            String::from_utf8_lossy(&add_out.stderr).trim()
        );
    }

    let result = (|| -> Result<()> {
        let epic_md = wt_path.join("EPIC.md");
        std::fs::write(&epic_md, format!("# {title}\n"))?;

        let stage_out = Command::new("git")
            .current_dir(&wt_path)
            .args(["add", "EPIC.md"])
            .output()?;
        if !stage_out.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&stage_out.stderr).trim());
        }

        let commit_msg = format!("epic({id}): create {title}");
        let commit_out = Command::new("git")
            .current_dir(&wt_path)
            .args(["commit", "-m", &commit_msg])
            .output()?;
        if !commit_out.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&commit_out.stderr).trim());
        }
        Ok(())
    })();

    let _ = Command::new("git")
        .current_dir(root)
        .args(["worktree", "remove", "--force", &wt_path.to_string_lossy()])
        .output();
    let _ = std::fs::remove_dir_all(&wt_path);

    result?;

    crate::git::push_branch_tracking(root, &branch)?;

    Ok(branch)
}
