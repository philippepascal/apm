use anyhow::Result;
use apm_core::{config::{Config, resolve_identity}, git, ticket};
use std::path::Path;
use crate::ctx::CmdContext;

pub fn run(root: &Path, title: String, no_edit: bool, side_note: bool, context: Option<String>, context_section: Option<String>, no_aggressive: bool, sections: Vec<String>, sets: Vec<String>, epic: Option<String>, depends_on: Vec<String>) -> Result<()> {
    let config = CmdContext::load_config_only(root)?;

    if context_section.is_some() && context.is_none() {
        anyhow::bail!("--context-section requires --context");
    }

    if !sets.is_empty() && sections.is_empty() {
        anyhow::bail!("--set requires --section");
    }
    if sections.len() != sets.len() {
        anyhow::bail!(
            "--section and --set must be paired: {} --section flag(s) but {} --set flag(s)",
            sections.len(),
            sets.len()
        );
    }

    if !config.ticket.sections.is_empty() {
        for name in &sections {
            if !config.ticket.sections.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
                anyhow::bail!("unknown section {:?}; not defined in [ticket.sections]", name);
            }
        }
    }

    let aggressive = config.sync.aggressive && !no_aggressive;
    if side_note && !config.agents.side_tickets {
        anyhow::bail!("side tickets are disabled in apm.toml (agents.side_tickets = false)");
    }

    let author = resolve_identity(root);

    let (epic_id, target_branch, base_branch) = if let Some(ref id) = epic {
        match git::find_epic_branch(root, id) {
            Some(branch) => (Some(id.clone()), Some(branch.clone()), Some(branch)),
            None => anyhow::bail!("No epic branch found for id '{id}'"),
        }
    } else {
        (None, None, None)
    };

    let depends_on_parsed: Option<Vec<String>> = if depends_on.is_empty() {
        None
    } else {
        Some(
            depends_on
                .iter()
                .flat_map(|s| s.split(','))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        )
    };

    let section_sets: Vec<(String, String)> = sections.into_iter().zip(sets).collect();
    let t = ticket::create(root, &config, title, author, context, context_section, aggressive, section_sets, epic_id, target_branch, depends_on_parsed, base_branch)?;
    let id = &t.frontmatter.id;
    let branch = t.frontmatter.branch.as_deref().unwrap_or("");
    let filename = t.path.file_name().unwrap().to_string_lossy();
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);

    println!("Created ticket {id}: {filename} (branch: {branch})");

    if !no_edit {
        open_editor(root, &config, branch, &rel_path)?;
    }

    Ok(())
}

fn open_editor(root: &Path, _config: &Config, branch: &str, rel_path: &str) -> Result<()> {
    let editor = std::env::var("VISUAL")
        .ok()
        .filter(|e| !e.is_empty())
        .or_else(|| std::env::var("EDITOR").ok().filter(|e| !e.is_empty()))
        .unwrap_or_else(|| "vi".to_string());

    // Check out the ticket branch, open editor, commit result, return to previous branch.
    let prev_branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "main".to_string());

    let _ = std::process::Command::new("git")
        .args(["checkout", branch])
        .current_dir(root)
        .status();

    let file_path = root.join(rel_path);
    let mut parts = editor.split_whitespace();
    let bin = parts.next().unwrap();
    let status = std::process::Command::new(bin)
        .args(parts)
        .arg(&file_path)
        .status();

    // Commit whatever the user wrote, even if editor exited non-zero.
    let _ = std::process::Command::new("git")
        .args(["-c", "commit.gpgsign=false", "add", rel_path])
        .current_dir(root)
        .status();
    let _ = std::process::Command::new("git")
        .args(["-c", "commit.gpgsign=false", "commit", "--allow-empty", "-m", "write spec"])
        .current_dir(root)
        .status();

    let _ = std::process::Command::new("git")
        .args(["checkout", &prev_branch])
        .current_dir(root)
        .status();

    if let Ok(s) = status {
        if !s.success() {
            eprintln!("warning: editor exited with non-zero status");
        }
    }

    Ok(())
}
