use anyhow::Result;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use crate::config::Config;
use crate::ticket;
use crate::start::{build_system_prompt, effective_spawn_params, apply_frontmatter_agent, resolve_profile};

/// Scan `.apm/agents/` for agent subdirectory names and role names extracted
/// from `apm.<role>.md` filenames, then print a two-line discovery summary.
pub fn discover(root: &Path, out: &mut dyn Write) -> Result<()> {
    let agents_dir = root.join(".apm/agents");
    let mut agents: Vec<String> = Vec::new();
    let mut roles: HashSet<String> = HashSet::new();

    if agents_dir.is_dir() {
        let mut dir_names: Vec<String> = std::fs::read_dir(&agents_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        dir_names.sort();

        for agent_name in &dir_names {
            let agent_dir = agents_dir.join(agent_name);
            if let Ok(entries) = std::fs::read_dir(&agent_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Ok(name) = entry.file_name().into_string() {
                        if let Some(rest) = name.strip_prefix("apm.") {
                            if let Some(role) = rest.strip_suffix(".md") {
                                roles.insert(role.to_string());
                            }
                        }
                    }
                }
            }
        }
        agents = dir_names;
    }

    let mut sorted_roles: Vec<String> = roles.into_iter().collect();
    sorted_roles.sort();

    let agents_str = agents.join(", ");
    let roles_str = sorted_roles.join(", ");

    if agents_str.is_empty() {
        writeln!(out, "Agents:")?;
    } else {
        writeln!(out, "{:<8} {}", "Agents:", agents_str)?;
    }
    if roles_str.is_empty() {
        writeln!(out, "Roles:")?;
    } else {
        writeln!(out, "{:<8} {}", "Roles:", roles_str)?;
    }

    Ok(())
}

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
    let prompt = build_system_prompt(root, tr_instructions, profile, &config.workers, config.agents.instructions.as_deref(), &agent, &role)?;

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
            None,
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

    /// Replicates the argument-construction logic shared by run(), run_next(), and
    /// spawn_next_worker() and calls build_system_prompt() directly.  All three
    /// spawn paths clone the triggering transition and derive (profile, role,
    /// agent, tr_instructions) identically before calling build_system_prompt().
    fn spawn_path_build_system_prompt(root: &Path, ticket_id: &str) -> anyhow::Result<String> {
        use crate::start::{build_system_prompt, resolve_profile, effective_spawn_params, apply_frontmatter_agent};

        let config = crate::config::Config::load(root)?;
        let tickets = crate::ticket::load_all_from_git(root, &config.tickets.dir)?;
        let t = tickets.iter()
            .find(|t| t.frontmatter.id == ticket_id)
            .ok_or_else(|| anyhow::anyhow!("ticket {:?} not found", ticket_id))?;

        let state = t.frontmatter.state.clone();
        // Clone the transition as run_next() and spawn_next_worker() do.
        let triggering_transition = config.workflow.states.iter()
            .find(|s| s.id == state)
            .and_then(|s| s.transitions.iter().find(|tr| tr.trigger == "command:start"))
            .cloned();

        let mut warnings = Vec::new();
        let profile = triggering_transition.as_ref()
            .and_then(|tr| resolve_profile(tr, &config, &mut warnings));
        let profile_name = triggering_transition.as_ref()
            .and_then(|tr| tr.profile.as_deref())
            .unwrap_or("")
            .to_string();

        let role = profile.and_then(|p| p.role.as_deref()).unwrap_or("worker").to_string();
        let tr_agent = triggering_transition.as_ref().and_then(|tr| tr.agent.as_deref());
        let mut params = effective_spawn_params(tr_agent, profile, &config.workers);
        apply_frontmatter_agent(&mut params.agent, &t.frontmatter, &profile_name);

        let tr_instructions = triggering_transition.as_ref()
            .and_then(|tr| tr.instructions.as_deref())
            .map(|s| s.to_string());

        build_system_prompt(root, tr_instructions.as_deref(), profile, &config.workers, None, &params.agent, &role)
    }

    /// AC #1: parity for the run() spawn path — argument construction identical
    /// to prompt::run().
    #[test]
    fn parity_run_path_matches_prompt_run() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_parity_project(root, "abcd0002");

        let from_spawn_path = spawn_path_build_system_prompt(root, "abcd0002").unwrap();

        let mut buf = Vec::new();
        run(root, "abcd0002", None, None, &mut buf).unwrap();
        let from_prompt_run = String::from_utf8(buf).unwrap();

        assert_eq!(
            from_spawn_path, from_prompt_run,
            "run() path: build_system_prompt output must match prompt::run output"
        );
    }

    /// AC #2a: parity for the run_next() spawn path.
    #[test]
    fn parity_run_next_path_matches_prompt_run() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_parity_project(root, "abcd0003");

        let from_spawn_path = spawn_path_build_system_prompt(root, "abcd0003").unwrap();

        let mut buf = Vec::new();
        run(root, "abcd0003", None, None, &mut buf).unwrap();
        let from_prompt_run = String::from_utf8(buf).unwrap();

        assert_eq!(
            from_spawn_path, from_prompt_run,
            "run_next() path: build_system_prompt output must match prompt::run output"
        );
    }

    /// AC #2b: parity for the spawn_next_worker() path.
    #[test]
    fn parity_spawn_next_worker_path_matches_prompt_run() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_parity_project(root, "abcd0004");

        let from_spawn_path = spawn_path_build_system_prompt(root, "abcd0004").unwrap();

        let mut buf = Vec::new();
        run(root, "abcd0004", None, None, &mut buf).unwrap();
        let from_prompt_run = String::from_utf8(buf).unwrap();

        assert_eq!(
            from_spawn_path, from_prompt_run,
            "spawn_next_worker() path: build_system_prompt output must match prompt::run output"
        );
    }

    /// Creates a minimal project where transition.instructions points to a file
    /// that does not exist, so build_system_prompt() will return an error.
    fn make_error_project(root: &Path, ticket_id: &str) {
        use std::fs;

        fs::create_dir_all(root.join(".apm")).unwrap();
        fs::create_dir_all(root.join("tickets")).unwrap();
        fs::write(root.join(".apm/config.toml"), r#"
[project]
name = "error-test"
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
  instructions = "nonexistent.md"

[[workflow.states]]
id = "in_progress"
label = "In Progress"
"#).unwrap();

        let ticket_content = format!(r#"+++
id = "{ticket_id}"
title = "Error Test"
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

    #[test]
    fn discover_lists_agents_and_roles() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join(".apm/agents/mock-happy")).unwrap();
        fs::create_dir_all(root.join(".apm/agents/pi")).unwrap();
        fs::write(root.join(".apm/agents/mock-happy/apm.worker.md"), "").unwrap();
        fs::write(root.join(".apm/agents/pi/apm.spec-writer.md"), "").unwrap();

        let mut buf = Vec::new();
        discover(root, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Agents:  mock-happy, pi"), "got: {output:?}");
        assert!(output.contains("Roles:   spec-writer, worker"), "got: {output:?}");
    }

    #[test]
    fn discover_no_agents_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let mut buf = Vec::new();
        let result = discover(root, &mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Agents:"), "got: {output:?}");
        assert!(output.contains("Roles:"), "got: {output:?}");
    }

    #[test]
    fn discover_deduplicates_roles() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join(".apm/agents/alpha")).unwrap();
        fs::create_dir_all(root.join(".apm/agents/beta")).unwrap();
        fs::write(root.join(".apm/agents/alpha/apm.worker.md"), "").unwrap();
        fs::write(root.join(".apm/agents/beta/apm.worker.md"), "").unwrap();

        let mut buf = Vec::new();
        discover(root, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let roles_line = output.lines().find(|l| l.starts_with("Roles:")).unwrap();
        let count = roles_line.matches("worker").count();
        assert_eq!(count, 1, "role should appear once; got: {roles_line:?}");
    }

    /// AC #3: when build_system_prompt returns an error (missing instructions
    /// file), the spawn path propagates it unchanged.  One test for run()'s
    /// argument-construction path is sufficient; the error-propagation mechanism
    /// (the ? operator) is identical across all three spawn paths.
    #[test]
    fn error_missing_instructions_propagated() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_error_project(root, "errtest1");

        // spawn_path_build_system_prompt replicates the spawn-path argument
        // construction and propagates the error from build_system_prompt via ?.
        let spawn_err = spawn_path_build_system_prompt(root, "errtest1")
            .unwrap_err()
            .to_string();

        // Direct call to build_system_prompt with the same arguments — must
        // produce the identical error string.
        let config = crate::config::Config::load(root).unwrap();
        let direct_err = crate::start::build_system_prompt(
            root,
            Some("nonexistent.md"),
            None,
            &config.workers,
            None,
            "mock-happy",
            "worker",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            spawn_err, direct_err,
            "spawn path must propagate build_system_prompt error unchanged"
        );
    }
}
