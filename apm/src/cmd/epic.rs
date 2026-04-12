use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use crate::ctx::CmdContext;

pub fn run_list(root: &Path) -> Result<()> {
    let ctx = CmdContext::load(root, false)?;

    let epic_branches = apm_core::git::epic_branches(root)?;
    if epic_branches.is_empty() {
        return Ok(());
    }

    let tickets = ctx.tickets;

    for branch in &epic_branches {
        // branch = "epic/<8-char-id>-<slug>"
        let after_prefix = branch.trim_start_matches("epic/");
        let id = &after_prefix[..after_prefix.find('-').unwrap_or(after_prefix.len()).min(8)];
        let title = branch_to_title(branch);

        // Find tickets belonging to this epic.
        let epic_tickets: Vec<_> = tickets
            .iter()
            .filter(|t| t.frontmatter.epic.as_deref() == Some(id))
            .collect();

        // Collect StateConfig references for each ticket (skip unknown states).
        let state_configs: Vec<&apm_core::config::StateConfig> = epic_tickets
            .iter()
            .filter_map(|t| ctx.config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
            .collect();

        let derived = apm_core::epic::derive_epic_state(&state_configs);

        // Build per-state counts (non-zero only).
        let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for t in &epic_tickets {
            *counts.entry(t.frontmatter.state.clone()).or_insert(0) += 1;
        }
        let counts_str: String = counts
            .iter()
            .filter(|(_, &v)| v > 0)
            .map(|(k, v)| format!("{v} {k}"))
            .collect::<Vec<_>>()
            .join(", ");

        println!("{id:<8} [{derived:<12}] {title:<40} {counts_str}");
    }

    Ok(())
}

pub fn run_new(root: &Path, title: String) -> Result<()> {
    let branch = apm_core::epic::create(root, &title)?;
    println!("{branch}");
    Ok(())
}

pub fn run_close(root: &Path, id_arg: &str) -> Result<()> {
    let config = CmdContext::load_config_only(root)?;

    // 1. Resolve the epic branch from the id prefix.
    let matches = apm_core::git::find_epic_branches(root, id_arg);
    let epic_branch = match matches.len() {
        0 => anyhow::bail!("no epic branch found matching '{id_arg}'"),
        1 => matches.into_iter().next().unwrap(),
        _ => anyhow::bail!(
            "ambiguous id '{id_arg}': matches {}\n  {}",
            matches.len(),
            matches.join("\n  ")
        ),
    };

    // 2. Parse the 8-char epic ID from the branch name: epic/<id>-<slug>
    let after_prefix = epic_branch.trim_start_matches("epic/");
    let epic_id = after_prefix.split('-').next().unwrap_or("");

    // 3. Load all tickets and find those belonging to this epic.
    let tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;
    let epic_tickets: Vec<_> = tickets
        .iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
        .collect();

    // 4. Gate check: every epic ticket must be in a satisfies_deps or terminal state.
    let mut not_ready: Vec<String> = Vec::new();
    for t in &epic_tickets {
        let state_id = &t.frontmatter.state;
        let passes = config
            .workflow
            .states
            .iter()
            .find(|s| &s.id == state_id)
            .map(|s| matches!(s.satisfies_deps, apm_core::config::SatisfiesDeps::Bool(true)) || s.terminal)
            .unwrap_or(false);
        if !passes {
            not_ready.push(format!("  {} — {} (state: {})", t.frontmatter.id, t.frontmatter.title, state_id));
        }
    }
    if !not_ready.is_empty() {
        anyhow::bail!(
            "cannot close epic: the following tickets are not ready:\n{}",
            not_ready.join("\n")
        );
    }

    // 5. Check for an existing open PR (idempotency).
    let pr_check = Command::new("gh")
        .args([
            "pr", "list",
            "--head", &epic_branch,
            "--state", "open",
            "--json", "number",
            "--jq", ".[0].number",
        ])
        .current_dir(root)
        .output();
    if let Ok(out) = pr_check {
        let number_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !number_str.is_empty() {
            if let Ok(n) = number_str.parse::<u64>() {
                println!("PR #{n} already open for {epic_branch}");
                return Ok(());
            }
        }
    }

    // 6. Derive a human-readable title from the branch name.
    let pr_title = branch_to_title(&epic_branch);

    // 7. Create the PR.
    let default_branch = &config.project.default_branch;
    let pr_body = format!("Epic: {epic_branch}");
    let create_out = Command::new("gh")
        .args([
            "pr", "create",
            "--base", default_branch,
            "--head", &epic_branch,
            "--title", &pr_title,
            "--body", &pr_body,
        ])
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("gh not found: {e}"))?;

    if !create_out.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&create_out.stderr).trim());
    }

    let url = String::from_utf8_lossy(&create_out.stdout).trim().to_string();
    println!("{url}");
    Ok(())
}

