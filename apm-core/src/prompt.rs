use anyhow::Result;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use crate::config::Config;
use crate::start::{build_system_prompt, build_user_message, explain_system_prompt, PromptProvenance, resolve_worker_profile, apply_frontmatter_agent};
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

fn resolve_agent_role(
    config: &Config,
    triggering_transition: Option<&crate::config::TransitionConfig>,
    frontmatter: &crate::ticket_fmt::Frontmatter,
    agent_override: Option<&str>,
    role_override: Option<&str>,
) -> Result<(String, String)> {
    let default_wp = config.workers.default.as_deref().unwrap_or("claude/coder");
    let wp_str = triggering_transition
        .and_then(|tr| tr.worker_profile.as_deref())
        .unwrap_or(default_wp);
    let wp = resolve_worker_profile(wp_str, &config.workers)?;
    let mut agent = wp.agent.clone();
    apply_frontmatter_agent(&mut agent, frontmatter, wp_str);
    let agent = agent_override.unwrap_or(&agent).to_string();
    let role = role_override.unwrap_or(&wp.role).to_string();
    Ok((agent, role))
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

    let (agent, role) = resolve_agent_role(&config, triggering_transition, &t.frontmatter, agent_override, role_override)?;
    let project_file = config.agents.project.as_deref().map(Path::new);
    let prompt = build_system_prompt(root, project_file, &agent, &role, Some(&ticket_id))?;

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

    let (agent, role) = resolve_agent_role(&config, triggering_transition, &t.frontmatter, agent_override, role_override)?;
    let project_file = config.agents.project.as_deref().map(Path::new);
    let prov = explain_system_prompt(root, project_file, &agent, &role)?;

    format_provenance(&prov, out)
}

/// Build and print the system prompt for a given agent+role without a ticket.
pub fn run_without_ticket(
    root: &Path,
    agent: &str,
    role: &str,
    out: &mut dyn Write,
) -> Result<()> {
    let config = Config::load(root)?;
    let project_file = config.agents.project.as_deref().map(Path::new);
    let prompt = build_system_prompt(root, project_file, agent, role, None)?;
    out.write_all(prompt.as_bytes())?;
    Ok(())
}

/// Print the user message that would be sent to the worker if the ticket's
/// current command:start transition fired.  Includes the dependency context
/// bundle (if the ticket has deps) prepended to the serialised ticket content.
pub fn run_message(
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

    let (_, role) = resolve_agent_role(&config, triggering_transition, &t.frontmatter, agent_override, role_override)?;
    let depends_on = t.frontmatter.depends_on.clone().unwrap_or_default();

    let msg = build_user_message(root, t, &depends_on, &role, &config)?;
    out.write_all(msg.as_bytes())?;
    Ok(())
}

/// Print both the system prompt and user message, separated by section headers.
/// Use --system or --message to narrow to one part.
pub fn run_full(
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

    let (agent, role) = resolve_agent_role(&config, triggering_transition, &t.frontmatter, agent_override, role_override)?;
    let project_file = config.agents.project.as_deref().map(Path::new);
    let depends_on = t.frontmatter.depends_on.clone().unwrap_or_default();

    let sys = build_system_prompt(root, project_file, &agent, &role, Some(&ticket_id))?;
    let msg = build_user_message(root, t, &depends_on, &role, &config)?;

    writeln!(out, "=== system ===")?;
    out.write_all(sys.as_bytes())?;
    writeln!(out, "\n=== user ===")?;
    out.write_all(msg.as_bytes())?;
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
    let project_file = config.agents.project.as_deref().map(Path::new);
    let prov = explain_system_prompt(root, project_file, agent, role)?;
    format_provenance(&prov, out)
}

