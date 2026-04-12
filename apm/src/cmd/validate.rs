use anyhow::Result;
pub use apm_core::validate::validate_config;
pub use apm_core::validate::validate_warnings;
use apm_core::{config::Config, git, ticket, ticket_fmt};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use crate::ctx::CmdContext;

#[derive(Debug, Serialize)]
struct Issue {
    kind: String,
    subject: String,
    message: String,
}

pub fn run(root: &Path, fix: bool, json: bool, config_only: bool, no_aggressive: bool) -> Result<()> {
    let config_errors;
    let config_warnings;
    let mut ticket_issues: Vec<Issue> = Vec::new();
    let mut tickets_checked = 0usize;
    let config: Config;

    if config_only {
        config = CmdContext::load_config_only(root)?;
        config_errors = validate_config(&config, root);
        config_warnings = validate_warnings(&config);
    } else {
        let ctx = CmdContext::load(root, no_aggressive)?;
        config = ctx.config;
        config_errors = validate_config(&config, root);
        config_warnings = validate_warnings(&config);
        tickets_checked = ctx.tickets.len();

        let tickets = ctx.tickets;

        let state_ids: HashSet<&str> = config.workflow.states.iter()
            .map(|s| s.id.as_str())
            .collect();

        let mut branch_fixes: Vec<(ticket::Ticket, String, String)> = Vec::new();

        for t in &tickets {
            let fm = &t.frontmatter;
            let ticket_subject = format!("#{}", fm.id);

            if !state_ids.is_empty() && fm.state != "closed" && !state_ids.contains(fm.state.as_str()) {
                ticket_issues.push(Issue {
                    kind: "ticket".into(),
                    subject: ticket_subject.clone(),
                    message: format!(
                        "ticket #{} has unknown state '{}'",
                        fm.id, fm.state
                    ),
                });
            }

            if let Some(branch) = &fm.branch {
                let canonical = ticket_fmt::branch_name_from_path(&t.path);
                if let Some(expected) = canonical {
                    if branch != &expected {
                        ticket_issues.push(Issue {
                            kind: "ticket".into(),
                            subject: ticket_subject.clone(),
                            message: format!(
                                "ticket #{} branch field '{}' does not match expected '{}'",
                                fm.id, branch, expected
                            ),
                        });
                        if fix {
                            branch_fixes.push((t.clone(), expected, branch.clone()));
                        }
                    }
                }
            }
        }

        if fix {
            apply_branch_fixes(root, &config, branch_fixes)?;
        }
    }

    let has_errors = !config_errors.is_empty() || !ticket_issues.is_empty();

    if json {
        let out = serde_json::json!({
            "tickets_checked": tickets_checked,
            "config_errors": config_errors,
            "warnings": config_warnings,
            "errors": ticket_issues,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for e in &config_errors {
            eprintln!("{e}");
        }
        for w in &config_warnings {
            eprintln!("warning: {w}");
        }
        for e in &ticket_issues {
            println!("error [{}] {}: {}", e.kind, e.subject, e.message);
        }
        println!(
            "{} tickets checked, {} config errors, {} warnings, {} ticket errors",
            tickets_checked,
            config_errors.len(),
            config_warnings.len(),
            ticket_issues.len(),
        );
    }

    if has_errors {
        anyhow::bail!(
            "{} config errors, {} ticket errors",
            config_errors.len(),
            ticket_issues.len()
        );
    }

    Ok(())
}

fn apply_branch_fixes(
    root: &Path,
    config: &Config,
    fixes: Vec<(ticket::Ticket, String, String)>,
) -> Result<()> {
    for (mut t, expected_branch, _old_branch) in fixes {
        let id = t.frontmatter.id.clone();
        t.frontmatter.branch = Some(expected_branch.clone());
        let content = t.serialize()?;
        let filename = t.path.file_name().unwrap().to_string_lossy().to_string();
        let rel_path = format!("{}/{filename}", config.tickets.dir.to_string_lossy());
        match git::commit_to_branch(
            root,
            &expected_branch,
            &rel_path,
            &content,
            &format!("ticket({id}): fix branch field (validate --fix)"),
        ) {
            Ok(_) => println!("  fixed {id}: branch -> {expected_branch}"),
            Err(e) => eprintln!("  warning: could not fix {id}: {e:#}"),
        }
    }
    Ok(())
}
