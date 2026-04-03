use anyhow::Result;
use std::io::IsTerminal;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub fn setup(root: &Path) -> Result<()> {
    let tickets_dir = root.join("tickets");
    if !tickets_dir.exists() {
        std::fs::create_dir_all(&tickets_dir)?;
        println!("Created tickets/");
    }

    let apm_dir = root.join(".apm");
    std::fs::create_dir_all(&apm_dir)?;

    let config_path = apm_dir.join("config.toml");
    if !config_path.exists() {
        let default_name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");
        let (name, description) = if std::io::stdin().is_terminal() {
            prompt_project_info(default_name)?
        } else {
            (default_name.to_string(), String::new())
        };
        let branch = detect_default_branch(root);
        std::fs::write(&config_path, default_config(&name, &description, &branch))?;
        println!("Created .apm/config.toml");
    }
    let workflow_path = apm_dir.join("workflow.toml");
    if !workflow_path.exists() {
        std::fs::write(&workflow_path, default_workflow_toml())?;
        println!("Created .apm/workflow.toml");
    }
    let ticket_path = apm_dir.join("ticket.toml");
    if !ticket_path.exists() {
        std::fs::write(&ticket_path, default_ticket_toml())?;
        println!("Created .apm/ticket.toml");
    }
    let agents_path = apm_dir.join("agents.md");
    if !agents_path.exists() {
        std::fs::write(&agents_path, default_agents_md())?;
        println!("Created .apm/agents.md");
    }
    let spec_writer_path = apm_dir.join("apm.spec-writer.md");
    if !spec_writer_path.exists() {
        std::fs::write(&spec_writer_path, include_str!("apm.spec-writer.md"))?;
        println!("Created .apm/apm.spec-writer.md");
    }
    let worker_md_path = apm_dir.join("apm.worker.md");
    if !worker_md_path.exists() {
        std::fs::write(&worker_md_path, include_str!("apm.worker.md"))?;
        println!("Created .apm/apm.worker.md");
    }
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
    let entries = ["tickets/NEXT_ID", ".apm/local.toml"];
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

fn default_config(name: &str, description: &str, default_branch: &str) -> String {
    let log_file = default_log_file(name);
    let name = toml_escape(name);
    let description = toml_escape(description);
    let default_branch = toml_escape(default_branch);
    let log_file = toml_escape(&log_file);
    format!(
        r##"[project]
name = "{name}"
description = "{description}"
default_branch = "{default_branch}"

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

fn default_workflow_toml() -> &'static str {
    r##"[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id    = "new"
label = "New"
color = "#6b7280"

[[workflow.states]]
id           = "groomed"
label        = "Groomed"
color        = "#6366f1"
actionable   = ["agent"]
instructions = ".apm/apm.spec-writer.md"

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
id           = "ammend"
label        = "Ammend"
color        = "#ef4444"
actionable   = ["agent"]
instructions = ".apm/apm.spec-writer.md"

[[workflow.states]]
id           = "in_design"
label        = "In Design"
color        = "#f97316"
actionable   = ["agent"]
instructions = ".apm/apm.spec-writer.md"

[[workflow.states]]
id           = "ready"
label        = "Ready"
color        = "#10b981"
actionable   = ["agent"]
instructions = ".apm/apm.worker.md"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
  actor   = "agent"

[[workflow.states]]
id           = "in_progress"
label        = "In Progress"
color        = "#8b5cf6"
instructions = ".apm/apm.worker.md"

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
id       = "closed"
label    = "Closed"
color    = "#374151"
terminal = true
"##
}

fn default_ticket_toml() -> &'static str {
    r#"[[ticket.sections]]
name        = "Problem"
type        = "free"
required    = true
placeholder = "What is broken or missing, and why it matters."

[[ticket.sections]]
name        = "Acceptance criteria"
type        = "tasks"
required    = true
placeholder = "Checkboxes; each one independently testable."

[[ticket.sections]]
name        = "Out of scope"
type        = "free"
required    = true
placeholder = "Explicit list of what this ticket does not cover."

[[ticket.sections]]
name        = "Approach"
type        = "free"
required    = true
placeholder = "How the implementation will work."

[[ticket.sections]]
name     = "Open questions"
type     = "qa"
required = false

[[ticket.sections]]
name     = "Amendment requests"
type     = "tasks"
required = false

[[ticket.sections]]
name     = "Code review"
type     = "tasks"
required = false
"#
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
        let config = default_config(name, description, branch);
        toml::from_str::<toml::Value>(&config).expect("default_config output must be valid TOML");
    }
}