pub fn run_show(root: &std::path::Path, id_arg: &str, no_aggressive: bool) -> anyhow::Result<()> {
    let ctx = CmdContext::load(root, no_aggressive)?;

    let matches = apm_core::git::find_epic_branches(root, id_arg);
    let branch = match matches.len() {
        0 => anyhow::bail!("no epic matching '{id_arg}'"),
        1 => matches.into_iter().next().unwrap(),
        _ => anyhow::bail!(
            "ambiguous prefix '{id_arg}', matches:\n  {}",
            matches.join("\n  ")
        ),
    };

    // Parse the 8-char epic ID from the branch: epic/<id>-<slug>
    let after_prefix = branch.trim_start_matches("epic/");
    let epic_id = after_prefix.split('-').next().unwrap_or("");
    let title = branch_to_title(&branch);

    let epic_tickets: Vec<_> = ctx.tickets
        .iter()
        .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id))
        .collect();

    let state_configs: Vec<&apm_core::config::StateConfig> = epic_tickets
        .iter()
        .filter_map(|t| ctx.config.workflow.states.iter().find(|s| s.id == t.frontmatter.state))
        .collect();

    let derived = apm_core::epic::derive_epic_state(&state_configs);

    println!("Epic:   {title}");
    println!("Branch: {branch}");
    println!("State:  {derived}");
    if let Some(limit) = ctx.config.epic_max_workers(epic_id) {
        println!("Max workers: {limit}");
    }

    if epic_tickets.is_empty() {
        println!();
        println!("(no tickets)");
        return Ok(());
    }

    // Column widths
    let id_w = 8usize;
    let state_w = 13usize;
    let title_w = 32usize;

    println!();
    println!(
        "{:<id_w$}  {:<state_w$}  {:<title_w$}  {}",
        "ID", "State", "Title", "Depends on"
    );
    println!(
        "{:-<id_w$}  {:-<state_w$}  {:-<title_w$}  {}",
        "", "", "", "----------"
    );

    for t in &epic_tickets {
        let fm = &t.frontmatter;
        let deps = fm
            .depends_on
            .as_deref()
            .map(|d| d.join(", "))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<id_w$}  {:<state_w$}  {:<title_w$}  {}",
            fm.id, fm.state, fm.title, deps
        );
    }

    Ok(())
}

pub fn run_set(root: &std::path::Path, id_arg: &str, field: &str, value: &str) -> anyhow::Result<()> {
    if field != "max_workers" && field != "owner" {
        anyhow::bail!("unknown field {field:?}; valid fields: max_workers, owner");
    }

    // Validate the epic exists.
    let matches = apm_core::git::find_epic_branches(root, id_arg);
    if matches.is_empty() {
        eprintln!("error: no epic branch found matching '{id_arg}'");
        std::process::exit(1);
    }
    if matches.len() > 1 {
        anyhow::bail!(
            "ambiguous id '{id_arg}': matches {}\n  {}",
            matches.len(),
            matches.join("\n  ")
        );
    }
    let branch = &matches[0];
    let after_prefix = branch.trim_start_matches("epic/");
    let epic_id = after_prefix.split('-').next().unwrap_or("").to_string();

    if field == "owner" {
        let config = apm_core::config::Config::load(root)?;
        let all_tickets = apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?;
        let terminal = config.terminal_state_ids();

        let (mut to_change, skipped): (Vec<_>, Vec<_>) = all_tickets
            .into_iter()
            .filter(|t| t.frontmatter.epic.as_deref() == Some(epic_id.as_str()))
            .partition(|t| !terminal.contains(&t.frontmatter.state));

        // Pre-flight: ownership check (abort before any writes if any fail)
        for t in &to_change {
            apm_core::ticket::check_owner(root, t)?;
        }

        // Pre-flight: validate the new owner
        let local = apm_core::config::LocalConfig::load(root);
        apm_core::validate::validate_owner(&config, &local, value)?;

        // Apply changes
        for t in &mut to_change {
            apm_core::ticket::set_field(&mut t.frontmatter, "owner", value)?;
            let content = t.serialize()?;
            let rel_path = format!(
                "{}/{}",
                config.tickets.dir.to_string_lossy(),
                t.path.file_name().unwrap().to_string_lossy()
            );
            let ticket_branch = t.frontmatter.branch.clone()
                .or_else(|| apm_core::ticket_fmt::branch_name_from_path(&t.path))
                .unwrap_or_else(|| format!("ticket/{}", t.frontmatter.id));
            apm_core::git::commit_to_branch(
                root,
                &ticket_branch,
                &rel_path,
                &content,
                &format!("ticket({}): bulk set owner = {}", t.frontmatter.id, value),
            )?;
        }

        // Output
        for t in &to_change {
            println!("changed  {}  {}", t.frontmatter.id, t.frontmatter.title);
        }
        for t in &skipped {
            println!("skipped  {}  {}  (state: {})", t.frontmatter.id, t.frontmatter.title, t.frontmatter.state);
        }
        println!("{} ticket(s) changed, {} skipped.", to_change.len(), skipped.len());
        return Ok(());
    }

    let apm_dir = root.join(".apm");
    let epics_path = apm_dir.join("epics.toml");

    let raw = if epics_path.exists() {
        std::fs::read_to_string(&epics_path)
            .with_context(|| format!("cannot read {}", epics_path.display()))?
    } else {
        String::new()
    };
    let mut doc: toml_edit::DocumentMut = raw.parse()
        .with_context(|| format!("cannot parse {}", epics_path.display()))?;

    if value == "-" {
        // Remove max_workers from the epic table.
        if let Some(epic_tbl) = doc.get_mut(&epic_id) {
            if let Some(t) = epic_tbl.as_table_mut() {
                t.remove("max_workers");
            }
        }
    } else {
        let n: i64 = value.parse().map_err(|_| anyhow::anyhow!("max_workers must be a positive integer, got {value:?}"))?;
        if n <= 0 {
            eprintln!("error: max_workers must be ≥ 1, got {n}");
            std::process::exit(1);
        }

        // Ensure [<epic_id>] table exists.
        if doc.get(&epic_id).is_none() {
            doc.insert(&epic_id, toml_edit::Item::Table(toml_edit::Table::new()));
        }
        doc[&epic_id]["max_workers"] = toml_edit::value(n);
    }

    std::fs::create_dir_all(&apm_dir)?;
    std::fs::write(&epics_path, doc.to_string())
        .with_context(|| format!("cannot write {}", epics_path.display()))?;
    Ok(())
}

