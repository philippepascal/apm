use anyhow::Result;
use std::path::Path;

pub struct SetupOutput {
    pub messages: Vec<String>,
}

pub struct SetupDockerOutput {
    pub messages: Vec<String>,
}

/// What happened when we tried to write a default file.
#[allow(dead_code)]
enum WriteAction {
    Created,
    Unchanged,
    Replaced,
    InitWritten,
    Skipped,
}

/// Write `content` to `path`. If the file already exists and differs from the
/// default, write a .init copy for comparison (always non-interactive in core).
fn write_default(path: &Path, content: &str, label: &str, messages: &mut Vec<String>) -> Result<WriteAction> {
    if !path.exists() {
        std::fs::write(path, content)?;
        messages.push(format!("Created {label}"));
        return Ok(WriteAction::Created);
    }

    let existing = std::fs::read_to_string(path)?;
    if existing == content {
        return Ok(WriteAction::Unchanged);
    }

    // Always take the non-interactive path in the library: write .init copy.
    let init_path = init_path_for(path);
    std::fs::write(&init_path, content)?;
    messages.push(format!("{label} differs from default — wrote {label}.init for comparison"));
    Ok(WriteAction::InitWritten)
}

/// foo.toml → foo.toml.init, agents.md → agents.md.init
fn init_path_for(path: &Path) -> std::path::PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".init");
    path.with_file_name(name)
}

pub fn setup(root: &Path, name: Option<&str>, description: Option<&str>, username: Option<&str>) -> Result<SetupOutput> {
    let mut messages: Vec<String> = Vec::new();

    let tickets_dir = root.join("tickets");
    if !tickets_dir.exists() {
        std::fs::create_dir_all(&tickets_dir)?;
        messages.push("Created tickets/".to_string());
    }

    let apm_dir = root.join(".apm");
    std::fs::create_dir_all(&apm_dir)?;

    let local_toml = apm_dir.join("local.toml");

    // Check if git_host is configured — if so, identity comes from the provider
    let has_git_host = {
        let config_path = apm_dir.join("config.toml");
        config_path.exists() && crate::config::Config::load(root)
            .map(|cfg| cfg.git_host.provider.is_some())
            .unwrap_or(false)
    };

    // Only write local username when there is no git_host
    if !has_git_host && !local_toml.exists() {
        if let Some(u) = username {
            if !u.is_empty() {
                write_local_toml(&apm_dir, u)?;
                messages.push("Created .apm/local.toml".to_string());
            }
        }
    }

    let effective_username = username.unwrap_or("");
    let config_path = apm_dir.join("config.toml");
    if !config_path.exists() {
        let default_name = name.unwrap_or_else(|| {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
        });
        let effective_description = description.unwrap_or("");
        let collaborators: Vec<&str> = if effective_username.is_empty() {
            vec![]
        } else {
            vec![effective_username]
        };
        let branch = detect_default_branch(root);
        std::fs::write(&config_path, default_config(default_name, effective_description, &branch, &collaborators))?;
        messages.push("Created .apm/config.toml".to_string());
    } else {
        // Extract project values from existing config to generate a
        // comparable default (so the .init file has the right name/branch).
        let existing = std::fs::read_to_string(&config_path)?;
        if let Ok(val) = existing.parse::<toml::Value>() {
            let n = val.get("project")
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("project");
            let d = val.get("project")
                .and_then(|p| p.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let b = val.get("project")
                .and_then(|p| p.get("default_branch"))
                .and_then(|v| v.as_str())
                .unwrap_or("main");
            write_default(&config_path, &default_config(n, d, b, &[]), ".apm/config.toml", &mut messages)?;
        }
    }
    write_default(&apm_dir.join("workflow.toml"), default_workflow_toml(), ".apm/workflow.toml", &mut messages)?;
    write_default(&apm_dir.join("ticket.toml"), default_ticket_toml(), ".apm/ticket.toml", &mut messages)?;
    write_default(&apm_dir.join("agents.md"), default_agents_md(), ".apm/agents.md", &mut messages)?;
    write_default(&apm_dir.join("apm.spec-writer.md"), include_str!("default/apm.spec-writer.md"), ".apm/apm.spec-writer.md", &mut messages)?;
    write_default(&apm_dir.join("apm.worker.md"), include_str!("default/apm.worker.md"), ".apm/apm.worker.md", &mut messages)?;
    ensure_claude_md(root, ".apm/agents.md", &mut messages)?;
    let gitignore = root.join(".gitignore");
    ensure_gitignore(&gitignore, &mut messages)?;
    maybe_initial_commit(root, &mut messages)?;
    ensure_worktrees_dir(root, &mut messages)?;
    Ok(SetupOutput { messages })
}

pub fn migrate(root: &Path) -> Result<Vec<String>> {
    let mut messages: Vec<String> = Vec::new();
    let apm_dir = root.join(".apm");
    let new_config = apm_dir.join("config.toml");

    if new_config.exists() {
        messages.push("Already migrated.".to_string());
        return Ok(messages);
    }

    let old_config = root.join("apm.toml");
    let old_agents = root.join("apm.agents.md");

    if !old_config.exists() && !old_agents.exists() {
        messages.push("Nothing to migrate.".to_string());
        return Ok(messages);
    }

    std::fs::create_dir_all(&apm_dir)?;

    if old_config.exists() {
        std::fs::rename(&old_config, &new_config)?;
        messages.push("Moved apm.toml → .apm/config.toml".to_string());
    }

    if old_agents.exists() {
        let new_agents = apm_dir.join("agents.md");
        std::fs::rename(&old_agents, &new_agents)?;
        messages.push("Moved apm.agents.md → .apm/agents.md".to_string());
    }

    let claude_path = root.join("CLAUDE.md");
    if claude_path.exists() {
        let contents = std::fs::read_to_string(&claude_path)?;
        if contents.contains("@apm.agents.md") {
            let updated = contents.replace("@apm.agents.md", "@.apm/agents.md");
            std::fs::write(&claude_path, updated)?;
            messages.push("Updated CLAUDE.md (@apm.agents.md → @.apm/agents.md)".to_string());
        }
    }

    Ok(messages)
}

pub fn detect_default_branch(root: &Path) -> String {
    crate::git_util::current_branch(root)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "main".to_string())
}

