use anyhow::Result;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(root: &Path, no_claude: bool, migrate: bool) -> Result<()> {
    if migrate {
        return run_migrate(root);
    }

    let tickets_dir = root.join("tickets");
    if !tickets_dir.exists() {
        std::fs::create_dir_all(&tickets_dir)?;
        println!("Created tickets/");
    }

    // Write config to .apm/
    let apm_dir = root.join(".apm");
    std::fs::create_dir_all(&apm_dir)?;

    let config_path = apm_dir.join("config.toml");
    if !config_path.exists() {
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");
        let branch = detect_default_branch(root);
        std::fs::write(&config_path, default_config(name, &branch))?;
        println!("Created .apm/config.toml");
    }
    let agents_path = apm_dir.join("agents.md");
    if !agents_path.exists() {
        std::fs::write(&agents_path, default_agents_md())?;
        println!("Created .apm/agents.md");
    }
    let spec_writer_path = apm_dir.join("spec-writer.md");
    if !spec_writer_path.exists() {
        std::fs::write(&spec_writer_path, "# APM Spec-Writer Agent\n\n_Fill in spec-writing instructions here._\n")?;
        println!("Created .apm/spec-writer.md");
    }
    let worker_md_path = apm_dir.join("worker.md");
    if !worker_md_path.exists() {
        std::fs::write(&worker_md_path, include_str!("../apm.worker.md"))?;
        println!("Created .apm/worker.md");
    }
    ensure_claude_md(root, ".apm/agents.md")?;
    let gitignore = root.join(".gitignore");
    ensure_gitignore(&gitignore)?;
    update_claude_settings(root, no_claude)?;
    maybe_initial_commit(root)?;
    maybe_create_meta_branch(root)?;
    ensure_worktrees_dir(root)?;
    update_user_claude_settings()?;
    warn_if_settings_untracked(root);
    println!("apm initialized.");
    Ok(())
}

fn run_migrate(root: &Path) -> Result<()> {
    let apm_dir = root.join(".apm");
    let new_config = apm_dir.join("config.toml");

    if new_config.exists() {
        println!("Already migrated.");
        return Ok(());
    }

    let old_config = root.join("apm.toml");
    let old_agents = root.join("apm.agents.md");

    if !old_config.exists() && !old_agents.exists() {
        println!("Nothing to migrate.");
        return Ok(());
    }

    std::fs::create_dir_all(&apm_dir)?;

    if old_config.exists() {
        std::fs::rename(&old_config, &new_config)?;
        println!("Moved apm.toml → .apm/config.toml");
    }

    if old_agents.exists() {
        let new_agents = apm_dir.join("agents.md");
        std::fs::rename(&old_agents, &new_agents)?;
        println!("Moved apm.agents.md → .apm/agents.md");
    }

    // Update CLAUDE.md import if present
    let claude_path = root.join("CLAUDE.md");
    if claude_path.exists() {
        let contents = std::fs::read_to_string(&claude_path)?;
        if contents.contains("@apm.agents.md") {
            let updated = contents.replace("@apm.agents.md", "@.apm/agents.md");
            std::fs::write(&claude_path, updated)?;
            println!("Updated CLAUDE.md (@apm.agents.md → @.apm/agents.md)");
        }
    }

    Ok(())
}

fn warn_if_settings_untracked(root: &Path) {
    let settings = root.join(".claude/settings.json");
    if !settings.exists() {
        return;
    }
    let tracked = Command::new("git")
        .args(["ls-files", "--error-unmatch", ".claude/settings.json"])
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !tracked {
        eprintln!(
            "Warning: .claude/settings.json exists but is not committed. \
Agent worktrees won't have it — run: git add .claude/settings.json && git commit"
        );
    }
}

fn ensure_claude_md(root: &Path, agents_path: &str) -> Result<()> {
    let import_line = format!("@{agents_path}");
    let claude_path = root.join("CLAUDE.md");
    if claude_path.exists() {
        let contents = std::fs::read_to_string(&claude_path)?;
        if contents.contains(&import_line) {
            return Ok(());
        }
        std::fs::write(&claude_path, format!("{import_line}\n\n{contents}"))?;
        println!("Updated CLAUDE.md (added {import_line} import).");
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

#[cfg(target_os = "macos")]
fn default_log_file(name: &str) -> String {
    format!("~/Library/Logs/apm/{name}.log")
}

#[cfg(not(target_os = "macos"))]
fn default_log_file(name: &str) -> String {
    format!("~/.local/state/apm/{name}.log")
}

fn default_config(name: &str, default_branch: &str) -> String {
    let log_file = default_log_file(name);
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
id         = "in_design"
label      = "In Design"
color      = "#f97316"
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

  [[workflow.states.transitions]]
  to      = "implemented"
  trigger = "manual"
  actor   = "agent"

  [[workflow.states.transitions]]
  to      = "blocked"
  trigger = "command:block"
  actor   = "agent"

[[workflow.states]]
id         = "blocked"
label      = "Blocked"
color      = "#dc2626"
actionable = ["supervisor"]

  [[workflow.states.transitions]]
  to      = "ready"
  trigger = "command:unblock"
  actor   = "supervisor"

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

[logging]
enabled = false
file = "{log_file}"
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

    // Stage .apm/config.toml and .gitignore.
    Command::new("git")
        .args(["add", ".apm/config.toml", ".gitignore"])
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
