use anyhow::Result;
use apm_core::{classify_recovery_options, config::resolve_identity, is_merge_failure_state, RecoveryKind, RecoveryOption};
use std::path::Path;
use crate::ctx::CmdContext;

#[allow(clippy::too_many_arguments)]
// Each argument maps to a distinct CLI flag.
pub fn run(root: &Path, state_filter: Option<String>, unassigned: bool, all: bool, actionable_filter: Option<String>, no_aggressive: bool, mine: bool, author: Option<String>, owner: Option<String>, epic: Option<String>, format: Option<String>) -> Result<()> {
    let ctx = CmdContext::load(root, no_aggressive)?;

    let mine_user: Option<String> = if mine {
        Some(resolve_identity(root))
    } else {
        None
    };
    let author_filter = if mine { None } else { author };

    let filtered = apm_core::ticket::list_filtered(
        &ctx.tickets,
        &ctx.config,
        state_filter.as_deref(),
        unassigned,
        all,
        actionable_filter.as_deref(),
        author_filter.as_deref(),
        owner.as_deref(),
        mine_user.as_deref(),
        epic.as_deref(),
    );

    match format.as_deref() {
        Some("ids") => {
            let ids: Vec<&str> = filtered.iter().map(|t| t.frontmatter.id.as_str()).collect();
            println!("{}", ids.join(","));
            return Ok(());
        }
        Some("json") => {
            let fms: Vec<_> = filtered.iter().map(|t| &t.frontmatter).collect();
            println!("{}", serde_json::to_string(&fms)?);
            return Ok(());
        }
        Some(other) => {
            anyhow::bail!("unknown format {:?}; supported: ids, json", other);
        }
        None => {}
    }

    // Pre-compute stale epic IDs before printing rows.
    let default_branch = &ctx.config.project.default_branch;
    let mut epic_map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for t in &filtered {
        if let Some(tb) = t.frontmatter.target_branch.as_deref() {
            if tb.starts_with("epic/") {
                let id = apm_core::epic::epic_id_from_branch(tb).to_owned();
                epic_map.entry(id).or_insert_with(|| tb.to_owned());
            }
        }
    }
    let mut stale_epic_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (id, branch) in &epic_map {
        let s = apm_core::epic::merge_tree_status(root, default_branch, branch)
            .unwrap_or(apm_core::epic::MergeStatus { ahead: 0, clean: true });
        if s.ahead > 0 {
            stale_epic_ids.insert(id.clone());
        }
    }

    let mut stale_tickets: Vec<(&str, &str)> = Vec::new();
    let mut diverged_tickets: Vec<(&str, &str)> = Vec::new();

    for t in &filtered {
        let fm = &t.frontmatter;
        let owner = fm.owner.as_deref().unwrap_or("-");
        let base = match fm.target_branch.as_deref() {
            Some(branch) if branch.starts_with("epic/") => {
                let id = apm_core::epic::epic_id_from_branch(branch);
                if stale_epic_ids.contains(id) {
                    format!("{}↓", id)
                } else {
                    id.to_owned()
                }
            }
            Some(branch) => apm_core::epic::epic_id_from_branch(branch).to_owned(),
            None => ctx.config.project.default_branch.clone(),
        };
        let id_display = if t.local_stale {
            format!("*{}", fm.id)
        } else {
            fm.id.clone()
        };
        println!("{:<9} [{:<12}] {:<16} {:<12} {}", id_display, fm.state, owner, base, fm.title);

        if t.local_stale {
            stale_tickets.push((&fm.id, &fm.title));
        }
        if t.local_diverged {
            diverged_tickets.push((&fm.id, &fm.title));
        }
    }

    if !diverged_tickets.is_empty() {
        eprintln!();
        eprintln!("warning: local ref has diverged from origin on {} ticket(s) — showing local content:", diverged_tickets.len());
        for (id, title) in &diverged_tickets {
            eprintln!("    {}  {}", id, title);
        }
    }

    if !stale_tickets.is_empty() {
        println!();
        println!("  * local ref behind origin — run `apm sync` to fast-forward:");
        for (id, title) in &stale_tickets {
            println!("      *{}  {}", id, title);
        }
    }

    if let Some(state) = &state_filter {
        if is_merge_failure_state(state, &ctx.config.workflow) {
            let opts = classify_recovery_options(state, &ctx.config.workflow);
            let relevant: Vec<&RecoveryOption> = opts.iter().filter(|o| matches!(
                o.kind,
                RecoveryKind::RetryMerge | RecoveryKind::ReturnToWorker
            )).collect();
            if !relevant.is_empty() {
                let parts: Vec<String> = relevant.iter()
                    .map(|o| format!("{} → apm state <id> {}", o.label, o.to))
                    .collect();
                println!("\nRecovery: {}", parts.join("  "));
            }
        }
    }

    Ok(())
}