fn format_provenance(prov: &PromptProvenance, out: &mut dyn Write) -> Result<()> {
    // Layer 1: role file (highest-attention position, was previously layer 3)
    writeln!(
        out,
        "{:<16}{}  (level {} \u{2014} {})",
        "layer 1:",
        prov.winner.source,
        prov.winner.level,
        prov.winner.label,
    )?;
    // Layer 2: project context (unchanged)
    match &prov.layer2_path {
        Some(path) => writeln!(out, "{:<16}{}", "layer 2:", path)?,
        None => writeln!(out, "{:<16}not configured", "layer 2:")?,
    }
    // Layer 3: apm instructions (reference material, was previously layer 1)
    let role_str = prov.instructions_role.as_deref().unwrap_or("(none)");
    writeln!(out, "{:<16}apm instructions (dynamic, role: {})", "layer 3:", role_str)?;
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
default = "mock-happy/worker"

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

        let expected = crate::start::build_system_prompt(
            root,
            None,
            "mock-happy",
            "worker",
            Some("abcd0001"),
        ).unwrap();

        let mut buf = Vec::new();
        run(root, "abcd0001", None, None, &mut buf).unwrap();
        let actual = String::from_utf8(buf).unwrap();

        assert_eq!(
            actual, expected,
            "prompt::run output must match build_system_prompt output for same inputs"
        );
    }

    #[test]
    fn explain_role_file_is_layer1() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_explain_project(root, "eeee0005", "mock-happy", true, None);

        let mut buf = Vec::new();
        explain(root, "eeee0005", None, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let layer1_line = output.lines().find(|l| l.starts_with("layer 1:")).unwrap();
        assert!(
            layer1_line.contains(".apm/agents/mock-happy/apm.coder.md"),
            "layer 1 should identify the role file, not apm instructions; got: {layer1_line:?}"
        );
        assert!(
            !layer1_line.contains("apm instructions"),
            "apm instructions should not appear on layer 1 line; got: {layer1_line:?}"
        );
    }

    #[test]
    fn explain_instructions_is_layer3() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_explain_project(root, "ffff0006", "claude", false, None);

        let mut buf = Vec::new();
        explain(root, "ffff0006", None, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let layer3_line = output.lines().find(|l| l.starts_with("layer 3:")).unwrap();
        assert!(
            layer3_line.contains("apm instructions"),
            "layer 3 should identify apm instructions; got: {layer3_line:?}"
        );
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

    // --- explain tests ---

    fn make_explain_project(root: &Path, ticket_id: &str, agent: &str, create_per_agent_file: bool, project: Option<&str>) {
        use std::fs;

        fs::create_dir_all(root.join(".apm")).unwrap();
        fs::create_dir_all(root.join("tickets")).unwrap();

        if create_per_agent_file {
            let agent_dir = root.join(format!(".apm/agents/{agent}"));
            fs::create_dir_all(&agent_dir).unwrap();
            fs::write(agent_dir.join("apm.coder.md"), format!("INSTRUCTIONS FOR {agent}")).unwrap();
        }

        let agents_section = match project {
            Some(path) => format!("\n[agents]\nproject = \"{path}\"\n"),
            None => String::new(),
        };

        fs::write(root.join(".apm/config.toml"), format!(r#"
[project]
name = "explain-test"
default_branch = "main"

[workers]
default = "{agent}/coder"

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
        assert!(output.contains(".apm/agents/mock-happy/apm.coder.md"), "should show per-agent path; got:\n{output}");
        assert_eq!(output.matches("not reached").count(), 2, "levels 1-2 should be not reached; got:\n{output}");
    }

    #[test]
    fn explain_level2_wins() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_explain_project(root, "bbbb0002", "claude", false, None);

        let mut buf = Vec::new();
        explain(root, "bbbb0002", None, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("level 2"), "should show level 2; got:\n{output}");
        assert!(output.contains("built-in default"), "should show built-in default; got:\n{output}");
    }

    #[test]
    fn explain_prefix_shown() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let project_path = ".apm/project.md";
        std::fs::create_dir_all(root.join(".apm")).unwrap();
        std::fs::write(root.join(project_path), "Project context.").unwrap();
        make_explain_project(root, "cccc0003", "claude", false, Some(project_path));

        let mut buf = Vec::new();
        explain(root, "cccc0003", None, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let layer2_line = output.lines().find(|l| l.starts_with("layer 2:")).unwrap();
        assert!(
            layer2_line.contains(project_path),
            "layer 2 line should name the configured file; got: {layer2_line:?}"
        );
    }

    #[test]
    fn explain_agent_role_override() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_explain_project(root, "dddd0004", "mock-happy", true, None);

        let mut buf = Vec::new();
        explain(root, "dddd0004", Some("claude"), Some("coder"), &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("level 2"), "claude override should fall to level 2; got:\n{output}");
        assert!(output.contains("built-in default (claude/coder)"), "should name claude/coder; got:\n{output}");
        assert!(output.contains(".apm/agents/claude/apm.coder.md"), "skipped level 0 should use overridden agent name; got:\n{output}");
    }

    #[test]
    fn run_message_contains_ticket_id() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_parity_project(root, "abcd0005");

        let mut buf = Vec::new();
        run_message(root, "abcd0005", None, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("abcd0005"), "user message should contain ticket id; got:\n{output}");
        assert!(output.contains("Worker agent"), "user message should contain role prefix; got:\n{output}");
    }

    #[test]
    fn run_full_contains_both_sections() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_parity_project(root, "abcd0006");

        let mut buf = Vec::new();
        run_full(root, "abcd0006", None, None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("=== system ==="), "full output should have system header; got:\n{output}");
        assert!(output.contains("=== user ==="), "full output should have user header; got:\n{output}");
        let sys_pos = output.find("=== system ===").unwrap();
        let usr_pos = output.find("=== user ===").unwrap();
        assert!(sys_pos < usr_pos, "system section should come before user section");
        assert!(output.contains("PER-AGENT INSTRUCTIONS"), "system prompt content should appear; got:\n{output}");
        assert!(output.contains("abcd0006"), "ticket id should appear in user section; got:\n{output}");
    }

    #[test]
    fn run_full_system_matches_run() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        make_parity_project(root, "abcd0007");

        let mut sys_buf = Vec::new();
        run(root, "abcd0007", None, None, &mut sys_buf).unwrap();
        let sys_only = String::from_utf8(sys_buf).unwrap();

        let mut full_buf = Vec::new();
        run_full(root, "abcd0007", None, None, &mut full_buf).unwrap();
        let full = String::from_utf8(full_buf).unwrap();

        let sys_section = full
            .split("=== user ===")
            .next()
            .unwrap()
            .trim_start_matches("=== system ===\n");
        assert_eq!(sys_section.trim_end(), sys_only.trim_end(),
            "system section in run_full must match run() output");
    }
}
