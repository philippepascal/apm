use anyhow::Result;
use apm_core::{classify_recovery_options, config::Config, is_merge_failure_state, ticket, RecoveryKind, RecoveryOption};
use std::path::Path;

pub fn run(root: &Path, json: bool, no_aggressive: bool) -> Result<()> {
    let config = Config::load(root)?;
    let aggressive = config.sync.aggressive && !no_aggressive;

    crate::util::fetch_if_aggressive(root, aggressive);

    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let actionable_owned = config.actionable_states_for("agent");
    let actionable: Vec<&str> = actionable_owned.iter().map(|s| s.as_str()).collect();
    let p = &config.workflow.prioritization;
    let caller_name = apm_core::config::resolve_caller_name();
    let current_user = apm_core::config::resolve_identity(root);

    match ticket::pick_next(&tickets, &actionable, &[], p.priority_weight, p.effort_weight, p.risk_weight, &config, Some(&caller_name), Some(&current_user)) {
        None => {
            if json {
                println!("null");
            } else {
                println!("No actionable tickets.");
            }
        }
        Some(t) => {
            let fm = &t.frontmatter;
            if json {
                println!(
                    r#"{{"id":{:?}, "title":{:?}, "state":{:?}, "score":{}}}"#,
                    fm.id, fm.title, fm.state, t.score(p.priority_weight, p.effort_weight, p.risk_weight)
                );
            } else {
                println!("{} [{}] {}", fm.id, fm.state, fm.title);
                if let Some(epic_id) = fm.epic.as_deref() {
                    if let Some(epic_branch) = apm_core::epic::find_epic_branch(root, epic_id) {
                        let s = apm_core::epic::merge_tree_status(root, &config.project.default_branch, &epic_branch)
                            .unwrap_or(apm_core::epic::MergeStatus { ahead: 0, clean: true });
                        let label = if s.ahead == 0 {
                            "up to date".to_string()
                        } else if s.clean {
                            format!("↓{} clean", s.ahead)
                        } else {
                            format!("↓{} CONFLICTS", s.ahead)
                        };
                        println!("  (epic {epic_id}: {label})");
                    }
                }
                if is_merge_failure_state(&fm.state, &config.workflow) {
                    let opts = classify_recovery_options(&fm.state, &config.workflow);
                    let mut groups: [Vec<&RecoveryOption>; 4] = [vec![], vec![], vec![], vec![]];
                    for opt in &opts {
                        let idx = match opt.kind {
                            RecoveryKind::RetryMerge     => 0,
                            RecoveryKind::ReturnToWorker => 1,
                            RecoveryKind::Abandon        => 2,
                            RecoveryKind::Other          => 3,
                        };
                        groups[idx].push(opt);
                    }
                    println!("\nRecovery options:");
                    for opt in groups.iter().flatten() {
                        println!("  {}  →  apm state {} {}", opt.label, fm.id, opt.to);
                    }
                    println!();
                }
            }
        }
    }
    Ok(())
}