pub fn ensure_gitignore(path: &Path, messages: &mut Vec<String>) -> Result<()> {
    let entries = ["tickets/NEXT_ID", ".apm/local.toml", ".apm/epics.toml", ".apm/*.init", ".apm/sessions.json", ".apm/credentials.json"];
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
            messages.push("Updated .gitignore".to_string());
        }
    } else {
        std::fs::write(path, entries.join("\n") + "\n")?;
        messages.push("Created .gitignore".to_string());
    }
    Ok(())
}

fn ensure_claude_md(root: &Path, agents_path: &str, messages: &mut Vec<String>) -> Result<()> {
    let import_line = format!("@{agents_path}");
    let claude_path = root.join("CLAUDE.md");
    if claude_path.exists() {
        let contents = std::fs::read_to_string(&claude_path)?;
        if contents.contains(&import_line) {
            return Ok(());
        }
        std::fs::write(&claude_path, format!("{import_line}\n\n{contents}"))?;
        messages.push(format!("Updated CLAUDE.md (added {import_line} import)."));
    } else {
        std::fs::write(&claude_path, format!("{import_line}\n"))?;
        messages.push("Created CLAUDE.md.".to_string());
    }
    Ok(())
}

fn default_agents_md() -> &'static str {
    include_str!("default/apm.agents.md")
}

#[cfg(target_os = "macos")]
fn default_log_file(name: &str) -> String {
    format!("~/Library/Logs/apm/{name}.log")
}

#[cfg(not(target_os = "macos"))]
fn default_log_file(name: &str) -> String {
    format!("~/.local/state/apm/{name}.log")
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
archive_dir = "archive/tickets"

[worktrees]
dir = "../{name}--worktrees"
agent_dirs = [".claude", ".cursor", ".windsurf"]

[agents]
max_concurrent = 3
instructions = ".apm/agents.md"

[workers]
command = "claude"
args = ["--print"]

[worker_profiles.spec_agent]
command = "claude"
args = ["--print"]
instructions = ".apm/apm.spec-writer.md"
role_prefix = "You are a Spec-Writer agent assigned to ticket #<id>."

[worker_profiles.impl_agent]
command = "claude"
args = ["--print"]
instructions = ".apm/apm.worker.md"
role_prefix = "You are a Worker agent assigned to ticket #<id>."

[logging]
enabled = false
file = "{log_file}"
"##
    )
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
    include_str!("default/workflow.toml")
}

fn default_ticket_toml() -> &'static str {
    include_str!("default/ticket.toml")
}

fn maybe_initial_commit(root: &Path, messages: &mut Vec<String>) -> Result<()> {
    if crate::git_util::has_commits(root) {
        return Ok(());
    }

    crate::git_util::stage_files(root, &[
        ".apm/config.toml", ".apm/workflow.toml", ".apm/ticket.toml", ".gitignore",
    ])?;

    if crate::git_util::commit(root, "apm: initialize project").is_ok() {
        messages.push("Created initial commit.".to_string());
    }
    Ok(())
}

fn ensure_worktrees_dir(root: &Path, messages: &mut Vec<String>) -> Result<()> {
    if let Ok(config) = crate::config::Config::load(root) {
        let wt_dir = root.join(&config.worktrees.dir);
        if !wt_dir.exists() {
            std::fs::create_dir_all(&wt_dir)?;
            messages.push(format!("Created worktrees dir: {}", wt_dir.display()));
        }
    }
    Ok(())
}

