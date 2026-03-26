use anyhow::Result;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let root = crate::repo_root()?;
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
