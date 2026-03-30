use anyhow::{bail, Result};
use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::io::{self, BufRead, Write};
use std::path::Path;

const SENTINEL: &str = "# --- edit the ticket spec below this line ---";

struct TransitionOption {
    to: String,
    label: String,
    hint: String,
}

pub fn run(root: &Path, id: u32, to: Option<String>, no_aggressive: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    let prefix = format!("ticket/{id:04}-");
    let branches = git::ticket_branches(root)?;
    let branch = branches.into_iter().find(|b| b.starts_with(&prefix));
    if let Some(ref b) = branch {
        if aggressive {
            if let Err(e) = git::fetch_branch(root, b) {
                eprintln!("warning: fetch failed: {e:#}");
            }
        }
    }

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let Some(mut t) = tickets.into_iter().find(|t| t.frontmatter.id == id) else {
        bail!("ticket #{id} not found");
    };

    let current_state = t.frontmatter.state.clone();
    let state_cfg = config.workflow.states.iter().find(|s| s.id == current_state);
    let transitions = manual_transitions(&config, state_cfg, &current_state);

    // Pre-validate --to before opening editor.
    if let Some(ref target) = to {
        let valid = transitions.iter().any(|tr| &tr.to == target)
            || config.workflow.states.iter().any(|s| &s.id == target && s.terminal);
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
    let (spec_body, history_section) = split_body(&t.body);

    // Write temp file: header + sentinel + spec body.
    let header = build_header(id, &t.frontmatter.title, &current_state, &transitions, to.as_deref());
    let tmp_path = {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("apm-review-{id}-{unique}.md"))
    };
    std::fs::write(&tmp_path, format!("{header}\n{SENTINEL}\n\n{spec_body}"))?;

    open_editor(&tmp_path)?;

    let edited_raw = std::fs::read_to_string(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);
    let mut new_spec = extract_spec(&edited_raw);

    // Determine transition.
    let chosen_state = match to {
        Some(s) => Some(s),
        None => prompt_transition(id, &current_state, &transitions)?,
    };

    // Normalise plain bullets → checkboxes in the amendment section when transitioning to ammend.
    if chosen_state.as_deref() == Some("ammend") {
        new_spec = normalise_amendment_checkboxes(new_spec);
    }

    let changed = new_spec.trim_end() != spec_body.trim_end();

    if !changed && chosen_state.is_none() {
        println!("No changes.");
        return Ok(());
    }

    let rel_path = format!(
        "{}/{}",
        config.tickets.dir.to_string_lossy(),
        t.path.file_name().unwrap().to_string_lossy()
    );
    let branch = t.frontmatter.branch.clone()
        .or_else(|| git::branch_name_from_path(&t.path))
        .unwrap_or_else(|| format!("ticket/{id:04}"));

    // Commit the spec edit if the body changed.
    if changed {
        // Splice: trimmed new spec + original history section.
        t.body = format!("{}{}", new_spec.trim_end(), history_section);
        t.frontmatter.updated_at = Some(Utc::now());
        let content = t.serialize()?;
        git::commit_to_branch(root, &branch, &rel_path, &content,
            &format!("ticket({id}): review edit"))?;
        println!("#{id}: spec updated");
    }

    // Apply the state transition (state::run re-reads from git, handles history etc.).
    if let Some(target) = chosen_state {
        super::state::run(root, id, target, false)?;
    }

    Ok(())
}

/// Extract the editable spec from the saved temp file.
/// Everything after the sentinel line (or after leading `# ` comment lines
/// if the sentinel was deleted) is the spec content.
fn extract_spec(content: &str) -> String {
    if let Some(idx) = content.find(SENTINEL) {
        let after = &content[idx + SENTINEL.len()..];
        after.trim_start_matches('\n').to_string()
    } else {
        // Sentinel was deleted — strip leading comment lines as fallback.
        let mut lines = content.lines().peekable();
        let mut out = Vec::new();
        let mut past_header = false;
        for line in lines.by_ref() {
            if !past_header && (line == "#" || line.starts_with("# ")) {
                continue;
            }
            past_header = true;
            out.push(line);
        }
        out.join("\n")
    }
}

