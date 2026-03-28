use anyhow::Result;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(root: &Path, no_claude: bool) -> Result<()> {
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
        let branch = detect_default_branch(root);
        std::fs::write(&config_path, default_config(name, &branch))?;
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
    update_claude_settings(root, no_claude)?;
    maybe_initial_commit(root)?;
    maybe_create_meta_branch(root)?;
    ensure_worktrees_dir(root)?;
    update_user_claude_settings()?;
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

fn detect_default_branch(root: &Path) -> String {
    std::process::Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "main".to_string())
}

fn default_config(name: &str, default_branch: &str) -> String {
    format!(
        r##"[project]
name = "{name}"
default_branch = "{default_branch}"

[tickets]
dir = "tickets"

[worktrees]
dir = "../{name}--worktrees"

[agents]
max_concurrent = 3
instructions = "apm.agents.md"

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id         = "new"
label      = "New"
color      = "#6b7280"
actionable = ["agent"]

[[workflow.states]]
id         = "question"
label      = "Question"
color      = "#f59e0b"
actionable = ["supervisor"]

[[workflow.states]]
id         = "specd"
label      = "Specd"
color      = "#3b82f6"
actionable = ["supervisor"]

[[workflow.states]]
id         = "ammend"
label      = "Ammend"
color      = "#ef4444"
actionable = ["agent"]

[[workflow.states]]
id         = "ready"
label      = "Ready"
color      = "#10b981"
actionable = ["agent"]

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
  actor   = "agent"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"
color = "#8b5cf6"

[[workflow.states]]
id         = "blocked"
label      = "Blocked"
color      = "#dc2626"
actionable = ["supervisor"]

[[workflow.states]]
id         = "implemented"
label      = "Implemented"
color      = "#06b6d4"
actionable = ["supervisor"]

[[workflow.states]]
id         = "accepted"
label      = "Accepted"
color      = "#84cc16"
actionable = ["supervisor"]

[[workflow.states]]
id       = "closed"
label    = "Closed"
color    = "#374151"
terminal = true

# [logging]
# enabled = true
# file = "apm.log"
"##
    )
}

fn ensure_gitignore(path: &PathBuf) -> Result<()> {
    // tickets/NEXT_ID is a local counter used when apm/meta branch is unavailable.
    let entries = ["tickets/NEXT_ID"];
    if path.exists() {
        let mut contents = std::fs::read_to_string(path)?;
        let mut changed = false;
        for entry in &entries {
            if !contents.contains(entry) {
                if !contents.ends_with('\n') { contents.push('\n'); }
                contents.push_str(entry);
                contents.push('\n');
                changed = true;
            }
        }
        if changed {
            std::fs::write(path, &contents)?;
            println!("Updated .gitignore");
        }
    } else {
        std::fs::write(path, entries.join("\n") + "\n")?;
        println!("Created .gitignore");
    }
    Ok(())
}

const APM_ALLOW_ENTRIES: &[&str] = &[
    "Bash(apm sync*)",
    "Bash(apm next*)",
    "Bash(apm list*)",
    "Bash(apm show*)",
    "Bash(apm set *)",
    "Bash(apm state *)",
    "Bash(apm start *)",
    "Bash(apm take *)",
    "Bash(apm spec *)",
    "Bash(apm agents*)",
    "Bash(apm _hook *)",
    "Bash(apm verify*)",
    "Bash(apm new *)",
    "Bash(apm worktrees*)",
];

/// Entries added to ~/.claude/settings.json so subagents running in isolated
/// worktrees (which don't inherit project settings) can use git and apm.
const APM_USER_ALLOW_ENTRIES: &[&str] = &[
    "Bash(git add*)",
    "Bash(git commit*)",
    "Bash(git -C*)",
    "Bash(apm sync*)",
    "Bash(apm next*)",
    "Bash(apm list*)",
    "Bash(apm show*)",
    "Bash(apm set *)",
    "Bash(apm state *)",
    "Bash(apm start *)",
    "Bash(apm take *)",
    "Bash(apm agents*)",
    "Bash(apm verify*)",
    "Bash(apm new *)",
    "Bash(apm worktrees*)",
];