/// Convert an epic branch name to a human-readable PR title.
/// `epic/ab12cd34-user-authentication` → `"User Authentication"`
pub fn branch_to_title(branch: &str) -> String {
    // Strip "epic/" prefix
    let rest = branch.trim_start_matches("epic/");
    // Strip the "<8-char-id>-" segment (first hyphen-separated token)
    let slug = match rest.find('-') {
        Some(pos) => &rest[pos + 1..],
        None => rest,
    };
    // Replace hyphens with spaces and title-case each word
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_to_title_basic() {
        assert_eq!(branch_to_title("epic/ab12cd34-user-authentication"), "User Authentication");
    }

    #[test]
    fn branch_to_title_single_word() {
        assert_eq!(branch_to_title("epic/ab12cd34-dashboard"), "Dashboard");
    }

    #[test]
    fn branch_to_title_many_words() {
        assert_eq!(branch_to_title("epic/ab12cd34-add-oauth-login-flow"), "Add Oauth Login Flow");
    }

    #[test]
    fn branch_to_title_no_slug() {
        // Degenerate: no hyphen after id — returns empty string (id treated as slug)
        assert_eq!(branch_to_title("epic/ab12cd34"), "Ab12cd34");
    }

    // Gate check logic tests
    #[test]
    fn gate_check_all_passing() {
        use apm_core::config::{StateConfig, WorkflowConfig};

        let states = vec![
            make_state("implemented", true, false),
            make_state("closed", false, true),
        ];
        let wf = WorkflowConfig { states, ..Default::default() };

        // Both states satisfy the gate
        for s in &wf.states {
            assert!(matches!(s.satisfies_deps, apm_core::config::SatisfiesDeps::Bool(true)) || s.terminal, "state {} should pass", s.id);
        }
    }

    #[test]
    fn gate_check_failing_state() {
        use apm_core::config::{StateConfig, WorkflowConfig};

        let states = vec![
            make_state("in_progress", false, false),
            make_state("implemented", true, false),
        ];
        let wf = WorkflowConfig { states, ..Default::default() };

        let in_prog = wf.states.iter().find(|s| s.id == "in_progress").unwrap();
        assert!(!matches!(in_prog.satisfies_deps, apm_core::config::SatisfiesDeps::Bool(true)) && !in_prog.terminal);

        let implemented = wf.states.iter().find(|s| s.id == "implemented").unwrap();
        assert!(matches!(implemented.satisfies_deps, apm_core::config::SatisfiesDeps::Bool(true)) || implemented.terminal);
    }

    fn make_state(id: &str, satisfies_deps: bool, terminal: bool) -> apm_core::config::StateConfig {
        apm_core::config::StateConfig {
            id: id.to_string(),
            label: id.to_string(),
            description: String::new(),
            terminal,
            worker_end: false,
            satisfies_deps: apm_core::config::SatisfiesDeps::Bool(satisfies_deps),
            dep_requires: None,
            transitions: vec![],
            actionable: vec![],
            instructions: None,
        }
    }
}
