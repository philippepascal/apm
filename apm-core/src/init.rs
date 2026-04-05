use anyhow::Result;
use std::io::IsTerminal;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// What happened when we tried to write a default file.
enum WriteAction {
    Created,
    Unchanged,
    Replaced,
    InitWritten,
    Skipped,
}

/// Write `content` to `path`. If the file already exists and differs from the
/// default, prompt the user: [s]kip, [r]eplace, or write a .init copy for
/// comparison. In non-interactive mode, silently writes the .init copy.
fn write_default(path: &Path, content: &str, label: &str) -> Result<WriteAction> {
    if !path.exists() {
        std::fs::write(path, content)?;
        println!("Created {label}");
        return Ok(WriteAction::Created);
    }

    let existing = std::fs::read_to_string(path)?;
    if existing == content {
        return Ok(WriteAction::Unchanged);
    }

    if !std::io::stdin().is_terminal() {
        let init_path = init_path_for(path);
        std::fs::write(&init_path, content)?;
        println!("{label} differs from default — wrote {}.init for comparison", label);
        return Ok(WriteAction::InitWritten);
    }

    print!("{label} exists and differs from default. [s]kip / [r]eplace / [c]ompare (.init)? ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    match line.trim().to_ascii_lowercase().as_str() {
        "r" | "replace" => {
            std::fs::write(path, content)?;
            println!("Replaced {label}");
            Ok(WriteAction::Replaced)
        }
        "c" | "compare" => {
            let init_path = init_path_for(path);
            std::fs::write(&init_path, content)?;
            println!("Wrote {label}.init — compare with your version and delete when done");
            Ok(WriteAction::InitWritten)
        }
        _ => {
            println!("Skipped {label}");
            Ok(WriteAction::Skipped)
        }
    }
}

/// foo.toml → foo.toml.init, agents.md → agents.md.init
fn init_path_for(path: &Path) -> std::path::PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".init");
    path.with_file_name(name)
}

