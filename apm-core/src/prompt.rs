use anyhow::Result;
use std::io::Write;
use std::path::Path;
use crate::config::Config;
use crate::ticket;
use crate::start::{build_system_prompt, effective_spawn_params, apply_frontmatter_agent, resolve_profile};

/// Print the system prompt that would be used if the ticket's current
/// `command:start` transition fired.  Agent and role may be overridden for
/// inspection without modifying the ticket.
pub fn run(
    root: &Path,
    id: &str,
    agent_override: Option<&str>,
    role_override: Option<&str>,
    out: &mut dyn Write,
) -> Result<()> {
    let config = Config::load(root)?;
    let tickets = ticket::load_all_from_git(root, &config.tickets.dir)?;
    let ticket_id = ticket::resolve_id_in_slice(&tickets, id)?;
    let t = tickets.iter()
        .find(|t| t.frontmatter.id == ticket_id)
        .ok_or_else(|| anyhow::anyhow!("ticket {:?} not found", ticket_id))?;

    let state = &t.frontmatter.state;
    let triggering_transition = config.workflow.states.iter()
        .find(|s| s.id == *state)
        .and_then(|s| s.transitions.iter().find(|tr| tr.trigger == "command:start"));

    let mut warnings = Vec::new();
    let profile = triggering_transition.and_then(|tr| resolve_profile(tr, &config, &mut warnings));
    let profile_name = triggering_transition
        .and_then(|tr| tr.profile.as_deref())
        .unwrap_or("")
        .to_string();

    let role_from_cascade = profile.and_then(|p| p.role.as_deref()).unwrap_or("worker");
    let role = role_override.unwrap_or(role_from_cascade).to_string();

    let tr_agent = triggering_transition.and_then(|tr| tr.agent.as_deref());
    let mut params = effective_spawn_params(tr_agent, profile, &config.workers);
    apply_frontmatter_agent(&mut params.agent, &t.frontmatter, &profile_name);

    let agent = agent_override.unwrap_or(&params.agent).to_string();

    let tr_instructions = triggering_transition.and_then(|tr| tr.instructions.as_deref());
    let prompt = build_system_prompt(root, tr_instructions, profile, &config.workers, &agent, &role)?;

    out.write_all(prompt.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parity_project(root: &Path, ticket_id: &str) {
        use std::fs;

        fs::create_dir_all(root.join(".apm/agents/mock-happy")).unwrap();
        fs::create_dir_all(root.join("tickets")).unwrap();
        fs::write(root.join(".apm/agents/mock-happy/apm.worker.md"), "PER-AGENT INSTRUCTIONS").unwrap();
        fs::write(root.join(".apm/config.toml"), r#"
[project]
name = "parity-test"
default_branch = "main"

[workers]
agent = "mock-happy"

[tickets]
dir = "tickets"
"#).unwrap();
        fs::write(root.join(".apm/workflow.toml"), r#"
[[workflow.states]]
id = "ready"
label = "Ready"
actionable = ["agent"]

  [[workflow.states.transitions]]
  to = "in_progress"
  trigger = "command:start"
  label = "Start"

[[workflow.states]]
id = "in_progress"
label = "In Progress"
"#).unwrap();

        let ticket_content = format!(r#"+++
id = "{ticket_id}"
title = "Parity Test"
state = "ready"
priority = 0
effort = 5
risk = 3
author = "test"
owner = "test"
branch = "ticket/{ticket_id}-test"
created_at = "2026-01-01T00:00:00Z"
updated_at = "2026-01-01T00:00:00Z"
+++

## Spec

### Problem

Test.

## History

| When | From | To | By |
|------|------|----|----|
"#);
        fs::write(root.join(format!("tickets/{ticket_id}-test.md")), ticket_content).unwrap();

        std::process::Command::new("git")
            .arg("init").current_dir(root).output().unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "t@t.com"]).current_dir(root).output().unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "T"]).current_dir(root).output().unwrap();
        std::process::Command::new("git")
            .args(["add", ".apm"]).current_dir(root).output().unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "init", "--allow-empty"]).current_dir(root).output().unwrap();
        let branch = format!("ticket/{ticket_id}-test");
        std::process::Command::new("git")
            .args(["checkout", "-b", &branch]).current_dir(root).output().unwrap();
        std::process::Command::new("git")
            .args(["add", &format!("tickets/{ticket_id}-test.md")]).current_dir(root).output().unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "add ticket"]).current_dir(root).output().unwrap();
        std::process::Command::new("git")
            .args(["checkout", "main"]).current_dir(root).output().unwrap();
    }

    /// AC #6: prompt::run() must produce the same output as build_system_prompt()
    /// for the same (agent, role, ticket) inputs.
    #[test]
    fn parity_build_system_prompt_matches_prompt_run() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_parity_project(root, "abcd0001");

        let config = crate::config::Config::load(root).unwrap();

        // Direct call to build_system_prompt — the same code path used by
        // run(), run_next(), and spawn_next_worker().
        let expected = crate::start::build_system_prompt(
            root,
            None,
            None,
            &config.workers,
            "mock-happy",
            "worker",
        ).unwrap();

        // prompt::run in capture mode
        let mut buf = Vec::new();
        run(root, "abcd0001", None, None, &mut buf).unwrap();
        let actual = String::from_utf8(buf).unwrap();

        assert_eq!(
            actual, expected,
            "prompt::run output must match build_system_prompt output for same inputs"
        );
    }
}
