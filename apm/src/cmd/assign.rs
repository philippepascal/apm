use anyhow::{bail, Result};
use apm_core::{config::{Config, LocalConfig}, git, ticket};
use chrono::Utc;
use std::io::{self, Write, BufRead};
use std::path::Path;

pub fn run(root: &Path, id_arg: &str, username: &str, no_aggressive: bool, force: bool) -> Result<()> {
    run_inner(root, id_arg, username, no_aggressive, force, None)
}

pub fn run_inner(root: &Path, id_arg: &str, username: &str, no_aggressive: bool, force: bool, confirm_override: Option<bool>) -> Result<()> {
    let config = Config::load(root)?;
    let local = LocalConfig::load(root);
    apm_core::validate::validate_owner(&config, &local, username)?;
    let aggressive = config.sync.aggressive && !no_aggressive;
    let mut tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let id = ticket::resolve_id_in_slice(&tickets, id_arg)?;

    if aggressive {
        let branches = git::ticket_branches(root).unwrap_or_default();
        if let Some(b) = branches.iter().find(|b| {
            b.strip_prefix("ticket/")
                .and_then(|s| s.split('-').next())
                .map(|bid| bid == id.as_str())
                .unwrap_or(false)
        }) {
            if let Err(e) = git::fetch_branch(root, b) {
                eprintln!("warning: fetch failed: {e:#}");
            }
        }
    }

    let Some(t) = tickets.iter_mut().find(|t| t.frontmatter.id == id) else {
        bail!("ticket {id:?} not found");
    };

    if force {
        let is_terminal = config.workflow.states.iter()
            .find(|s| s.id == t.frontmatter.state)
            .map(|s| s.terminal)
            .unwrap_or(false);
        if is_terminal {
            bail!("cannot change owner of a closed ticket");
        }
        if let Some(current_owner) = &t.frontmatter.owner.clone() {
            let confirmed = match confirm_override {
                Some(b) => b,
                None => {
                    print!("Ticket {id} is currently owned by {current_owner}. Reassign to {username}? [y/N] ");
                    io::stdout().flush()?;
                    let mut line = String::new();
                    io::stdin().lock().read_line(&mut line)?;
                    line.trim().eq_ignore_ascii_case("y")
                }
            };
            if !confirmed {
                println!("aborted");
                return Ok(());
            }
        }
    } else {
        ticket::check_owner(root, t)?;
    }

    ticket::set_field(&mut t.frontmatter, "owner", username)?;
    t.frontmatter.updated_at = Some(Utc::now());

    let content = t.serialize()?;
    let rel_path = format!(
        "{}/{}",
        config.tickets.dir.to_string_lossy(),
        t.path.file_name().unwrap().to_string_lossy()
    );
    let branch = t
        .frontmatter
        .branch
        .clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));

    let commit_msg = if username == "-" {
        format!("ticket({id}): assign owner = -")
    } else {
        format!("ticket({id}): assign owner = {username}")
    };

    git::commit_to_branch(root, &branch, &rel_path, &content, &commit_msg)?;

    if aggressive {
        if let Err(e) = git::push_branch(root, &branch) {
            eprintln!("warning: push failed: {e:#}");
        }
    }

    if username == "-" {
        println!("{id}: owner cleared");
    } else {
        println!("{id}: owner = {username}");
    }
    Ok(())
}