pub fn setup_docker(root: &Path) -> Result<SetupDockerOutput> {
    let mut messages: Vec<String> = Vec::new();
    let apm_dir = root.join(".apm");
    std::fs::create_dir_all(&apm_dir)?;
    let dockerfile_path = apm_dir.join("Dockerfile.apm-worker");
    if dockerfile_path.exists() {
        messages.push(".apm/Dockerfile.apm-worker already exists — not overwriting.".to_string());
        return Ok(SetupDockerOutput { messages });
    }
    std::fs::write(&dockerfile_path, DOCKERFILE_TEMPLATE)?;
    messages.push("Created .apm/Dockerfile.apm-worker".to_string());
    messages.push(String::new());
    messages.push("Next steps:".to_string());
    messages.push("  1. Review .apm/Dockerfile.apm-worker and add project-specific dependencies.".to_string());
    messages.push("  2. Build the image:".to_string());
    messages.push("       docker build -f .apm/Dockerfile.apm-worker -t apm-worker .".to_string());
    messages.push("  3. Add to .apm/config.toml:".to_string());
    messages.push("       [workers]".to_string());
    messages.push("       container = \"apm-worker\"".to_string());
    messages.push("  4. Configure credential lookup (optional, macOS only):".to_string());
    messages.push("       [workers.keychain]".to_string());
    messages.push("       ANTHROPIC_API_KEY = \"anthropic-api-key\"".to_string());
    Ok(SetupDockerOutput { messages })
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
        let mut msgs = Vec::new();
        ensure_gitignore(&path, &mut msgs).unwrap();
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
        let mut msgs = Vec::new();
        ensure_gitignore(&path, &mut msgs).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("node_modules"));
        assert!(contents.contains("tickets/NEXT_ID"));
    }

    #[test]
    fn ensure_gitignore_idempotent() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        let mut msgs = Vec::new();
        ensure_gitignore(&path, &mut msgs).unwrap();
        let before = std::fs::read_to_string(&path).unwrap();
        ensure_gitignore(&path, &mut msgs).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn setup_creates_expected_files() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        setup(tmp.path(), None, None, None).unwrap();

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
        setup(tmp.path(), None, None, None).unwrap();

        let config = std::fs::read_to_string(tmp.path().join(".apm/config.toml")).unwrap();
        let dir_name = tmp.path().file_name().unwrap().to_str().unwrap();
        assert!(config.contains(&format!("name = \"{dir_name}\"")));
        assert!(config.contains("description = \"\""));
    }

    #[test]
    fn setup_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        git_init(tmp.path());
        setup(tmp.path(), None, None, None).unwrap();

        // Write sentinel content to config
        let config_path = tmp.path().join(".apm/config.toml");
        let original = std::fs::read_to_string(&config_path).unwrap();

        setup(tmp.path(), None, None, None).unwrap();
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
        setup(tmp.path(), None, None, None).unwrap();
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
        let mut msgs = Vec::new();
        let action = write_default(&path, "content", "test.toml", &mut msgs).unwrap();
        assert!(matches!(action, WriteAction::Created));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "content");
    }

    #[test]
    fn write_default_unchanged_when_identical() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.toml");
        std::fs::write(&path, "content").unwrap();
        let mut msgs = Vec::new();
        let action = write_default(&path, "content", "test.toml", &mut msgs).unwrap();
        assert!(matches!(action, WriteAction::Unchanged));
    }

    #[test]
    fn write_default_non_tty_writes_init_when_differs() {
        // In test context stdin is not a terminal, so this exercises
        // the non-interactive path: write .init copy.
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.toml");
        std::fs::write(&path, "modified").unwrap();
        let mut msgs = Vec::new();
        let action = write_default(&path, "default", "test.toml", &mut msgs).unwrap();
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
        setup(tmp.path(), None, None, None).unwrap();

        // Modify a file
        let workflow = tmp.path().join(".apm/workflow.toml");
        std::fs::write(&workflow, "# custom workflow\n").unwrap();

        // Second setup (non-tty): should write .init copy
        setup(tmp.path(), None, None, None).unwrap();
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
        setup(tmp.path(), None, None, None).unwrap();

        // Modify config.toml (add a custom section)
        let config_path = tmp.path().join(".apm/config.toml");
        let mut content = std::fs::read_to_string(&config_path).unwrap();
        content.push_str("\n[custom]\nfoo = \"bar\"\n");
        std::fs::write(&config_path, &content).unwrap();

        // Second setup (non-tty): should write config.toml.init
        setup(tmp.path(), None, None, None).unwrap();
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
            ["new", "groomed", "question", "specd", "ammend", "in_design", "ready", "in_progress", "blocked", "implemented", "merge_failed", "closed"]
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
