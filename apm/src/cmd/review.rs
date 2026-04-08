use anyhow::{bail, Result};
use apm_core::{git, review as core_review, ticket};
use chrono::Utc;
use std::io::{self, BufRead, Write};
use std::path::Path;
use crate::ctx::CmdContext;

struct TransitionOption {
    to: String,
    label: String,
    hint: String,
}

pub fn run(root: &Path, id_arg: &str, to: Option<String>, no_aggressive: bool) -> Result<()> {
    let ctx = CmdContext::load(root, no_aggressive)?;
    let id = ticket::resolve_id_in_slice(&ctx.tickets, id_arg)?;
    let Some(mut t) = ctx.tickets.into_iter().find(|t| t.frontmatter.id == id) else {
        bail!("ticket {id:?} not found");
    };

    let current_state = t.frontmatter.state.clone();
    let raw_transitions = core_review::available_transitions(&ctx.config, &current_state);
    let transitions: Vec<TransitionOption> = raw_transitions.into_iter()
        .map(|(to, label, hint)| TransitionOption { to, label, hint })
        .collect();

    // Pre-validate --to before opening editor.
    if let Some(ref target) = to {
        let valid = transitions.iter().any(|tr| &tr.to == target)
            || ctx.config.workflow.states.iter().any(|s| &s.id == target && s.terminal);
        if !valid {
            let options: Vec<&str> = transitions.iter().map(|t| t.to.as_str()).collect();
            bail!(
                "transition '{target}' is not available from '{current_state}'\n\
                 Valid options: {}",
                if options.is_empty() { "(none defined)".to_string() } else { options.join(", ") }
            );
        }
    }

    // Split body into editable spec and preserved history.
    let (spec_body, history_section) = core_review::split_body(&t.body);

    // Write temp file: header + sentinel + spec body.
    let header = build_header(&id, &t.frontmatter.title, &current_state, &transitions, to.as_deref());
    let tmp_path = {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("apm-review-{id}-{unique}.md"))
    };
    std::fs::write(&tmp_path, format!("{header}\n{}\n\n{spec_body}", core_review::SENTINEL))?;

    crate::editor::open(&tmp_path)?;

    let edited_raw = std::fs::read_to_string(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);
    let mut new_spec = core_review::extract_spec(&edited_raw);

    // Determine transition.
    let chosen_state = match to {
        Some(s) => Some(s),
        None => prompt_transition(&id, &current_state, &transitions)?,
    };

    // Normalise plain bullets → checkboxes in the amendment section when transitioning to ammend.
    if chosen_state.as_deref() == Some("ammend") {
        new_spec = core_review::normalize_amendments(new_spec);
    }

    let changed = new_spec.trim_end() != spec_body.trim_end();

    if !changed && chosen_state.is_none() {
        println!("No changes.");
        return Ok(());
    }

    let rel_path = format!(
        "{}/{}",
        ctx.config.tickets.dir.to_string_lossy(),
        t.path.file_name().unwrap().to_string_lossy()
    );
    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id}"));

    // Commit the spec edit if the body changed.
    if changed {
        // Splice: trimmed new spec + original history section.
        t.body = core_review::apply_review(&new_spec, &history_section);
        t.frontmatter.updated_at = Some(Utc::now());
        let content = t.serialize()?;
        git::commit_to_branch(root, &branch, &rel_path, &content,
            &format!("ticket({id}): review edit"))?;
        if ctx.aggressive {
            if let Err(e) = git::push_branch(root, &branch) {
                eprintln!("warning: push failed: {e:#}");
            }
        }
        println!("{id}: spec updated");
    }

    // Apply the state transition (state::run re-reads from git, handles history etc.).
    if let Some(target) = chosen_state {
        super::state::run(root, &id, target, false, false)?;
    }

    Ok(())
}

fn build_header(
    id: &str,
    title: &str,
    state: &str,
    transitions: &[TransitionOption],
    fixed_to: Option<&str>,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# Reviewing ticket {id} · state: {state}"));
    lines.push(format!("# \"{title}\""));
    lines.push("#".to_string());

    if let Some(target) = fixed_to {
        lines.push(format!("# Will transition to: {target}"));
    } else if !transitions.is_empty() {
        lines.push("# Transitions (choose after saving):".to_string());
        for tr in transitions {
            if tr.label.is_empty() {
                lines.push(format!("#   {}", tr.to));
            } else {
                lines.push(format!("#   {} — {}", tr.to, tr.label));
            }
            if !tr.hint.is_empty() {
                lines.push(format!("#       → {}", tr.hint));
            }
        }
    } else {
        lines.push("# No transitions defined for this state.".to_string());
    }

    lines.push("#".to_string());
    lines.push("# Lines starting with \"# \" are ignored. Do not delete the dashed line below.".to_string());
    lines.join("\n")
}

fn prompt_transition(
    id: &str,
    current_state: &str,
    transitions: &[TransitionOption],
) -> Result<Option<String>> {
    if transitions.is_empty() {
        return Ok(None);
    }

    let options: Vec<&str> = transitions.iter().map(|t| t.to.as_str()).collect();
    print!(
        "{id} {current_state} → ?   {} / [keep]  > ",
        options.join(" / ")
    );
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    let input = line.trim().to_lowercase();

    if input.is_empty() || input == "keep" || input == "k" {
        return Ok(None);
    }

    // Exact match first, then prefix match.
    if let Some(tr) = transitions.iter().find(|t| t.to.to_lowercase() == input) {
        return Ok(Some(tr.to.clone()));
    }
    let matches: Vec<&TransitionOption> = transitions.iter()
        .filter(|t| t.to.to_lowercase().starts_with(&input))
        .collect();
    match matches.len() {
        0 => bail!("unknown transition '{input}' — valid: {}", options.join(", ")),
        1 => Ok(Some(matches[0].to.clone())),
        _ => bail!("ambiguous: '{}' — be more specific", input),
    }
}
