use anyhow::{Context, Result};
pub use apm_core::validate::validate_config;
pub use apm_core::validate::validate_depends_on;
pub use apm_core::validate::validate_warnings;
pub use apm_core::validate::verify_tickets;
use apm_core::{config::Config, git, ticket, ticket_fmt};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use crate::ctx::CmdContext;

/// Rewrites `.apm/config.toml` (or `apm.toml`) to replace legacy
/// `[workers] command/args/model` fields with the agent-wrapper shape.
///
/// Returns `true` when the file was rewritten, `false` when no legacy fields
/// were detected (no-op) or when migration was blocked by a non-Claude command.
pub fn apply_config_migration_fixes(root: &Path) -> Result<bool> {
    use std::fs;

    // 1. Locate config file
    let config_path = {
        let p = root.join(".apm").join("config.toml");
        if p.exists() {
            p
        } else {
            let p = root.join("apm.toml");
            if p.exists() {
                p
            } else {
                return Ok(false);
            }
        }
    };

    // 2. Parse with toml_edit (preserves comments, whitespace, key order)
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("reading {}", config_path.display()))?;
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("parsing {}", config_path.display()))?;

    // 3. Detect legacy fields.
    // Use .get() throughout: DocumentMut::index panics for missing top-level keys.
    let has_workers_legacy = doc
        .get("workers")
        .and_then(|v| v.as_table())
        .map_or(false, |t| {
            t.contains_key("command") || t.contains_key("args") || t.contains_key("model")
        });

    let profiles_with_legacy: Vec<String> = doc
        .get("worker_profiles")
        .and_then(|v| v.as_table())
        .map(|wp| {
            wp.iter()
                .filter_map(|(name, item)| {
                    item.as_table()
                        .filter(|t| {
                            t.contains_key("command")
                                || t.contains_key("args")
                                || t.contains_key("model")
                        })
                        .map(|_| name.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    if !has_workers_legacy && profiles_with_legacy.is_empty() {
        return Ok(false);
    }

    // 4. Guard: non-claude command — block migration and warn if any command
    //    is not "claude" (we can't safely choose a wrapper for unknown tools).
    if let Some(cmd) = doc
        .get("workers")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("command"))
        .and_then(|v| v.as_str())
    {
        if cmd != "claude" {
            #[allow(clippy::print_stderr)]
            {
                eprintln!(
                    "warning: [workers] command = {:?} is not \"claude\" \u{2014} cannot auto-migrate; choose a wrapper manually",
                    cmd
                );
            }
            return Ok(false);
        }
    }

    for name in &profiles_with_legacy {
        if let Some(cmd) = doc
            .get("worker_profiles")
            .and_then(|v| v.as_table())
            .and_then(|wp| wp.get(name.as_str()))
            .and_then(|p| p.as_table())
            .and_then(|t| t.get("command"))
            .and_then(|v| v.as_str())
        {
            if cmd != "claude" {
                #[allow(clippy::print_stderr)]
                {
                    eprintln!(
                        "warning: [worker_profiles.{}] command = {:?} is not \"claude\" \u{2014} cannot auto-migrate; choose a wrapper manually",
                        name, cmd
                    );
                }
                return Ok(false);
            }
        }
    }

    // 5. Migrate [workers]
    if has_workers_legacy {
        let has_command;
        let model_val: Option<String>;
        let has_args;
        {
            let workers = doc
                .get("workers")
                .and_then(|v| v.as_table())
                .expect("workers is a table (checked in step 3)");
            has_command = workers.contains_key("command");
            model_val = workers.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
            has_args = workers.contains_key("args");
        }

        let workers = doc
            .get_mut("workers")
            .and_then(|v| v.as_table_mut())
            .expect("workers is a table");

        if has_command {
            workers.remove("command");
            workers.insert("agent", toml_edit::value("claude"));
        }
        if has_args {
            workers.remove("args");
        }
        if let Some(ref model) = model_val {
            workers.remove("model");
            if !workers.contains_key("options") {
                workers.insert("options", toml_edit::Item::Table(toml_edit::Table::new()));
            }
            // workers is &mut Table; Table::IndexMut creates keys when missing.
            // options was just inserted as Item::Table, so ["options"] returns &mut Item::Table.
            // ["model"] on Item::Table creates the "model" entry via Item::IndexMut.
            workers["options"]["model"] = toml_edit::value(model.as_str());
        }
    }

    // 6. Migrate each [worker_profiles.<name>]
    for name in &profiles_with_legacy {
        let name = name.as_str();

        let has_command;
        let model_val: Option<String>;
        let has_args;
        {
            let profile = doc
                .get("worker_profiles")
                .and_then(|v| v.as_table())
                .and_then(|wp| wp.get(name))
                .and_then(|v| v.as_table())
                .expect("profile is a table (checked in step 3)");
            has_command = profile.contains_key("command");
            model_val = profile.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
            has_args = profile.contains_key("args");
        }

        let profile = doc
            .get_mut("worker_profiles")
            .and_then(|v| v.as_table_mut())
            .and_then(|wp| wp.get_mut(name))
            .and_then(|v| v.as_table_mut())
            .expect("profile is a table");

        if has_command {
            // Remove command; do NOT add agent at profile level (inherits from [workers])
            profile.remove("command");
        }
        if has_args {
            profile.remove("args");
        }
        if let Some(ref model) = model_val {
            profile.remove("model");
            if !profile.contains_key("options") {
                profile.insert("options", toml_edit::Item::Table(toml_edit::Table::new()));
            }
            profile["options"]["model"] = toml_edit::value(model.as_str());
        }
    }

    // 7. Write back (toml_edit preserves comments, whitespace, and unrelated sections)
    fs::write(&config_path, doc.to_string())
        .with_context(|| format!("writing {}", config_path.display()))?;

    // 8. Re-validate: confirm the migration did not produce an invalid config.
    let migrated_config = apm_core::config::Config::load(root)
        .context("migration produced an unparseable config (this is a bug)")?;
    let errors = apm_core::validate::validate_config(&migrated_config, root);
    if !errors.is_empty() {
        anyhow::bail!(
            "migration produced an invalid config:\n{}",
            errors.join("\n")
        );
    }

    Ok(true)
}

#[derive(Debug, Serialize)]
struct Issue {
    kind: String,
    subject: String,
    message: String,
}

pub fn run(root: &Path, fix: bool, json: bool, config_only: bool, no_aggressive: bool) -> Result<()> {
    // Config migration runs first so the freshly-written config is loaded below.
    if fix && apply_config_migration_fixes(root)? {
        println!("migrated [workers] config to agent-driven shape; legacy command/args/model removed");
    }

    let config_errors;
    let config_warnings;
    let mut ticket_issues: Vec<Issue> = Vec::new();
    let mut tickets_checked = 0usize;
    let config: Config;

    if config_only {
        config = CmdContext::load_config_only(root)?;
        config_errors = validate_config(&config, root);
        config_warnings = validate_warnings(&config, root);
    } else {
        let ctx = CmdContext::load(root, no_aggressive)?;
        config = ctx.config;
        config_errors = validate_config(&config, root);
        config_warnings = validate_warnings(&config, root);
        tickets_checked = ctx.tickets.len();

        let tickets = ctx.tickets;

        let merged = apm_core::git::merged_into_main(root, &config.project.default_branch).unwrap_or_default();
        let merged_set: HashSet<String> = merged.into_iter().collect();

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

        for (subject, message) in validate_depends_on(&config, &tickets) {
            ticket_issues.push(Issue {
                kind: "depends_on".into(),
                subject,
                message,
            });
        }

        for issue in verify_tickets(root, &config, &tickets, &merged_set) {
            ticket_issues.push(Issue {
                kind: "integrity".into(),
                subject: String::new(),
                message: issue,
            });
        }

        if fix {
            apply_branch_fixes(root, &config, branch_fixes)?;
            let merged_refs: HashSet<&str> = merged_set.iter().map(|s| s.as_str()).collect();
            apply_merged_fixes(root, &config, &tickets, &merged_refs)?;
        }
    }

    if fix {
        apply_on_failure_fixes(root, &config)?;
        let pattern = apm_core::init::worktree_gitignore_pattern(&config.worktrees.dir);
        if let Some(p) = pattern {
            let mut msgs = Vec::new();
            apm_core::init::ensure_gitignore(&root.join(".gitignore"), Some(&p), &mut msgs)?;
            for m in &msgs {
                println!("  fixed: {m}");
            }
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

    if config_errors.is_empty() && ticket_issues.is_empty() {
        if let Ok(hash) = apm_core::hash_stamp::config_hash(root) {
            let _ = apm_core::hash_stamp::write_stamp(root, &hash);
        }
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

/// Returns `true` when `workflow.toml` was modified.
/// Repairs in a single write pass:
/// (a) inserts a missing `on_failure` field after each `completion` line
///     for Merge/PrOrEpicMerge transitions, porting the value from the
///     default template's matching transition.
/// (b) appends any state block referenced by `on_failure` that is absent
///     from the project's workflow.
fn apply_on_failure_fixes(root: &Path, config: &Config) -> Result<bool> {
    let workflow_path = root.join(".apm").join("workflow.toml");
    if !workflow_path.exists() {
        return Ok(false);
    }

    let default_on_failure = apm_core::init::default_on_failure_map();
    let default_toml = apm_core::init::default_workflow_toml();

    let declared_states: std::collections::HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();

    // Collect (from_state, to) pairs where on_failure is absent and we know the default value.
    let mut needs_field_patch: Vec<(String, String)> = Vec::new();
    // Collect state names that are referenced by on_failure but not declared.
    let mut needs_state_append: std::collections::HashSet<String> = std::collections::HashSet::new();

    for state in &config.workflow.states {
        for tr in &state.transitions {
            if matches!(
                tr.completion,
                apm_core::config::CompletionStrategy::Merge
                    | apm_core::config::CompletionStrategy::PrOrEpicMerge
            ) {
                if tr.on_failure.is_none() {
                    if default_on_failure.contains_key(&tr.to) {
                        needs_field_patch.push((state.id.clone(), tr.to.clone()));
                        let of_name = &default_on_failure[&tr.to];
                        if !declared_states.contains(of_name.as_str()) {
                            needs_state_append.insert(of_name.clone());
                        }
                    }
                } else if let Some(ref name) = tr.on_failure {
                    if !declared_states.contains(name.as_str()) {
                        needs_state_append.insert(name.clone());
                    }
                }
            }
        }
    }

    if needs_field_patch.is_empty() && needs_state_append.is_empty() {
        return Ok(false);
    }

    let raw = std::fs::read_to_string(&workflow_path)
        .context("reading .apm/workflow.toml")?;
    let mut result = raw.clone();

    // 5a: Insert missing on_failure fields.
    if !needs_field_patch.is_empty() {
        result = patch_on_failure_fields(&result, &needs_field_patch, &default_on_failure);
    }

    // 5b: Append missing state blocks.
    for name in &needs_state_append {
        if let Some(block) = extract_state_block_from_default(default_toml, name) {
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
            result.push_str(&block);
            result.push('\n');
            println!("  fixed: appended state '{name}' from default template");
        } else {
            eprintln!("  warning: state '{name}' not found in default template — add it manually");
        }
    }

    if result == raw {
        return Ok(false);
    }

    std::fs::write(&workflow_path, &result).context("writing .apm/workflow.toml")?;
    Ok(true)
}

/// Insert `on_failure = "..."` after each `completion = "..."` line for the
/// transitions listed in `needs_patch`.
fn patch_on_failure_fields(
    raw: &str,
    needs_patch: &[(String, String)],
    default_on_failure: &std::collections::HashMap<String, String>,
) -> String {
    enum Scope { TopLevel, InState, InTransition }

    let mut scope = Scope::TopLevel;
    let mut current_state_id: Option<String> = None;
    let mut current_to: Option<String> = None;
    let mut out: Vec<String> = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "[[workflow.states]]" {
            scope = Scope::InState;
            current_state_id = None;
            current_to = None;
            out.push(line.to_string());
            continue;
        }
        if trimmed == "[[workflow.states.transitions]]" {
            scope = Scope::InTransition;
            current_to = None;
            out.push(line.to_string());
            continue;
        }
        match scope {
            Scope::InState => {
                if let Some(v) = toml_str_val(trimmed, "id") {
                    current_state_id = Some(v);
                }
            }
            Scope::InTransition => {
                if let Some(v) = toml_str_val(trimmed, "to") {
                    current_to = Some(v);
                }
                if let Some(comp) = toml_str_val(trimmed, "completion") {
                    if comp == "merge" || comp == "pr_or_epic_merge" {
                        if let (Some(ref from), Some(ref to)) =
                            (&current_state_id, &current_to)
                        {
                            let want = needs_patch.iter().any(|(f, t)| f == from && t == to);
                            if want {
                                if let Some(of_val) = default_on_failure.get(to) {
                                    let indent: String = line
                                        .chars()
                                        .take_while(|c| c.is_whitespace())
                                        .collect();
                                    out.push(line.to_string());
                                    out.push(format!("{indent}on_failure = \"{of_val}\""));
                                    println!(
                                        "  fixed: added on_failure = \"{of_val}\" to \
                                         transition '{from}' → '{to}'"
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
            Scope::TopLevel => {}
        }
        out.push(line.to_string());
    }

    let mut s = out.join("\n");
    if raw.ends_with('\n') && !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Scan the default workflow template and return the full TOML block for the
/// state with `id = state_id`, including its transition sub-tables.
/// Returns `None` when the state is not found in the template.
fn extract_state_block_from_default(default_toml: &str, state_id: &str) -> Option<String> {
    let mut in_block = false;
    let mut block: Vec<&str> = Vec::new();

    for line in default_toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[[workflow.states]]" {
            if in_block {
                break; // reached the next state, done
            }
            // Start collecting a candidate block.
            block.clear();
            block.push(line);
            // in_block stays false until we confirm the id.
        } else if !block.is_empty() || in_block {
            block.push(line);
            if !in_block {
                if let Some(v) = toml_str_val(trimmed, "id") {
                    if v == state_id {
                        in_block = true;
                    } else {
                        block.clear(); // wrong state
                    }
                }
            }
        }
    }

    if !in_block || block.is_empty() {
        return None;
    }

    // Strip trailing blank lines.
    while block.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        block.pop();
    }

    Some(block.join("\n"))
}

/// Parse `key = "value"` (with optional whitespace) from a trimmed TOML line.
fn toml_str_val(line: &str, key: &str) -> Option<String> {
    if !line.starts_with(key) {
        return None;
    }
    let rest = line[key.len()..].trim_start();
    if !rest.starts_with('=') {
        return None;
    }
    let after_eq = rest[1..].trim_start();
    if !after_eq.starts_with('"') {
        return None;
    }
    let inner = &after_eq[1..];
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

fn apply_merged_fixes(
    root: &Path,
    config: &Config,
    tickets: &[ticket::Ticket],
    merged_set: &HashSet<&str>,
) -> Result<()> {
    for t in tickets {
        let fm = &t.frontmatter;
        let Some(branch) = &fm.branch else { continue };
        if (fm.state == "in_progress" || fm.state == "implemented")
            && merged_set.contains(branch.as_str())
        {
            let id = fm.id.clone();
            let old_state = fm.state.clone();
            match apm_core::ticket::close(root, config, &id, None, "validate --fix", false) {
                Ok(msgs) => {
                    for msg in &msgs {
                        println!("{msg}");
                    }
                    println!("  fixed {id}: {old_state} → closed");
                }
                Err(e) => eprintln!("  warning: could not fix {id}: {e:#}"),
            }
        }
    }
    Ok(())
}
