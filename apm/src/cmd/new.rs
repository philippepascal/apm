use anyhow::Result;
use apm_core::{
    config::Config,
    git,
    ticket::{slugify, Frontmatter, Ticket},
};
use chrono::Utc;
use std::path::Path;

pub fn run(root: &Path, title: String, no_edit: bool, side_note: bool, context: Option<String>, no_aggressive: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;
    if side_note && !config.agents.side_tickets {
        anyhow::bail!("side tickets are disabled in apm.toml (agents.side_tickets = false)");
    }
    let tickets_dir = root.join(&config.tickets.dir);
    std::fs::create_dir_all(&tickets_dir)?;

    let id = git::next_ticket_id(root, &tickets_dir)?;
    let slug = slugify(&title);
    let filename = format!("{id:04}-{slug}.md");
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);
    let branch = format!("ticket/{id:04}-{slug}");
    let now = Utc::now();
    let author = std::env::var("APM_AGENT_NAME")
        .ok()
        .unwrap_or_else(|| "apm".into());
    let fm = Frontmatter {
        id,
        title: title.clone(),
        state: "new".into(),
        priority: 0,
        effort: 0,
        risk: 0,
        author: Some(author.clone()),
        supervisor: None,
        agent: None,
        branch: Some(branch.clone()),
        created_at: Some(now),
        updated_at: Some(now),
        focus_section: None,
    };
    let when = now.format("%Y-%m-%dT%H:%MZ");
    let problem_section = match &context {
        Some(ctx) => format!("### Problem\n\n{ctx}\n\n"),
        None => "### Problem\n\n".to_string(),
    };
    let body = format!(
        "## Spec\n\n{problem_section}### Acceptance criteria\n\n### Out of scope\n\n### Approach\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|\n| {when} | — | new | {author} |\n"
    );
    let path = tickets_dir.join(&filename);
    let t = Ticket { frontmatter: fm, body, path };
    let content = t.serialize()?;

    git::commit_to_branch(
        root,
        &branch,
        &rel_path,
        &content,
        &format!("ticket({id}): create {title}"),
    )?;

    if aggressive {
        if let Err(e) = git::push_branch(root, &branch) {
            eprintln!("warning: push failed: {e:#}");
        }
    }

    println!("Created ticket #{id}: {filename} (branch: {branch})");

    if !no_edit {
        open_editor(root, &config, &branch, &rel_path)?;
    }

    Ok(())
}

fn open_editor(root: &Path, _config: &Config, branch: &str, rel_path: &str) -> Result<()> {
    let editor = match std::env::var("EDITOR") {
        Ok(e) if !e.is_empty() => e,
        _ => {
            eprintln!("warning: $EDITOR is not set, skipping editor open");
            return Ok(());
        }
    };

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
    let status = std::process::Command::new(&editor)
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
