use anyhow::Result;
use apm_core::{config::{resolve_identity, resolve_caller_name}, epic, ticket};
use std::path::Path;
use crate::ctx::CmdContext;

#[allow(clippy::too_many_arguments)]
// Each argument maps to a distinct CLI flag.
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
        anyhow::bail!("side tickets are disabled in .apm/config.toml (agents.side_tickets = false)");
    }

    let author = resolve_identity(root);
    let actor = resolve_caller_name();

    let (epic_id, target_branch, base_branch) = if let Some(ref id) = epic {
        match epic::find_epic_branch(root, id) {
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

    if let Some(ref dep_ids) = depends_on_parsed {
        if !dep_ids.is_empty() {
            let all_tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;
            let strategy = apm_core::validate::active_completion_strategy(&config);
            apm_core::validate::check_depends_on_rules(
                &strategy,
                epic_id.as_deref(),
                target_branch.as_deref(),
                dep_ids,
                &all_tickets,
                &config.project.default_branch,
            )?;
        }
    }

    let section_sets: Vec<(String, String)> = sections.into_iter().zip(sets).collect();
    let mut warnings = Vec::new();
    let t = ticket::create(root, &config, title, author, actor, context, context_section, aggressive, section_sets, epic_id, target_branch, depends_on_parsed, base_branch, &mut warnings)?;
    for w in &warnings {
        eprintln!("{w}");
    }
    let id = &t.frontmatter.id;
    let branch = t.frontmatter.branch.as_deref().unwrap_or("");
    let filename = t.path.file_name().unwrap().to_string_lossy();
    let rel_path = format!("{}/{}", config.tickets.dir.to_string_lossy(), filename);

    println!("Created ticket {id}: {filename} (branch: {branch})");

    if !no_edit {
        open_editor(root, branch, &rel_path)?;
    }

    Ok(())
}

fn open_editor(root: &Path, branch: &str, rel_path: &str) -> Result<()> {
    let content = apm_core::git_util::read_from_branch(root, branch, rel_path)?;

    let fname = std::path::Path::new(rel_path)
        .file_name().unwrap().to_string_lossy().into_owned();
    let tmp_path = std::env::temp_dir()
        .join(format!("apm-{}-{}", std::process::id(), fname));
    std::fs::write(&tmp_path, &content)?;

    crate::editor::open(&tmp_path)?;

    let new_content = std::fs::read_to_string(&tmp_path)?;

    apm_core::git_util::commit_to_branch(root, branch, rel_path, &new_content, "write spec")?;

    let _ = std::fs::remove_file(&tmp_path);

    Ok(())
}