pub fn setup(root: &Path) -> Result<()> {
    let tickets_dir = root.join("tickets");
    if !tickets_dir.exists() {
        std::fs::create_dir_all(&tickets_dir)?;
        println!("Created tickets/");
    }

    let apm_dir = root.join(".apm");
    std::fs::create_dir_all(&apm_dir)?;

    let local_toml = apm_dir.join("local.toml");
    let is_tty = std::io::stdin().is_terminal();

    // Check if git_host is configured — if so, identity comes from the provider
    let has_git_host = {
        let config_path = apm_dir.join("config.toml");
        config_path.exists() && crate::config::Config::load(root)
            .map(|cfg| cfg.git_host.provider.is_some())
            .unwrap_or(false)
    };

    // Only prompt for local username when there is no git_host
    let username = if !has_git_host && !local_toml.exists() {
        let u = if is_tty {
            prompt_username()?
        } else {
            String::new()
        };
        if !u.is_empty() {
            write_local_toml(&apm_dir, &u)?;
            println!("Created .apm/local.toml");
            u
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let config_path = apm_dir.join("config.toml");
    if !config_path.exists() {
        let default_name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");
        let (name, description) = if is_tty {
            prompt_project_info(default_name)?
        } else {
            (default_name.to_string(), String::new())
        };
        let collaborators: Vec<&str> = if username.is_empty() {
            vec![]
        } else {
            vec![username.as_str()]
        };
        let branch = detect_default_branch(root);
        std::fs::write(&config_path, default_config(&name, &description, &branch, &collaborators))?;
        println!("Created .apm/config.toml");
    } else {
        // Extract project values from existing config to generate a
        // comparable default (so the .init file has the right name/branch).
        let existing = std::fs::read_to_string(&config_path)?;
        if let Ok(val) = existing.parse::<toml::Value>() {
            let name = val.get("project")
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("project");
            let description = val.get("project")
                .and_then(|p| p.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let branch = val.get("project")
                .and_then(|p| p.get("default_branch"))
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            write_default(&config_path, &default_config(name, description, branch, &[]), ".apm/config.toml")?;
        }
    }
    write_default(&apm_dir.join("workflow.toml"), default_workflow_toml(), ".apm/workflow.toml")?;
    write_default(&apm_dir.join("ticket.toml"), default_ticket_toml(), ".apm/ticket.toml")?;
    write_default(&apm_dir.join("agents.md"), default_agents_md(), ".apm/agents.md")?;
    write_default(&apm_dir.join("apm.spec-writer.md"), include_str!("apm.spec-writer.md"), ".apm/apm.spec-writer.md")?;
    write_default(&apm_dir.join("apm.worker.md"), include_str!("apm.worker.md"), ".apm/apm.worker.md")?;
    ensure_claude_md(root, ".apm/agents.md")?;
    let gitignore = root.join(".gitignore");
    ensure_gitignore(&gitignore)?;
    maybe_initial_commit(root)?;
    ensure_worktrees_dir(root)?;
    Ok(())
}

pub fn migrate(root: &Path) -> Result<()> {
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

pub fn detect_default_branch(root: &Path) -> String {
    Command::new("git")
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

pub fn ensure_gitignore(path: &Path) -> Result<()> {
    let entries = ["tickets/NEXT_ID", ".apm/local.toml", ".apm/*.init", ".apm/sessions.json", ".apm/credentials.json"];
    if path.exists() {
        let mut contents = std::fs::read_to_string(path)?;
        let mut changed = false;
        for entry in &entries {
            if !contents.contains(entry) {
                if !contents.ends_with('\n') {
                    contents.push('\n');
                }
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
    include_str!("apm.agents.md")
}

#[cfg(target_os = "macos")]
fn default_log_file(name: &str) -> String {
    format!("~/Library/Logs/apm/{name}.log")
}

#[cfg(not(target_os = "macos"))]
fn default_log_file(name: &str) -> String {
    format!("~/.local/state/apm/{name}.log")
}

fn prompt_project_info(default_name: &str) -> Result<(String, String)> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    print!("Project name [{}]: ", default_name);
    stdout.flush()?;
    let mut name_input = String::new();
    stdin.read_line(&mut name_input)?;
    let name = {
        let trimmed = name_input.trim();
        if trimmed.is_empty() {
            default_name.to_string()
        } else {
            trimmed.to_string()
        }
    };

    print!("Project description []: ");
    stdout.flush()?;
    let mut desc_input = String::new();
    stdin.read_line(&mut desc_input)?;
    let description = desc_input.trim().to_string();

    Ok((name, description))
}

fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn default_config(name: &str, description: &str, default_branch: &str, collaborators: &[&str]) -> String {
    let log_file = default_log_file(name);
    let name = toml_escape(name);
    let description = toml_escape(description);
    let default_branch = toml_escape(default_branch);
    let log_file = toml_escape(&log_file);
    let collaborators_line = {
        let items: Vec<String> = collaborators.iter().map(|u| format!("\"{}\"", toml_escape(u))).collect();
        format!("collaborators = [{}]", items.join(", "))
    };
    format!(
        r##"[project]
name = "{name}"
description = "{description}"
default_branch = "{default_branch}"
{collaborators_line}

[tickets]
dir = "tickets"

[worktrees]
dir = "../{name}--worktrees"

[agents]
max_concurrent = 3
instructions = ".apm/agents.md"

[workers]
command = "claude"
args = ["--print"]

[logging]
enabled = false
file = "{log_file}"
"##
    )
}

fn prompt_username() -> Result<String> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    print!("Username []: ");
    stdout.flush()?;
    let mut input = String::new();
    stdin.read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn write_local_toml(apm_dir: &Path, username: &str) -> Result<()> {
    let path = apm_dir.join("local.toml");
    if !path.exists() {
        let username_escaped = toml_escape(username);
        std::fs::write(&path, format!("username = \"{username_escaped}\"\n"))?;
    }
    Ok(())
}

fn default_workflow_toml() -> &'static str {
    r##"[workflow]

[[workflow.states]]
id    = "new"
label = "New"
color = "#6b7280"

  [[workflow.states.transitions]]
  to      = "groomed"
  trigger = "manual"
  actor   = "supervisor"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id           = "groomed"
label        = "Groomed"
color        = "#6366f1"
actionable   = ["agent"]
dep_requires = "spec"
instructions = ".apm/apm.spec-writer.md"

  [[workflow.states.transitions]]
  to              = "in_design"
  trigger         = "command:start"
  actor           = "agent"
  context_section = "Problem"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id         = "question"
label      = "Question"
color      = "#f59e0b"
actionable = ["supervisor"]

  [[workflow.states.transitions]]
  to      = "groomed"
  trigger = "manual"
  actor   = "any"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id             = "specd"
label          = "Specd"
color          = "#3b82f6"
actionable     = ["supervisor"]
satisfies_deps = "spec"
worker_end     = true

  [[workflow.states.transitions]]
  to           = "ready"
  trigger      = "manual"
  actor        = "supervisor"
  side_effects = ["set_agent_null"]

  [[workflow.states.transitions]]
  to      = "ammend"
  trigger = "manual"
  actor   = "supervisor"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id             = "ammend"
label          = "Ammend"
color          = "#ef4444"
actionable     = ["agent"]
dep_requires   = "spec"
satisfies_deps = "spec"
instructions   = ".apm/apm.spec-writer.md"

  [[workflow.states.transitions]]
  to            = "specd"
  trigger       = "manual"
  actor         = "agent"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

  [[workflow.states.transitions]]
  to      = "question"
  trigger = "manual"
  actor   = "agent"

  [[workflow.states.transitions]]
  to      = "in_design"
  trigger = "command:start"
  actor   = "agent"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id           = "in_design"
label        = "In Design"
color        = "#f97316"
instructions = ".apm/apm.spec-writer.md"

  [[workflow.states.transitions]]
  to            = "specd"
  trigger       = "manual"
  actor         = "agent"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

  [[workflow.states.transitions]]
  to      = "question"
  trigger = "manual"
  actor   = "agent"

  [[workflow.states.transitions]]
  to      = "ammend"
  trigger = "manual"
  actor   = "supervisor"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id             = "ready"
label          = "Ready"
color          = "#10b981"
actionable     = ["agent"]
satisfies_deps = "spec"
instructions   = ".apm/apm.worker.md"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
  actor   = "agent"

  [[workflow.states.transitions]]
  to      = "ammend"
  trigger = "manual"
  actor   = "supervisor"

  [[workflow.states.transitions]]
  to      = "specd"
  trigger = "manual"
  actor   = "supervisor"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id             = "in_progress"
label          = "In Progress"
color          = "#8b5cf6"
satisfies_deps = "spec"
instructions   = ".apm/apm.worker.md"

  [[workflow.states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  actor      = "agent"
  completion = "merge"

  [[workflow.states.transitions]]
  to      = "blocked"
  trigger = "manual"
  actor   = "agent"
  label   = "Agent is blocked — wrote questions in ### Open questions"

  [[workflow.states.transitions]]
  to           = "ready"
  trigger      = "manual"
  actor        = "supervisor"
  side_effects = ["set_agent_null"]
  warning      = "Reverting in_progress ticket to ready — any uncommitted work on the branch may be lost"

  [[workflow.states.transitions]]
  to           = "ammend"
  trigger      = "manual"
  actor        = "supervisor"
  side_effects = ["set_agent_null"]

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id         = "blocked"
label      = "Blocked"
color      = "#dc2626"
actionable = ["supervisor"]

  [[workflow.states.transitions]]
  to           = "ready"
  trigger      = "manual"
  actor        = "supervisor"
  label        = "Supervisor answered questions — agent can resume"
  side_effects = ["set_agent_null"]

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id             = "implemented"
label          = "Implemented"
color          = "#06b6d4"
actionable     = ["supervisor"]
satisfies_deps = true
worker_end     = true

  [[workflow.states.transitions]]
  to            = "ready"
  trigger       = "manual"
  actor         = "supervisor"
  side_effects  = ["set_agent_null"]
  focus_section = "Code review"

  [[workflow.states.transitions]]
  to           = "ammend"
  trigger      = "manual"
  actor        = "supervisor"
  side_effects = ["set_agent_null"]

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "manual"
  actor   = "any"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id             = "closed"
label          = "Closed"
color          = "#374151"
terminal       = true
satisfies_deps = true

[workflow.prioritization]
priority_weight = 10.0
effort_weight   = -2.0
risk_weight     = -1.0
"##
}

fn default_ticket_toml() -> &'static str {
    include_str!("ticket.toml")
}

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

    Command::new("git")
        .args(["add", ".apm/config.toml", ".apm/workflow.toml", ".apm/ticket.toml", ".gitignore"])
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

fn ensure_worktrees_dir(root: &Path) -> Result<()> {
    if let Ok(config) = crate::config::Config::load(root) {
        let wt_dir = root.join(&config.worktrees.dir);
        if !wt_dir.exists() {
            std::fs::create_dir_all(&wt_dir)?;
            println!("Created worktrees dir: {}", wt_dir.display());
        }
    }
    Ok(())
}

pub fn setup_docker(root: &Path) -> Result<()> {
    let apm_dir = root.join(".apm");
    std::fs::create_dir_all(&apm_dir)?;
    let dockerfile_path = apm_dir.join("Dockerfile.apm-worker");
    if dockerfile_path.exists() {
        println!(".apm/Dockerfile.apm-worker already exists — not overwriting.");
        return Ok(());
    }
    std::fs::write(&dockerfile_path, DOCKERFILE_TEMPLATE)?;
    println!("Created .apm/Dockerfile.apm-worker");
    println!();
    println!("Next steps:");
    println!("  1. Review .apm/Dockerfile.apm-worker and add project-specific dependencies.");
    println!("  2. Build the image:");
    println!("       docker build -f .apm/Dockerfile.apm-worker -t apm-worker .");
    println!("  3. Add to .apm/config.toml:");
    println!("       [workers]");
    println!("       container = \"apm-worker\"");
    println!("  4. Configure credential lookup (optional, macOS only):");
    println!("       [workers.keychain]");
    println!("       ANTHROPIC_API_KEY = \"anthropic-api-key\"");
    Ok(())
}

const DOCKERFILE_TEMPLATE: &str = r#"FROM rust:1.82-slim

# System tools
RUN apt-get update && apt-get install -y \
    curl git unzip ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Claude CLI
RUN curl -fsSL https://storage.googleapis.com/anthropic-claude-cli/install.sh | sh

# apm binary (replace with your version or a downloaded release)
COPY target/release/apm /usr/local/bin/apm

# Add project-specific dependencies here:
# RUN apt-get install -y nodejs npm   # for Node projects
# RUN pip install -r requirements.txt # for Python projects

# gh CLI is NOT needed — the worker only runs local git commits;
# push and PR creation happen on the host via apm state <id> implemented.

WORKDIR /workspace
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn git_init(dir: &Path) {
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn detect_default_branch_fresh_repo() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        let branch = detect_default_branch(tmp.path());
        assert_eq!(branch, "main");
    }

    #[test]
    fn detect_default_branch_non_git() {
        let tmp = TempDir::new().unwrap();
        let branch = detect_default_branch(tmp.path());
        assert_eq!(branch, "main");
    }

    #[test]
    fn ensure_gitignore_creates_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        ensure_gitignore(&path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("tickets/NEXT_ID"));
        assert!(contents.contains(".apm/local.toml"));
        assert!(contents.contains(".apm/*.init"));
        assert!(contents.contains(".apm/sessions.json"));
        assert!(contents.contains(".apm/credentials.json"));
    }

    #[test]
    fn ensure_gitignore_appends_missing_entry() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        std::fs::write(&path, "node_modules\n").unwrap();
        ensure_gitignore(&path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("node_modules"));
        assert!(contents.contains("tickets/NEXT_ID"));
    }

    #[test]
    fn ensure_gitignore_idempotent() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        ensure_gitignore(&path).unwrap();
        let before = std::fs::read_to_string(&path).unwrap();
        ensure_gitignore(&path).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn setup_creates_expected_files() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        setup(tmp.path()).unwrap();

        assert!(tmp.path().join("tickets").exists());
        assert!(tmp.path().join(".apm/config.toml").exists());
        assert!(tmp.path().join(".apm/workflow.toml").exists());
        assert!(tmp.path().join(".apm/ticket.toml").exists());
        assert!(tmp.path().join(".apm/agents.md").exists());
        assert!(tmp.path().join(".apm/apm.spec-writer.md").exists());
        assert!(tmp.path().join(".apm/apm.worker.md").exists());
        assert!(tmp.path().join(".gitignore").exists());
        assert!(tmp.path().join("CLAUDE.md").exists());
    }

    #[test]
    fn setup_non_tty_uses_dir_name_and_empty_description() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        setup(tmp.path()).unwrap();

        let config = std::fs::read_to_string(tmp.path().join(".apm/config.toml")).unwrap();
        let dir_name = tmp.path().file_name().unwrap().to_str().unwrap();
        assert!(config.contains(&format!("name = \"{dir_name}\"")));
        assert!(config.contains("description = \"\""));
    }

    #[test]
    fn setup_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        setup(tmp.path()).unwrap();

        // Write sentinel content to config
        let config_path = tmp.path().join(".apm/config.toml");
        let original = std::fs::read_to_string(&config_path).unwrap();

        setup(tmp.path()).unwrap();
        let after = std::fs::read_to_string(&config_path).unwrap();
        assert_eq!(original, after);
    }

    #[test]
    fn migrate_moves_files_and_updates_claude_md() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());

        std::fs::write(tmp.path().join("apm.toml"), "[project]\nname = \"x\"\n").unwrap();
        std::fs::write(tmp.path().join("apm.agents.md"), "# agents\n").unwrap();
        std::fs::write(tmp.path().join("CLAUDE.md"), "@apm.agents.md\n\nContent\n").unwrap();

        migrate(tmp.path()).unwrap();

        assert!(tmp.path().join(".apm/config.toml").exists());
        assert!(tmp.path().join(".apm/agents.md").exists());
        assert!(!tmp.path().join("apm.toml").exists());
        assert!(!tmp.path().join("apm.agents.md").exists());

        let claude = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
        assert!(claude.contains("@.apm/agents.md"));
        assert!(!claude.contains("@apm.agents.md"));
    }

    #[test]
    fn migrate_already_migrated() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        std::fs::create_dir_all(tmp.path().join(".apm")).unwrap();
        std::fs::write(tmp.path().join(".apm/config.toml"), "").unwrap();

        // Should not panic or error
        migrate(tmp.path()).unwrap();
    }

    #[test]
    fn setup_docker_creates_dockerfile() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        setup_docker(tmp.path()).unwrap();
        let dockerfile = tmp.path().join(".apm/Dockerfile.apm-worker");
        assert!(dockerfile.exists());
        let contents = std::fs::read_to_string(&dockerfile).unwrap();
        assert!(contents.contains("FROM rust:1.82-slim"));
        assert!(contents.contains("claude"));
        assert!(!contents.contains("gh CLI") || contents.contains("NOT needed"));
    }

    #[test]
    fn setup_docker_idempotent() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        setup_docker(tmp.path()).unwrap();
        let before = std::fs::read_to_string(tmp.path().join(".apm/Dockerfile.apm-worker")).unwrap();
        // Second call should not overwrite
        setup_docker(tmp.path()).unwrap();
        let after = std::fs::read_to_string(tmp.path().join(".apm/Dockerfile.apm-worker")).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn default_config_escapes_special_chars() {
        let name = r#"my\"project"#;
        let description = r#"desc with "quotes" and \backslash"#;
        let branch = "main";
        let config = default_config(name, description, branch, &[]);
        toml::from_str::<toml::Value>(&config).expect("default_config output must be valid TOML");
    }

    #[test]
    fn write_local_toml_creates_file() {
        let tmp = TempDir::new().unwrap();
        write_local_toml(tmp.path(), "alice").unwrap();
        let contents = std::fs::read_to_string(tmp.path().join("local.toml")).unwrap();
        assert!(contents.contains("username = \"alice\""));
    }

    #[test]
    fn write_local_toml_idempotent() {
        let tmp = TempDir::new().unwrap();
        write_local_toml(tmp.path(), "alice").unwrap();
        let first = std::fs::read_to_string(tmp.path().join("local.toml")).unwrap();
        write_local_toml(tmp.path(), "bob").unwrap();
        let second = std::fs::read_to_string(tmp.path().join("local.toml")).unwrap();
        assert_eq!(first, second);
        assert!(second.contains("alice"));
    }

    #[test]
    fn setup_non_tty_no_local_toml() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        setup(tmp.path()).unwrap();
        assert!(!tmp.path().join(".apm/local.toml").exists());
    }

    #[test]
    fn default_config_with_collaborators() {
        let config = default_config("proj", "desc", "main", &["alice"]);
        let parsed: toml::Value = toml::from_str(&config).unwrap();
        let collaborators = parsed["project"]["collaborators"].as_array().unwrap();
        assert_eq!(collaborators.len(), 1);
        assert_eq!(collaborators[0].as_str().unwrap(), "alice");
    }

    #[test]
    fn default_config_empty_collaborators() {
        let config = default_config("proj", "desc", "main", &[]);
        let parsed: toml::Value = toml::from_str(&config).unwrap();
        let collaborators = parsed["project"]["collaborators"].as_array().unwrap();
        assert!(collaborators.is_empty());
    }

    #[test]
    fn write_default_creates_new_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.toml");
        let action = write_default(&path, "content", "test.toml").unwrap();
        assert!(matches!(action, WriteAction::Created));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "content");
    }

    #[test]
    fn write_default_unchanged_when_identical() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.toml");
        std::fs::write(&path, "content").unwrap();
        let action = write_default(&path, "content", "test.toml").unwrap();
        assert!(matches!(action, WriteAction::Unchanged));
    }

    #[test]
    fn write_default_non_tty_writes_init_when_differs() {
        // In test context stdin is not a terminal, so this exercises
        // the non-interactive path: write .init copy.
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.toml");
        std::fs::write(&path, "modified").unwrap();
        let action = write_default(&path, "default", "test.toml").unwrap();
        assert!(matches!(action, WriteAction::InitWritten));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "modified");
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("test.toml.init")).unwrap(),
            "default"
        );
    }

    #[test]
    fn init_path_for_preserves_extension() {
        let p = std::path::Path::new("/a/b/workflow.toml");
        assert_eq!(init_path_for(p), std::path::PathBuf::from("/a/b/workflow.toml.init"));

        let p = std::path::Path::new("/a/b/agents.md");
        assert_eq!(init_path_for(p), std::path::PathBuf::from("/a/b/agents.md.init"));
    }

    #[test]
    fn setup_writes_init_files_when_content_differs() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        // First setup: creates all files
        setup(tmp.path()).unwrap();

        // Modify a file
        let workflow = tmp.path().join(".apm/workflow.toml");
        std::fs::write(&workflow, "# custom workflow\n").unwrap();

        // Second setup (non-tty): should write .init copy
        setup(tmp.path()).unwrap();
        assert!(tmp.path().join(".apm/workflow.toml.init").exists());
        // Original should be untouched
        assert_eq!(std::fs::read_to_string(&workflow).unwrap(), "# custom workflow\n");
        // .init should have the default content
        let init_content = std::fs::read_to_string(tmp.path().join(".apm/workflow.toml.init")).unwrap();
        assert_eq!(init_content, default_workflow_toml());
    }

    #[test]
    fn setup_writes_config_init_when_modified() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        setup(tmp.path()).unwrap();

        // Modify config.toml (add a custom section)
        let config_path = tmp.path().join(".apm/config.toml");
        let mut content = std::fs::read_to_string(&config_path).unwrap();
        content.push_str("\n[custom]\nfoo = \"bar\"\n");
        std::fs::write(&config_path, &content).unwrap();

        // Second setup (non-tty): should write config.toml.init
        setup(tmp.path()).unwrap();
        assert!(tmp.path().join(".apm/config.toml.init").exists());
        // Original should be untouched
        assert!(std::fs::read_to_string(&config_path).unwrap().contains("[custom]"));
        // .init should be the default for this project's name/branch
        let init_content = std::fs::read_to_string(tmp.path().join(".apm/config.toml.init")).unwrap();
        assert!(!init_content.contains("[custom]"));
        assert!(init_content.contains("[project]"));
        assert!(init_content.contains("[workers]"));
    }

    #[test]
    fn default_workflow_toml_is_valid() {
        use crate::config::{SatisfiesDeps, WorkflowFile};

        let parsed: WorkflowFile = toml::from_str(default_workflow_toml()).unwrap();
        let states = &parsed.workflow.states;

        let ids: Vec<&str> = states.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            ["new", "groomed", "question", "specd", "ammend", "in_design", "ready", "in_progress", "blocked", "implemented", "closed"]
        );

        for id in ["groomed", "ammend"] {
            let s = states.iter().find(|s| s.id == id).unwrap();
            assert!(s.dep_requires.is_some(), "state {id} should have dep_requires");
        }

        for id in ["specd", "ammend", "ready", "in_progress", "implemented"] {
            let s = states.iter().find(|s| s.id == id).unwrap();
            assert_ne!(s.satisfies_deps, SatisfiesDeps::Bool(false), "state {id} should have satisfies_deps");
        }
    }

    #[test]
    fn default_ticket_toml_is_valid() {
        use crate::config::TicketFile;

        let parsed: TicketFile = toml::from_str(default_ticket_toml()).unwrap();
        let sections = &parsed.ticket.sections;

        for name in ["Problem", "Acceptance criteria", "Out of scope", "Approach"] {
            let s = sections.iter().find(|s| s.name == name).unwrap();
            assert!(s.required, "section '{name}' should be required");
        }
    }
}
