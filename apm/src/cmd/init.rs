use anyhow::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub fn run(root: &Path) -> Result<()> {
    let tickets_dir = root.join("tickets");
    if !tickets_dir.exists() {
        std::fs::create_dir_all(&tickets_dir)?;
        println!("Created tickets/");
    }
    let next_id = tickets_dir.join("NEXT_ID");
    if !next_id.exists() {
        std::fs::write(&next_id, "1\n")?;
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
    let gitignore = root.join(".gitignore");
    ensure_gitignore(&gitignore)?;
    let git_dir = root.join(".git");
    write_hooks(&git_dir)?;
    println!("apm initialized.");
    Ok(())
}

fn default_config(name: &str) -> String {
    format!(
        r#"[project]
name = "{name}"

[tickets]
dir = "tickets"

[agents]
max_concurrent = 3
actionable_states = ["new", "ammend", "ready"]

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0
"#
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
        "#!/bin/sh\n# Fires event:branch_push_first → ready → in_progress\ncommand -v apm >/dev/null 2>&1 && apm _hook pre-push \"$@\" || true\n",
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
