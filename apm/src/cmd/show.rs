use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, no_aggressive: bool, edit: bool) -> Result<()> {
    let config = Config::load(root)?;

    let branches = git::ticket_branches(root)?;
    let branch = git::resolve_ticket_branch(&branches, id_arg)?;

    let aggressive = config.sync.aggressive && !no_aggressive;
    if aggressive {
        if let Err(e) = git::fetch_branch(root, &branch) {
            eprintln!("warning: fetch failed: {e:#}");
        }
    }

    let suffix = branch.trim_start_matches("ticket/");
    let filename = format!("{suffix}.md");
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);
    let dummy_path = root.join(&rel_path);

    let content = git::read_from_branch(root, &branch, &rel_path)?;
    let t = ticket::Ticket::parse(&dummy_path, &content)?;

    if !edit {
        let fm = &t.frontmatter;
        println!("{} — {}", fm.id, fm.title);
        println!("state:    {}", fm.state);
        println!("priority: {}  effort: {}  risk: {}", fm.priority, fm.effort, fm.risk);
        if let Some(b) = &fm.branch { println!("branch:   {b}"); }
        println!();
        print!("{}", t.body);
        return Ok(());
    }

    let id = &t.frontmatter.id;
    let tmp_path = std::env::temp_dir().join(format!("apm-{id}.md"));
    std::fs::write(&tmp_path, &content)?;

    let editor = std::env::var("VISUAL")
        .ok()
        .filter(|e| !e.is_empty())
        .or_else(|| std::env::var("EDITOR").ok().filter(|e| !e.is_empty()))
        .unwrap_or_else(|| "vi".to_string());

    let mut parts = editor.split_whitespace();
    let bin = parts.next().unwrap();
    let status = std::process::Command::new(bin)
        .args(parts)
        .arg(&tmp_path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("could not launch editor '{editor}': {e}"))?;

    if !status.success() {
        let _ = std::fs::remove_file(&tmp_path);
        bail!("editor exited with non-zero status");
    }

    let edited = std::fs::read_to_string(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);

    if edited != content {
        git::commit_to_branch(root, &branch, &rel_path, &edited, &format!("ticket({id}): edit"))?;
    }

    Ok(())
}