/// Split ticket body into (spec_part, history_section).
/// `history_section` starts with `\n## History` so it can be spliced back directly.
fn split_body(body: &str) -> (String, String) {
    if let Some(idx) = body.find("\n## History") {
        (body[..idx].to_string(), body[idx..].to_string())
    } else if body.starts_with("## History") {
        (String::new(), body.to_string())
    } else {
        (body.to_string(), String::new())
    }
}

/// Returns the manual (non-auto) transitions available from the current state.
fn manual_transitions(
    config: &Config,
    state_cfg: Option<&apm_core::config::StateConfig>,
    current_state: &str,
) -> Vec<TransitionOption> {
    let terminal_ids: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();

    if let Some(sc) = state_cfg {
        if !sc.transitions.is_empty() {
            return sc.transitions.iter()
                .filter(|tr| {
                    // Include manual and command:* triggers; exclude event:* auto-triggers.
                    !tr.trigger.starts_with("event:")
                })
                .map(|tr| TransitionOption {
                    to: tr.to.clone(),
                    label: tr.label.clone(),
                    hint: tr.hint.clone(),
                })
                .collect();
        }
    }

    // No explicit transitions: all non-terminal, non-current states are valid.
    config.workflow.states.iter()
        .filter(|s| s.id != current_state && !terminal_ids.contains(&s.id.as_str()))
        .map(|s| TransitionOption { to: s.id.clone(), label: s.label.clone(), hint: String::new() })
        .collect()
}

fn build_header(
    id: u32,
    title: &str,
    state: &str,
    transitions: &[TransitionOption],
    fixed_to: Option<&str>,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# Reviewing ticket #{id} · state: {state}"));
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

fn open_editor(path: &Path) -> Result<()> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let mut parts = editor.split_whitespace();
    let bin = parts.next().unwrap();
    let status = std::process::Command::new(bin)
        .args(parts)
        .arg(path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("could not launch editor '{editor}': {e}"))?;

    if !status.success() {
        bail!("editor exited with non-zero status");
    }
    Ok(())
}

/// Convert plain `- ` bullets in `### Amendment requests` to `- [ ] ` checkboxes.
/// Lines already formatted as `- [ ]` or `- [x]` are left unchanged.
/// Only lines inside the section (up to the next `##` heading) are affected.
fn normalise_amendment_checkboxes(spec: String) -> String {
    const SECTION: &str = "### Amendment requests";

    let parts: Vec<&str> = spec.split('\n').collect();
    let Some(sec_pos) = parts.iter().position(|l| *l == SECTION) else {
        return spec;
    };

    let mut result: Vec<String> = Vec::with_capacity(parts.len());
    let mut in_section = false;

    for (i, line) in parts.iter().enumerate() {
        if i < sec_pos {
            result.push((*line).to_string());
        } else if i == sec_pos {
            in_section = true;
            result.push((*line).to_string());
        } else if in_section && line.starts_with("##") {
            in_section = false;
            result.push((*line).to_string());
        } else if in_section
            && line.starts_with("- ")
            && !line.starts_with("- [ ]")
            && !line.starts_with("- [x]")
            && !line.starts_with("- [X]")
        {
            result.push(format!("- [ ]{}", &line[1..]));
        } else {
            result.push((*line).to_string());
        }
    }

    result.join("\n")
}

fn prompt_transition(
    id: u32,
    current_state: &str,
    transitions: &[TransitionOption],
) -> Result<Option<String>> {
    if transitions.is_empty() {
        return Ok(None);
    }

    let options: Vec<&str> = transitions.iter().map(|t| t.to.as_str()).collect();
    print!(
        "#{id} {current_state} → ?   {} / [keep]  > ",
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
