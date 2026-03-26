use apm_core::{config::Config, git, ticket};
use chrono::Utc;
use std::io::BufRead;
use std::path::Path;

pub fn run(root: &Path, hook_name: &str) {
    match hook_name {
        "pre-push" => pre_push(root),
        other => eprintln!("apm _hook: unknown hook {:?}", other),
    }
}

fn pre_push(root: &Path) {
    let config = match Config::load(root) {
        Ok(c) => c,
        Err(e) => { eprintln!("warning: apm _hook pre-push: {e:#}"); return; }
    };
    let tickets_dir = root.join(&config.tickets.dir);
    let tickets = match ticket::load_all(&tickets_dir) {
        Ok(t) => t,
        Err(e) => { eprintln!("warning: apm _hook pre-push: {e:#}"); return; }
    };

    for line in std::io::stdin().lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { continue; }
        let local_ref = parts[0];

        // Extract id from refs/heads/ticket/0007-...
        let branch = local_ref.trim_start_matches("refs/heads/");
        let id = match parse_ticket_id(branch) {
            Some(id) => id,
            None => continue,
        };

        let Some(t) = tickets.iter().find(|t| t.frontmatter.id == id) else { continue };
        if t.frontmatter.state != "ready" { continue; }

        let mut t = t.clone();
        let now = Utc::now();
        let old_state = t.frontmatter.state.clone();
        t.frontmatter.state = "in_progress".into();
        t.frontmatter.updated_at = Some(now);
        let when = now.format("%Y-%m-%dT%H:%MZ").to_string();
        crate::cmd::state::append_history(&mut t.body, &old_state, "in_progress", &when, "hook");

        let content = match t.serialize() {
            Ok(c) => c,
            Err(e) => { eprintln!("warning: ticket({id}) serialize failed: {e:#}"); continue; }
        };
        let rel_path = format!(
            "{}/{}",
            config.tickets.dir.to_string_lossy(),
            t.path.file_name().unwrap().to_string_lossy()
        );
        let branch_name = t.frontmatter.branch.clone()
            .or_else(|| git::branch_name_from_path(&t.path))
            .unwrap_or_else(|| format!("ticket/{id:04}"));

        match git::commit_to_branch(root, &branch_name, &rel_path, &content,
            &format!("ticket({id}): ready → in_progress (branch push)")) {
            Ok(_) => println!("#{id}: ready → in_progress (branch push)"),
            Err(e) => eprintln!("warning: ticket({id}) commit failed: {e:#}"),
        }
    }
}

fn parse_ticket_id(branch: &str) -> Option<u32> {
    // ticket/0007-some-slug → 7
    let suffix = branch.strip_prefix("ticket/")?;
    let num_str = suffix.split('-').next()?;
    num_str.parse().ok()
}