fn update_claude_settings(root: &Path, skip: bool) -> Result<()> {
    if skip {
        return Ok(());
    }
    let settings_path = root.join(".claude/settings.json");
    if !settings_path.exists() {
        return Ok(());
    }

    let raw = std::fs::read_to_string(&settings_path)?;
    let mut val: Value = serde_json::from_str(&raw)?;

    let allow = val
        .pointer_mut("/permissions/allow")
        .and_then(|v| v.as_array_mut());

    let missing: Vec<&str> = if let Some(arr) = allow {
        APM_ALLOW_ENTRIES
            .iter()
            .filter(|&&e| !arr.iter().any(|v| v.as_str() == Some(e)))
            .copied()
            .collect()
    } else {
        APM_ALLOW_ENTRIES.to_vec()
    };

    if missing.is_empty() {
        return Ok(());
    }

    println!("The following entries will be added to .claude/settings.json permissions.allow:");
    for e in &missing {
        println!("  {e}");
    }
    print!("Add apm commands to Claude allow list? [y/N] ");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    if !line.trim().eq_ignore_ascii_case("y") {
        println!("Skipped.");
        return Ok(());
    }

    // Ensure permissions.allow array exists
    if val.pointer("/permissions/allow").is_none() {
        let perms = val
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("settings.json root is not an object"))?
            .entry("permissions")
            .or_insert_with(|| Value::Object(Default::default()));
        perms
            .as_object_mut()
            .unwrap()
            .entry("allow")
            .or_insert_with(|| Value::Array(vec![]));
    }

    let arr = val
        .pointer_mut("/permissions/allow")
        .and_then(|v| v.as_array_mut())
        .unwrap();
    for e in missing {
        arr.push(Value::String(e.to_string()));
    }

    let updated = serde_json::to_string_pretty(&val)?;
    std::fs::write(&settings_path, updated + "\n")?;
    println!("Updated .claude/settings.json");
    Ok(())
}

fn update_user_claude_settings() -> Result<()> {
    let home = match std::env::var("HOME") {
        Ok(h) if !h.is_empty() => h,
        _ => return Ok(()),
    };
    let settings_path = PathBuf::from(&home).join(".claude/settings.json");

    let mut val: Value = if settings_path.exists() {
        let raw = std::fs::read_to_string(&settings_path)?;
        serde_json::from_str(&raw).unwrap_or(Value::Object(Default::default()))
    } else {
        Value::Object(Default::default())
    };

    let allow = val
        .pointer_mut("/permissions/allow")
        .and_then(|v| v.as_array_mut());

    let missing: Vec<&str> = if let Some(arr) = allow {
        APM_USER_ALLOW_ENTRIES
            .iter()
            .filter(|&&e| !arr.iter().any(|v| v.as_str() == Some(e)))
            .copied()
            .collect()
    } else {
        APM_USER_ALLOW_ENTRIES.to_vec()
    };

    if missing.is_empty() {
        return Ok(());
    }

    println!("The following entries will be added to ~/.claude/settings.json (user-level,");
    println!("required so apm subagents in isolated worktrees can run git and apm commands):");
    for e in &missing {
        println!("  {e}");
    }
    print!("Add to ~/.claude/settings.json? [y/N] ");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    if !line.trim().eq_ignore_ascii_case("y") {
        println!("Skipped.");
        return Ok(());
    }

    if val.pointer("/permissions/allow").is_none() {
        let perms = val
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("~/.claude/settings.json root is not an object"))?
            .entry("permissions")
            .or_insert_with(|| Value::Object(Default::default()));
        perms
            .as_object_mut()
            .unwrap()
            .entry("allow")
            .or_insert_with(|| Value::Array(vec![]));
    }

    let arr = val
        .pointer_mut("/permissions/allow")
        .and_then(|v| v.as_array_mut())
        .unwrap();
    for e in missing {
        arr.push(Value::String(e.to_string()));
    }

    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let updated = serde_json::to_string_pretty(&val)?;
    std::fs::write(&settings_path, updated + "\n")?;
    println!("Updated ~/.claude/settings.json");
    Ok(())
}

fn write_hooks(git_dir: &PathBuf) -> Result<()> {
    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    let pre_push = hooks_dir.join("pre-push");
    std::fs::write(
        &pre_push,
        "#!/bin/sh\n# Fires event:branch_push_first on first push of ticket/<id>-* in ready state\ncommand -v apm >/dev/null 2>&1 && apm _hook pre-push || true\n",
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

/// Create the worktrees directory specified in apm.toml (if the config exists).
fn ensure_worktrees_dir(root: &Path) -> Result<()> {
    if let Ok(config) = apm_core::config::Config::load(root) {
        let wt_dir = root.join(&config.worktrees.dir);
        if !wt_dir.exists() {
            std::fs::create_dir_all(&wt_dir)?;
            println!("Created worktrees dir: {}", wt_dir.display());
        }
    }
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
