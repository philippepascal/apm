use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, no_aggressive: bool, edit: bool) -> Result<()> {
    let config = Config::load(root)?;

    let branches = git::ticket_branches(root)?;
    let branch_result = git::resolve_ticket_branch(&branches, id_arg);

    match branch_result {
        Ok(branch) => {
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

            show_ticket(&t, &content, root, &branch, &rel_path, edit)
        }
        Err(_) => {
            // Fallback: search tickets/ on default branch, then archive_dir if set.
            let default_branch = &config.project.default_branch;
            let prefixes = ticket::id_arg_prefixes(id_arg)?;

            if let Some((rel_path, content)) = find_in_dir(
                root,
                default_branch,
                &config.tickets.dir.to_string_lossy(),
                &prefixes,
            ) {
                let dummy_path = root.join(&rel_path);
                let t = ticket::Ticket::parse(&dummy_path, &content)?;
                return show_ticket_readonly(&t, &content, edit);
            }

            if let Some(archive_dir) = &config.tickets.archive_dir {
                if let Some((rel_path, content)) = find_in_dir(
                    root,
                    default_branch,
                    &archive_dir.to_string_lossy(),
                    &prefixes,
                ) {
                    let dummy_path = root.join(&rel_path);
                    let t = ticket::Ticket::parse(&dummy_path, &content)?;
                    return show_ticket_readonly(&t, &content, edit);
                }
            }

            bail!("no ticket matches '{id_arg}'")
        }
    }
}

fn find_in_dir(
    root: &Path,
    branch: &str,
    dir: &str,
    prefixes: &[String],
) -> Option<(String, String)> {
    let files = git::list_files_on_branch(root, branch, dir).ok()?;
    for rel_path in files {
        let filename = rel_path.split('/').last().unwrap_or("");
        let file_id = filename.split('-').next().unwrap_or("");
        if prefixes.iter().any(|p| file_id.starts_with(p.as_str())) {
            if let Ok(content) = git::read_from_branch(root, branch, &rel_path) {
                return Some((rel_path, content));
            }
        }
    }
    None
}

fn show_ticket(
    t: &ticket::Ticket,
    content: &str,
    root: &Path,
    branch: &str,
    rel_path: &str,
    edit: bool,
) -> Result<()> {
    if !edit {
        print_ticket(t);
        return Ok(());
    }

    let id = &t.frontmatter.id;
    let tmp_path = std::env::temp_dir().join(format!("apm-{id}.md"));
    std::fs::write(&tmp_path, content)?;

    if let Err(e) = crate::editor::open(&tmp_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e);
    }

    let edited = std::fs::read_to_string(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);

    if edited != content {
        git::commit_to_branch(root, branch, rel_path, &edited, &format!("ticket({id}): edit"))?;
    }

    Ok(())
}

fn show_ticket_readonly(t: &ticket::Ticket, _content: &str, edit: bool) -> Result<()> {
    if edit {
        bail!("--edit is not supported for archived tickets (no active branch)");
    }
    print_ticket(t);
    Ok(())
}

fn print_ticket(t: &ticket::Ticket) {
    let fm = &t.frontmatter;
    println!("{} — {}", fm.id, fm.title);
    println!("state:    {}", fm.state);
    println!("priority: {}  effort: {}  risk: {}", fm.priority, fm.effort, fm.risk);
    if let Some(b) = &fm.branch { println!("branch:   {b}"); }
    if let Some(e) = &fm.epic { println!("epic:         {e}"); }
    if let Some(tb) = &fm.target_branch { println!("target_branch: {tb}"); }
    if let Some(deps) = &fm.depends_on {
        if !deps.is_empty() {
            println!("depends_on:   {}", deps.join(", "));
        }
    }
    if let Some(o) = &fm.owner {
        println!("owner:        {o}");
    }
    println!();
    print!("{}", t.body);
}
