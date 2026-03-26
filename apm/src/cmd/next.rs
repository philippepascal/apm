use anyhow::Result;
use apm_core::{config::Config, ticket};
use std::path::Path;

pub fn run(root: &Path, json: bool) -> Result<()> {
    let config = Config::load(root)?;
    let tickets_dir = root.join(&config.tickets.dir);
    let tickets = ticket::load_all(&tickets_dir)?;
    let actionable = &config.agents.actionable_states;
    let pw = config.workflow.prioritization.priority_weight;
    let ew = config.workflow.prioritization.effort_weight;
    let rw = config.workflow.prioritization.risk_weight;

    let mut candidates: Vec<_> = tickets
        .iter()
        .filter(|t| {
            let fm = &t.frontmatter;
            actionable.contains(&fm.state) && fm.agent.is_none()
        })
        .collect();

    candidates.sort_by(|a, b| {
        b.score(pw, ew, rw)
            .partial_cmp(&a.score(pw, ew, rw))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    match candidates.first() {
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
                    r#"{{"id":{}, "title":{:?}, "state":{:?}, "score":{}}}"#,
                    fm.id, fm.title, fm.state, t.score(pw, ew, rw)
                );
            } else {
                println!("#{} [{}] {}", fm.id, fm.state, fm.title);
            }
        }
    }
    Ok(())
}
