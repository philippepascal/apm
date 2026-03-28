use anyhow::Result;
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::collections::HashSet;
use std::path::Path;

pub fn run(root: &Path, fix: bool) -> Result<()> {
    let config = Config::load(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;

    let valid_states: HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();
    let terminal: HashSet<&str> = config.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();

    let merged = git::merged_into_main(root, &config.project.default_branch).unwrap_or_default();
    let merged_set: HashSet<&str> = merged.iter().map(|s| s.as_str()).collect();

    let in_progress_states: HashSet<&str> = ["in_progress", "implemented", "accepted"].iter().copied().collect();

    let mut issues: Vec<String> = Vec::new();

    for t in &tickets {
        let fm = &t.frontmatter;

        // Skip terminal-state tickets.
        if terminal.contains(fm.state.as_str()) { continue; }

        let prefix = format!("#{} [{}]", fm.id, fm.state);

        // State value not in config.
        if !valid_states.is_empty() && !valid_states.contains(fm.state.as_str()) {
            issues.push(format!("{prefix}: unknown state {:?}", fm.state));
        }

        // Frontmatter id doesn't match filename numeric prefix.
        if let Some(name) = t.path.file_name().and_then(|n| n.to_str()) {
            let expected_prefix = format!("{:04}", fm.id);
            if !name.starts_with(&expected_prefix) {
                issues.push(format!("{prefix}: id {} does not match filename {name}", fm.id));
            }
        }

        // in_progress/implemented/accepted with no branch.
        if in_progress_states.contains(fm.state.as_str()) && fm.branch.is_none() {
            issues.push(format!("{prefix}: state requires branch but none set"));
        }

        // Branch merged but ticket not yet accepted.
        if let Some(branch) = &fm.branch {
            if (fm.state == "in_progress" || fm.state == "implemented")
                && merged_set.contains(branch.as_str())
            {
                issues.push(format!("{prefix}: branch {branch} is merged but ticket not accepted"));
            }
        }

        // Agent set but state is not in in_progress/implemented/accepted.
        if fm.agent.is_some() && !in_progress_states.contains(fm.state.as_str()) {
            issues.push(format!("{prefix}: agent is set but state is not in_progress/implemented/accepted"));
        }

        // Missing ## Spec section.
        if !t.body.contains("## Spec") {
            issues.push(format!("{prefix}: missing ## Spec section"));
        }

        // Missing ## History section.
        if !t.body.contains("## History") {
            issues.push(format!("{prefix}: missing ## History section"));
        }
    }

    if issues.is_empty() {
        println!("verify: no issues found");
        return Ok(());
    }

    for issue in &issues {
        println!("{issue}");
    }

    if fix {
        apply_fixes(root, &config, &tickets, &merged_set)?;
    }

    std::process::exit(1);
}

fn apply_fixes(
    root: &Path,
    config: &Config,
    tickets: &[apm_core::ticket::Ticket],
    merged_set: &HashSet<&str>,
) -> Result<()> {
    for t in tickets {
        let fm = &t.frontmatter;
        let Some(branch) = &fm.branch else { continue };
        if (fm.state == "in_progress" || fm.state == "implemented")
            && merged_set.contains(branch.as_str())
        {
            let now = Utc::now();
            let mut t = t.clone();
            let old_state = fm.state.clone();
            t.frontmatter.state = "accepted".into();
            t.frontmatter.updated_at = Some(now);
            let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
            crate::cmd::state::append_history(&mut t.body, &old_state, "accepted", &when, "verify --fix");
            let content = t.serialize()?;
            let id = fm.id;
            let filename = t.path.file_name().unwrap().to_string_lossy().to_string();
            let rel_path = format!("{}/{filename}", config.tickets.dir.to_string_lossy());
            match git::commit_to_branch(root, branch, &rel_path, &content,
                &format!("ticket({id}): {} → accepted (verify --fix)", fm.state)) {
                Ok(_) => println!("  fixed #{id}: {} → accepted", fm.state),
                Err(e) => eprintln!("  warning: could not fix #{id}: {e:#}"),
            }
        }
    }
    Ok(())
}
