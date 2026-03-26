use anyhow::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(root: &Path) -> Result<()> {
    let tickets_dir = root.join("tickets");
    if !tickets_dir.exists() {
        std::fs::create_dir_all(&tickets_dir)?;
        println!("Created tickets/");
    }
    let config_path = root.join("apm.toml");
    if !config_path.exists() {
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");
        std::fs::write(&config_path, default_config(name))?;
        println!("Created apm.toml");
    }
    let agents_path = root.join("apm.agents.md");
    if !agents_path.exists() {
        std::fs::write(&agents_path, default_agents_md())?;
        println!("Created apm.agents.md");
    }
    ensure_claude_md(root)?;
    let gitignore = root.join(".gitignore");
    ensure_gitignore(&gitignore)?;
    let git_dir = root.join(".git");
    write_hooks(&git_dir)?;
    maybe_initial_commit(root)?;
    maybe_create_meta_branch(root)?;
    println!("apm initialized.");
    Ok(())
}

fn ensure_claude_md(root: &Path) -> Result<()> {
    let import_line = "@apm.agents.md";
    let claude_path = root.join("CLAUDE.md");
    if claude_path.exists() {
        let contents = std::fs::read_to_string(&claude_path)?;
        if contents.contains(import_line) {
            return Ok(());
        }
        std::fs::write(&claude_path, format!("{import_line}\n\n{contents}"))?;
        println!("Updated CLAUDE.md (added @apm.agents.md import).");
    } else {
        std::fs::write(&claude_path, format!("{import_line}\n"))?;
        println!("Created CLAUDE.md.");
    }
    Ok(())
}

fn default_agents_md() -> &'static str {
    include_str!("../../../apm.agents.md")
}

fn default_config(name: &str) -> String {
    format!(
        r##"[project]
name = "{name}"

[tickets]
dir = "tickets"

[agents]
max_concurrent = 3
actionable_states = ["new", "ammend", "ready"]
instructions = "apm.agents.md"

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id    = "new"
label = "New"
color = "#6b7280"

[[workflow.states]]
id    = "question"
label = "Question"
color = "#f59e0b"

[[workflow.states]]
id    = "specd"
label = "Specd"
color = "#3b82f6"

[[workflow.states]]
id    = "ammend"
label = "Ammend"
color = "#ef4444"

[[workflow.states]]
id    = "ready"
label = "Ready"
color = "#10b981"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"
color = "#8b5cf6"

[[workflow.states]]
id    = "implemented"
label = "Implemented"
color = "#06b6d4"

[[workflow.states]]
id    = "accepted"
label = "Accepted"
color = "#84cc16"

[[workflow.states]]
id       = "closed"
label    = "Closed"
color    = "#374151"
terminal = true
"##
    )
}

fn ensure_gitignore(path: &PathBuf) -> Result<()> {
    let entry = "tickets/NEXT_ID\n";
    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        if !contents.contains("tickets/NEXT_ID") {
            let mut updated = contents;
            if !updated.ends_with('\n') {
                updated.push('\n');
            }
            updated.push_str(entry);
            std::fs::write(path, updated)?;
            println!("Updated .gitignore");
        }
    } else {
        std::fs::write(path, entry)?;
        println!("Created .gitignore");
    }
    Ok(())
}

fn write_hooks(git_dir: &PathBuf) -> Result<()> {
    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    let pre_push = hooks_dir.join("pre-push");
    std::fs::write(
        &pre_push,
        "#!/bin/sh\n# Fires event:branch_push_first on first push of ticket/<id>-* in ready state\ncommand -v apm >/dev/null 2>&1 && apm _hook pre-push \"$@\" || true\n",
    )?;
    std::fs::set_permissions(&pre_push, std::fs::Permissions::from_mode(0o755))?;

    let post_merge = hooks_dir.join("post-merge");
    std::fs::write(
        &post_merge,
        "#!/bin/sh\ncommand -v apm >/dev/null 2>&1 && apm sync --quiet --offline || true\n",
    )?;
    std::fs::set_permissions(&post_merge, std::fs::Permissions::from_mode(0o755))?;

    println!("Installed git hooks (pre-push, post-merge).");
    Ok(())
}

/// Make an initial commit if the repo has no commits yet.
/// This is required for git worktree support.
fn maybe_initial_commit(root: &Path) -> Result<()> {
    let has_commits = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_commits {
        return Ok(());
    }

    // Stage apm.toml and .gitignore.
    Command::new("git")
        .args(["add", "apm.toml", ".gitignore"])
        .current_dir(root)
        .status()?;

    let out = Command::new("git")
        .args(["commit", "-m", "apm: initialize project"])
        .current_dir(root)
        .output()?;

    if out.status.success() {
        println!("Created initial commit.");
    }
    Ok(())
}

/// Create the apm/meta branch with NEXT_ID = 1 if it doesn't exist yet.
fn maybe_create_meta_branch(root: &Path) -> Result<()> {
    // Only attempt if the repo has commits.
    let has_commits = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_commits {
        return Ok(());
    }

    let meta_exists = Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/apm/meta"])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if meta_exists {
        return Ok(());
    }

    apm_core::git::init_meta_branch(root);
    Ok(())
}
