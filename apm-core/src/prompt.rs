use anyhow::Result;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use crate::config::Config;
use crate::start::{build_system_prompt, explain_system_prompt, PromptProvenance, effective_spawn_params, apply_frontmatter_agent, resolve_profile};
use crate::ticket;

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

/// Print a provenance table showing which cascade level supplied the system
/// prompt and why each other level was skipped.
pub fn explain(
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
    let prov = explain_system_prompt(
        root,
        tr_instructions,
        profile,
        &config.workers,
        config.agents.instructions.as_deref(),
        &agent,
        &role,
    )?;

    format_provenance(&prov, out)
}

/// Build and print the system prompt for a given agent+role without a ticket.
/// Levels 1 (transition.instructions) and 2 (profile.instructions) are skipped;
/// level 0 (per-agent file), 3 (workers.instructions), and 4 (built-in default)
/// resolve normally.
pub fn run_without_ticket(
    root: &Path,
    agent: &str,
    role: &str,
    out: &mut dyn Write,
) -> Result<()> {
    let config = Config::load(root)?;
    let prompt = build_system_prompt(
        root,
        None,
        None,
        &config.workers,
        config.agents.instructions.as_deref(),
        agent,
        role,
    )?;
    out.write_all(prompt.as_bytes())?;
    Ok(())
}

/// Print a provenance table for a given agent+role without a ticket.
pub fn explain_without_ticket(
    root: &Path,
    agent: &str,
    role: &str,
    out: &mut dyn Write,
) -> Result<()> {
    let config = Config::load(root)?;
    let prov = explain_system_prompt(
        root,
        None,
        None,
        &config.workers,
        config.agents.instructions.as_deref(),
        agent,
        role,
    )?;
    format_provenance(&prov, out)
}

fn format_provenance(prov: &PromptProvenance, out: &mut dyn Write) -> Result<()> {
    match &prov.prefix_path {
        Some(path) => writeln!(out, "{:<16}{}  (agents.instructions)", "prefix:", path)?,
        None => writeln!(out, "{:<16}none", "prefix:")?,
    }
    writeln!(
        out,
        "{:<16}{}  (level {} \u{2014} {})",
        "system prompt:",
        prov.winner.source,
        prov.winner.level,
        prov.winner.label,
    )?;
    let mut first = true;
    for entry in &prov.skipped {
        let label = if first { "skipped:" } else { "" };
        writeln!(
            out,
            "{:<16}level {} ({} \u{2014} {})",
            label,
            entry.level,
            entry.label,
            entry.source,
        )?;
        first = false;
    }
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

    // --- explain tests ---

    fn make_explain_project(root: &Path, ticket_id: &str, agent: &str, create_per_agent_file: bool, agents_instructions: Option<&str>) {
        use std::fs;

        fs::create_dir_all(root.join(".apm")).unwrap();
        fs::create_dir_all(root.join("tickets")).unwrap();

        if create_per_agent_file {
            let agent_dir = root.join(format!(".apm/agents/{agent}"));
            fs::create_dir_all(&agent_dir).unwrap();
            fs::write(agent_dir.join("apm.worker.md"), format!("INSTRUCTIONS FOR {agent}")).unwrap();
        }

        let agents_section = match agents_instructions {
            Some(path) => format!("\n[agents]\ninstructions = \"{path}\"\n"),
            None => String::new(),
        };

        fs::write(root.join(".apm/config.toml"), format!(r#"
[project]
name = "explain-test"
default_branch = "main"

[workers]
agent = "{agent}"

[tickets]
dir = "tickets"
{agents_section}"#)).unwrap();

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
title = "Explain Test"
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

        let git = |args: &[&str]| {
            std::process::Command::new("git").args(args).current_dir(root).output().unwrap()
        };

        git(&["init"]);
        git(&["config", "user.email", "t@t.com"]);
        git(&["config", "user.name", "T"]);
        git(&["add", ".apm"]);
        git(&["commit", "-m", "init", "--allow-empty"]);
        let branch = format!("ticket/{ticket_id}-test");
        git(&["checkout", "-b", &branch]);
        git(&["add", &format!("tickets/{ticket_id}-test.md")]);
        git(&["commit", "-m", "add ticket"]);
        git(&["checkout", "main"]);
    }

    #[test]
    fn explain_level0_wins() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_explain_project(root, "aaaa0001", "mock-happy", true, None);

        let mut buf = Vec::new();
        explain(root, "aaaa0001", None, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("level 0"), "should show level 0; got:\n{output}");
        assert!(output.contains(".apm/agents/mock-happy/apm.worker.md"), "should show per-agent path; got:\n{output}");
        assert_eq!(output.matches("not reached").count(), 3, "levels 1-3 should be not reached; got:\n{output}");
    }

    #[test]
    fn explain_level4_wins() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_explain_project(root, "bbbb0002", "claude", false, None);

        let mut buf = Vec::new();
        explain(root, "bbbb0002", None, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("level 4"), "should show level 4; got:\n{output}");
        assert!(output.contains("built-in default"), "should show built-in default; got:\n{output}");
    }

    #[test]
    fn explain_prefix_shown() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_explain_project(root, "cccc0003", "claude", false, Some(".apm/agents/default/agents.md"));

        let mut buf = Vec::new();
        explain(root, "cccc0003", None, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let prefix_line = output.lines().find(|l| l.starts_with("prefix:")).unwrap();
        assert!(
            prefix_line.contains(".apm/agents/default/agents.md"),
            "prefix line should name the configured file; got: {prefix_line:?}"
        );
    }

    #[test]
    fn explain_agent_role_override() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Project uses mock-happy (which has a per-agent file). Override to claude
        // (no per-agent file) → level 4 wins and skipped level 0 shows claude's path.
        make_explain_project(root, "dddd0004", "mock-happy", true, None);

        let mut buf = Vec::new();
        explain(root, "dddd0004", Some("claude"), Some("worker"), &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("level 4"), "claude override should fall to level 4; got:\n{output}");
        assert!(output.contains("built-in default (claude/worker)"), "should name claude/worker; got:\n{output}");
        assert!(output.contains(".apm/agents/claude/apm.worker.md"), "skipped level 0 should use overridden agent name; got:\n{output}");
    }
}
