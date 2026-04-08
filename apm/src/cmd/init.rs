use anyhow::Result;
use serde_json::Value;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(root: &Path, no_claude: bool, migrate: bool, with_docker: bool) -> Result<()> {
    if migrate {
        let msgs = apm_core::init::migrate(root)?;
        for msg in msgs {
            println!("{msg}");
        }
        return Ok(());
    }

    let is_tty = std::io::stdin().is_terminal();

    // Check if git_host is configured
    let has_git_host = {
        let config_path = root.join(".apm/config.toml");
        config_path.exists() && apm_core::config::Config::load(root)
            .map(|cfg| cfg.git_host.provider.is_some())
            .unwrap_or(false)
    };
    let local_toml = root.join(".apm/local.toml");

    let username = if !has_git_host && !local_toml.exists() && is_tty {
        prompt_username()?
    } else {
        String::new()
    };

    let default_name = root.file_name().and_then(|n| n.to_str()).unwrap_or("project").to_string();
    let (name, description) = if is_tty && !root.join(".apm/config.toml").exists() {
        prompt_project_info(&default_name)?
    } else {
        (String::new(), String::new())
    };

    let name_opt = if name.is_empty() { None } else { Some(name.as_str()) };
    let desc_opt = if description.is_empty() { None } else { Some(description.as_str()) };
    let user_opt = if username.is_empty() { None } else { Some(username.as_str()) };

    let setup_out = apm_core::init::setup(root, name_opt, desc_opt, user_opt)?;
    for msg in &setup_out.messages {
        println!("{msg}");
    }

    if with_docker {
        let docker_out = apm_core::init::setup_docker(root)?;
        for msg in &docker_out.messages {
            if msg.is_empty() {
                println!();
            } else {
                println!("{msg}");
            }
        }
    }
    update_claude_settings(root, no_claude)?;
    update_user_claude_settings()?;
    warn_if_settings_untracked(root);
    println!("apm initialized.");
    Ok(())
}

fn prompt_username() -> Result<String> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    print!("Username []: ");
    stdout.flush()?;
    let mut input = String::new();
    stdin.lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_project_info(default_name: &str) -> Result<(String, String)> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    print!("Project name [{}]: ", default_name);
    stdout.flush()?;
    let mut name_input = String::new();
    stdin.lock().read_line(&mut name_input)?;
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
    stdin.lock().read_line(&mut desc_input)?;
    let description = desc_input.trim().to_string();

    Ok((name, description))
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
