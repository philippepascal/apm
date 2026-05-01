use crate::config::{resolve_outcome, CompletionStrategy, Config, LocalConfig};
use crate::ticket_fmt::Ticket;
use crate::wrapper;
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::Path;

/// Return the completion strategy configured for the `in_progress → implemented`
/// transition.  Falls back to `None` when the transition is absent.
pub fn active_completion_strategy(config: &Config) -> CompletionStrategy {
    config.workflow.states.iter()
        .find(|s| s.id == "in_progress")
        .and_then(|s| s.transitions.iter().find(|t| t.to == "implemented"))
        .map(|t| t.completion.clone())
        .unwrap_or(CompletionStrategy::None)
}

fn strategy_name(strategy: &CompletionStrategy) -> &'static str {
    match strategy {
        CompletionStrategy::Pr => "pr",
        CompletionStrategy::Merge => "merge",
        CompletionStrategy::Pull => "pull",
        CompletionStrategy::PrOrEpicMerge => "pr_or_epic_merge",
        CompletionStrategy::None => "none",
    }
}

/// Validate that `dep_ids` satisfy the dependency rules for `strategy`.
///
/// - `ticket_epic`: epic ID of the ticket being written (None if no epic)
/// - `ticket_target_branch`: target_branch of the ticket (None = default branch)
/// - `dep_ids`: the proposed dependency list (empty slice → always Ok)
/// - `all_tickets`: all known tickets (used to look up dep metadata)
/// - `default_branch`: project default branch name
pub fn check_depends_on_rules(
    strategy: &CompletionStrategy,
    ticket_epic: Option<&str>,
    ticket_target_branch: Option<&str>,
    dep_ids: &[String],
    all_tickets: &[crate::ticket_fmt::Ticket],
    default_branch: &str,
) -> Result<()> {
    if dep_ids.is_empty() {
        return Ok(());
    }
    match strategy {
        CompletionStrategy::Pr | CompletionStrategy::None | CompletionStrategy::Pull => {
            bail!(
                "depends_on is not allowed under the {} completion strategy",
                strategy_name(strategy)
            );
        }
        CompletionStrategy::PrOrEpicMerge => {
            let Some(epic) = ticket_epic else {
                bail!(
                    "pr_or_epic_merge requires the ticket to belong to an epic before depends_on can be set"
                );
            };
            let mut offending: Vec<&str> = Vec::new();
            for dep_id in dep_ids {
                let dep = all_tickets.iter().find(|t| t.frontmatter.id == *dep_id)
                    .ok_or_else(|| anyhow::anyhow!("dep {dep_id} not found"))?;
                if dep.frontmatter.epic.as_deref() != Some(epic) {
                    offending.push(dep_id.as_str());
                }
            }
            if !offending.is_empty() {
                bail!(
                    "pr_or_epic_merge requires all deps to share epic {epic}; offending deps: {}",
                    offending.join(", ")
                );
            }
        }
        CompletionStrategy::Merge => {
            let ticket_target = ticket_target_branch.unwrap_or(default_branch);
            let mut offending: Vec<&str> = Vec::new();
            for dep_id in dep_ids {
                let dep = all_tickets.iter().find(|t| t.frontmatter.id == *dep_id)
                    .ok_or_else(|| anyhow::anyhow!("dep {dep_id} not found"))?;
                let dep_target = dep.frontmatter.target_branch.as_deref().unwrap_or(default_branch);
                if dep_target != ticket_target {
                    offending.push(dep_id.as_str());
                }
            }
            if !offending.is_empty() {
                bail!(
                    "merge requires all deps to share target_branch {ticket_target}; offending deps: {}",
                    offending.join(", ")
                );
            }
        }
    }
    Ok(())
}

/// Walk every non-closed ticket and return a vec of `(subject, message)` pairs
/// for each ticket whose `depends_on` violates the active completion strategy rule.
pub fn validate_depends_on(config: &Config, tickets: &[Ticket]) -> Vec<(String, String)> {
    let strategy = active_completion_strategy(config);
    let mut violations: Vec<(String, String)> = Vec::new();
    for ticket in tickets {
        let fm = &ticket.frontmatter;
        if fm.state == "closed" {
            continue;
        }
        let dep_ids = match &fm.depends_on {
            Some(deps) if !deps.is_empty() => deps,
            _ => continue,
        };
        if let Err(e) = check_depends_on_rules(
            &strategy,
            fm.epic.as_deref(),
            fm.target_branch.as_deref(),
            dep_ids,
            tickets,
            &config.project.default_branch,
        ) {
            violations.push((format!("#{}", fm.id), e.to_string()));
        }
    }
    violations
}

pub fn validate_owner(config: &Config, local: &LocalConfig, username: &str) -> Result<()> {
    if username == "-" {
        return Ok(());
    }
    let (collaborators, warnings) = crate::config::resolve_collaborators(config, local);
    for w in &warnings {
        #[allow(clippy::print_stderr)]
        { eprintln!("{w}"); }
    }
    if collaborators.is_empty() {
        return Ok(());
    }
    if collaborators.iter().any(|c| c == username) {
        return Ok(());
    }
    let list = collaborators.join(", ");
    bail!("unknown user '{username}'; valid collaborators: {list}");
}

fn is_external_worktree(dir: &Path) -> bool {
    let s = dir.to_string_lossy();
    s.starts_with('/') || s.starts_with("..")
}

fn gitignore_covers_dir(content: &str, dir: &str) -> bool {
    let normalized_dir = dir.trim_matches('/');
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .any(|line| line.trim_matches('/') == normalized_dir)
}

pub fn validate_config(config: &Config, root: &Path) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    let state_ids: HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();

    let section_names: HashSet<&str> = config.ticket.sections.iter()
        .map(|s| s.name.as_str())
        .collect();
    let has_sections = !section_names.is_empty();

    // Check whether any transition requires a provider.
    let needs_provider = config.workflow.states.iter()
        .flat_map(|s| s.transitions.iter())
        .any(|t| matches!(t.completion, CompletionStrategy::Pr | CompletionStrategy::Merge));

    let provider_ok = config.git_host.provider.as_ref()
        .map(|p| !p.is_empty())
        .unwrap_or(false);

    if needs_provider && !provider_ok {
        errors.push(
            "config: workflow — completion 'pr' or 'merge' requires [git_host] with a provider".into()
        );
    }

    // At least one non-terminal state.
    let has_non_terminal = config.workflow.states.iter().any(|s| !s.terminal);
    if !has_non_terminal {
        errors.push("config: workflow — no non-terminal state exists".into());
    }

    for state in &config.workflow.states {
        // Terminal state with outgoing transitions.
        if state.terminal && !state.transitions.is_empty() {
            errors.push(format!(
                "config: state.{} — terminal but has {} outgoing transition(s)",
                state.id,
                state.transitions.len()
            ));
        }

        // Non-terminal state with no outgoing transitions (tickets will be stranded).
        if !state.terminal && state.transitions.is_empty() {
            errors.push(format!(
                "config: state.{} — no outgoing transitions (tickets will be stranded)",
                state.id
            ));
        }

        // Instructions path exists on disk.
        if let Some(instructions) = &state.instructions {
            if !root.join(instructions).exists() {
                errors.push(format!(
                    "config: state.{}.instructions — file not found: {}",
                    state.id, instructions
                ));
            }
        }

        for transition in &state.transitions {
            // Transition target must exist.  "closed" is a built-in terminal state
            // that is always valid even when absent from [[workflow.states]].
            if transition.to != "closed" && !state_ids.contains(transition.to.as_str()) {
                errors.push(format!(
                    "config: state.{}.transition({}) — target state '{}' does not exist",
                    state.id, transition.to, transition.to
                ));
            }

            // context_section must match a known ticket section.
            if let Some(section) = &transition.context_section {
                if has_sections && !section_names.contains(section.as_str()) {
                    errors.push(format!(
                        "config: state.{}.transition({}).context_section — unknown section '{}'",
                        state.id, transition.to, section
                    ));
                }
            }

            // focus_section must match a known ticket section.
            if let Some(section) = &transition.focus_section {
                if has_sections && !section_names.contains(section.as_str()) {
                    errors.push(format!(
                        "config: state.{}.transition({}).focus_section — unknown section '{}'",
                        state.id, transition.to, section
                    ));
                }
            }

            // Merge/PrOrEpicMerge transitions require on_failure.
            if matches!(
                transition.completion,
                CompletionStrategy::Merge | CompletionStrategy::PrOrEpicMerge
            ) {
                if transition.on_failure.is_none() {
                    errors.push(format!(
                        "config: transition '{}' → '{}' uses completion '{}' but is missing \
                         `on_failure`; run `apm validate --fix` to add it",
                        state.id,
                        transition.to,
                        strategy_name(&transition.completion)
                    ));
                } else if let Some(ref name) = transition.on_failure {
                    if name != "closed" && !state_ids.contains(name.as_str()) {
                        errors.push(format!(
                            "config: transition '{}' → '{}' has `on_failure = \"{}\"` but \
                             state \"{}\" is not declared in workflow.toml",
                            state.id, transition.to, name, name
                        ));
                    }
                }
            }
        }
    }

    if !is_external_worktree(&config.worktrees.dir) {
        let dir_str = config.worktrees.dir.to_string_lossy();
        let gitignore = root.join(".gitignore");
        match std::fs::read_to_string(&gitignore) {
            Err(_) => errors.push(format!(
                "config: worktrees.dir '{dir_str}' is in-repo but .gitignore is missing; \
                 run 'apm init' or add '/{dir_str}/' manually"
            )),
            Ok(content) if !gitignore_covers_dir(&content, &dir_str) => errors.push(format!(
                "config: worktrees.dir '{dir_str}' is in-repo but .gitignore does not cover it; \
                 add '/{dir_str}/' or run 'apm init'"
            )),
            Ok(_) => {}
        }
    }

    errors
}

pub fn verify_tickets(
    root: &Path,
    config: &Config,
    tickets: &[Ticket],
    merged: &HashSet<String>,
) -> Vec<String> {
    let valid_states: HashSet<&str> = config.workflow.states.iter()
        .map(|s| s.id.as_str())
        .collect();
    let terminal = config.terminal_state_ids();

    let in_progress_states: HashSet<&str> =
        ["in_progress", "implemented"].iter().copied().collect();

    let worktree_states: HashSet<&str> =
        ["in_design", "in_progress"].iter().copied().collect();
    let main_root = crate::git_util::main_worktree_root(root)
        .unwrap_or_else(|| root.to_path_buf());
    let worktrees_base = main_root.join(&config.worktrees.dir);

    let mut issues: Vec<String> = Vec::new();

    for t in tickets {
        let fm = &t.frontmatter;

        // Skip terminal-state tickets.
        if terminal.contains(fm.state.as_str()) { continue; }

        let prefix = format!("#{} [{}]", fm.id, fm.state);

        // State value not in config.
        if !valid_states.is_empty() && !valid_states.contains(fm.state.as_str()) {
            issues.push(format!("{prefix}: unknown state {:?}", fm.state));
        }

        // Frontmatter id doesn't match filename numeric prefix.
        if let Some(name) = t.path.file_name().and_then(|n| n.to_str()) {
            let expected_prefix = format!("{:04}", fm.id);
            if !name.starts_with(&expected_prefix) {
                issues.push(format!("{prefix}: id {} does not match filename {name}", fm.id));
            }
        }

        // in_progress/implemented with no branch.
        if in_progress_states.contains(fm.state.as_str()) && fm.branch.is_none() {
            issues.push(format!("{prefix}: state requires branch but none set"));
        }

        // Branch merged but ticket not yet closed.
        if let Some(branch) = &fm.branch {
            if (fm.state == "in_progress" || fm.state == "implemented")
                && merged.contains(branch.as_str())
            {
                issues.push(format!("{prefix}: branch {branch} is merged but ticket not closed"));
            }
        }

        // in_design/in_progress with missing worktree directory.
        if worktree_states.contains(fm.state.as_str()) {
            if let Some(branch) = &fm.branch {
                let wt_name = branch.replace('/', "-");
                let wt_path = worktrees_base.join(&wt_name);
                if !wt_path.is_dir() {
                    issues.push(format!(
                        "{prefix}: worktree at {} is missing",
                        wt_path.display()
                    ));
                }
            }
        }

        // Missing ## Spec section.
        if !t.body.contains("## Spec") {
            issues.push(format!("{prefix}: missing ## Spec section"));
        }

        // Missing ## History section.
        if !t.body.contains("## History") {
            issues.push(format!("{prefix}: missing ## History section"));
        }

        // Validate document structure (required sections non-empty, AC items present).
        if let Ok(doc) = t.document() {
            for err in doc.validate(&config.ticket.sections) {
                issues.push(format!("{prefix}: {err}"));
            }
        }

        // Validate frontmatter agent names against known built-ins.
        let agents_to_check: Vec<&str> = fm.agent
            .as_deref()
            .into_iter()
            .chain(fm.agent_overrides.values().map(String::as_str))
            .collect();

        for name in agents_to_check {
            // TODO(2c32a282): upgrade to wrapper::resolve_wrapper(root, name) once
            // custom wrapper resolution lands so project-defined scripts referenced
            // in `agent` / `agent_overrides` are also validated here.
            if wrapper::resolve_builtin(name).is_none() {
                issues.push(format!(
                    "ticket {}: agent {:?} is not a known built-in",
                    fm.id, name
                ));
            }
        }
    }

    issues
}

pub fn validate_warnings(config: &crate::config::Config) -> Vec<String> {
    let mut warnings = config.load_warnings.clone();
    if let Some(container) = &config.workers.container {
        if !container.is_empty() {
            let docker_ok = std::process::Command::new("docker")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !docker_ok {
                warnings.push(
                    "workers.container is set but 'docker' is not in PATH".to_string()
                );
            }
        }
    }

    // Dead-end reachability check: warn when no agent-actionable state can reach a
    // transition whose outcome resolves to "success".
    let state_map: std::collections::HashMap<&str, &crate::config::StateConfig> =
        config.workflow.states.iter()
            .map(|s| (s.id.as_str(), s))
            .collect();

    let agent_startable: Vec<&str> = config.workflow.states.iter()
        .filter(|s| s.actionable.iter().any(|a| a == "agent" || a == "any"))
        .map(|s| s.id.as_str())
        .collect();

    if !agent_startable.is_empty() {
        let mut visited: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let mut queue: std::collections::VecDeque<&str> = std::collections::VecDeque::new();
        let mut found_success = false;

        for &start in &agent_startable {
            if visited.insert(start) {
                queue.push_back(start);
            }
        }

        'bfs: while let Some(state_id) = queue.pop_front() {
            if let Some(state) = state_map.get(state_id) {
                for t in &state.transitions {
                    let outcome = if let Some(target_state) = state_map.get(t.to.as_str()) {
                        resolve_outcome(t, target_state)
                    } else {
                        // Target not in map (e.g., built-in "closed" if not declared).
                        // Inline the resolve_outcome fallback treating unknown targets as terminal.
                        if let Some(ref o) = t.outcome {
                            o.as_str()
                        } else if t.completion != CompletionStrategy::None {
                            "success"
                        } else {
                            "cancelled"
                        }
                    };

                    if outcome == "success" {
                        found_success = true;
                        break 'bfs;
                    }

                    // Enqueue non-terminal target states for further exploration.
                    if let Some(target) = state_map.get(t.to.as_str()) {
                        if !target.terminal && visited.insert(t.to.as_str()) {
                            queue.push_back(t.to.as_str());
                        }
                    }
                }
            }
        }

        if !found_success {
            warnings.push(
                "workflow has no reachable 'success' outcome from any agent-actionable state; \
                 workers may never complete successfully".to_string()
            );
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, CompletionStrategy, LocalConfig};
    use crate::ticket::Ticket;
    use crate::git_util;
    use std::path::Path;
    use std::collections::HashSet;

    fn git_cmd(dir: &std::path::Path, args: &[&str]) {
        std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .status()
            .unwrap();
    }

    fn setup_verify_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();

        git_cmd(p, &["init", "-q", "-b", "main"]);
        git_cmd(p, &["config", "user.email", "test@test.com"]);
        git_cmd(p, &["config", "user.name", "test"]);

        std::fs::write(
            p.join("apm.toml"),
            r#"[project]
name = "test"

[tickets]
dir = "tickets"

[worktrees]
dir = "worktrees"

[[workflow.states]]
id = "in_design"
label = "In Design"

[[workflow.states]]
id = "in_progress"
label = "In Progress"

[[workflow.states]]
id = "specd"
label = "Specd"
"#,
        )
        .unwrap();

        git_cmd(p, &["add", "apm.toml"]);
        git_cmd(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init"]);

        dir
    }

    fn make_verify_ticket(root: &std::path::Path, id: &str, state: &str, branch: Option<&str>) -> Ticket {
        let branch_line = match branch {
            Some(b) => format!("branch = \"{b}\"\n"),
            None => String::new(),
        };
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"Test ticket\"\nstate = \"{state}\"\n{branch_line}+++\n\n## Spec\n\n## History\n"
        );
        let path = root.join("tickets").join(format!("{id}-test.md"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &raw).unwrap();
        Ticket::parse(&path, &raw).unwrap()
    }

    fn make_ticket(id: &str, epic: Option<&str>, target_branch: Option<&str>) -> Ticket {
        let epic_line = epic.map(|e| format!("epic = \"{e}\"\n")).unwrap_or_default();
        let target_line = target_branch.map(|b| format!("target_branch = \"{b}\"\n")).unwrap_or_default();
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"T\"\nstate = \"ready\"\n{epic_line}{target_line}+++\n\n"
        );
        Ticket::parse(Path::new(&format!("tickets/{id}-t.md")), &raw).unwrap()
    }

    fn strategy_config(completion: &str) -> Config {
        let toml = format!(
            r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

[[workflow.states.transitions]]
to         = "implemented"
completion = "{completion}"

[[workflow.states]]
id       = "implemented"
label    = "Implemented"
terminal = true
"#
        );
        toml::from_str(&toml).unwrap()
    }

    #[test]
    fn strategy_finds_in_progress_to_implemented() {
        let config = strategy_config("pr_or_epic_merge");
        assert_eq!(active_completion_strategy(&config), CompletionStrategy::PrOrEpicMerge);
    }

    #[test]
    fn strategy_defaults_to_none_when_absent() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to = "closed"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(active_completion_strategy(&config), CompletionStrategy::None);
    }

    #[test]
    fn dep_rules_pr_rejects_dep() {
        let dep = make_ticket("dep1", None, None);
        let result = check_depends_on_rules(
            &CompletionStrategy::Pr,
            None,
            None,
            &["dep1".to_string()],
            &[dep],
            "main",
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("pr"), "expected strategy name in: {msg}");
    }

    #[test]
    fn dep_rules_none_rejects_dep() {
        let dep = make_ticket("dep1", None, None);
        let result = check_depends_on_rules(
            &CompletionStrategy::None,
            None,
            None,
            &["dep1".to_string()],
            &[dep],
            "main",
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("none"), "expected strategy name in: {msg}");
    }

    #[test]
    fn dep_rules_pr_or_epic_merge_same_epic_ok() {
        let dep = make_ticket("dep1", Some("abc"), None);
        let result = check_depends_on_rules(
            &CompletionStrategy::PrOrEpicMerge,
            Some("abc"),
            None,
            &["dep1".to_string()],
            &[dep],
            "main",
        );
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[test]
    fn dep_rules_pr_or_epic_merge_different_epic_fails() {
        let dep = make_ticket("dep1", Some("xyz"), None);
        let result = check_depends_on_rules(
            &CompletionStrategy::PrOrEpicMerge,
            Some("abc"),
            None,
            &["dep1".to_string()],
            &[dep],
            "main",
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("dep1"), "expected dep ID in: {msg}");
    }

    #[test]
    fn dep_rules_pr_or_epic_merge_ticket_no_epic_fails() {
        let dep = make_ticket("dep1", Some("abc"), None);
        let result = check_depends_on_rules(
            &CompletionStrategy::PrOrEpicMerge,
            None,
            None,
            &["dep1".to_string()],
            &[dep],
            "main",
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("epic"), "expected epic mention in: {msg}");
    }

    #[test]
    fn dep_rules_merge_both_default_branch_ok() {
        let dep = make_ticket("dep1", None, None);
        let result = check_depends_on_rules(
            &CompletionStrategy::Merge,
            None,
            None,
            &["dep1".to_string()],
            &[dep],
            "main",
        );
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[test]
    fn dep_rules_merge_different_target_fails() {
        let dep = make_ticket("dep1", None, Some("epic/other"));
        let result = check_depends_on_rules(
            &CompletionStrategy::Merge,
            None,
            None,
            &["dep1".to_string()],
            &[dep],
            "main",
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("dep1"), "expected dep ID in: {msg}");
    }

    fn make_full_ticket(id: &str, state: &str, epic: Option<&str>, target_branch: Option<&str>, depends_on: &[&str]) -> Ticket {
        let epic_line = epic.map(|e| format!("epic = \"{e}\"\n")).unwrap_or_default();
        let target_line = target_branch.map(|b| format!("target_branch = \"{b}\"\n")).unwrap_or_default();
        let deps_line = if depends_on.is_empty() {
            String::new()
        } else {
            let quoted: Vec<String> = depends_on.iter().map(|d| format!("\"{d}\"")).collect();
            format!("depends_on = [{}]\n", quoted.join(", "))
        };
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"T\"\nstate = \"{state}\"\n{epic_line}{target_line}{deps_line}+++\n\n"
        );
        Ticket::parse(Path::new(&format!("tickets/{id}-t.md")), &raw).unwrap()
    }

    #[test]
    fn validate_depends_on_no_deps_clean() {
        let config = strategy_config("pr_or_epic_merge");
        let t1 = make_full_ticket("aa000001", "ready", Some("epic1"), None, &[]);
        let t2 = make_full_ticket("aa000002", "in_progress", Some("epic1"), None, &[]);
        let result = validate_depends_on(&config, &[t1, t2]);
        assert!(result.is_empty(), "expected no violations, got {result:?}");
    }

    #[test]
    fn validate_depends_on_closed_ticket_skipped() {
        let config = strategy_config("pr");
        let dep = make_full_ticket("bb000001", "closed", None, None, &[]);
        let ticket = make_full_ticket("bb000002", "closed", None, None, &["bb000001"]);
        let result = validate_depends_on(&config, &[dep, ticket]);
        assert!(result.is_empty(), "closed ticket should be skipped, got {result:?}");
    }

    #[test]
    fn validate_depends_on_pr_or_epic_merge_same_epic_ok() {
        let config = strategy_config("pr_or_epic_merge");
        let dep = make_full_ticket("cc000001", "ready", Some("abc"), None, &[]);
        let ticket = make_full_ticket("cc000002", "ready", Some("abc"), None, &["cc000001"]);
        let result = validate_depends_on(&config, &[dep, ticket]);
        assert!(result.is_empty(), "same-epic deps should pass, got {result:?}");
    }

    #[test]
    fn validate_depends_on_pr_or_epic_merge_cross_epic_fails() {
        let config = strategy_config("pr_or_epic_merge");
        let dep = make_full_ticket("dd000001", "ready", Some("xyz"), None, &[]);
        let ticket = make_full_ticket("dd000002", "ready", Some("abc"), None, &["dd000001"]);
        let result = validate_depends_on(&config, &[dep, ticket]);
        assert_eq!(result.len(), 1, "expected one violation, got {result:?}");
        assert!(result[0].1.contains("dd000001"), "message should mention dep ID: {}", result[0].1);
    }

    #[test]
    fn validate_depends_on_merge_same_target_ok() {
        let config = strategy_config("merge");
        let dep = make_full_ticket("ee000001", "ready", None, Some("feat"), &[]);
        let ticket = make_full_ticket("ee000002", "ready", None, Some("feat"), &["ee000001"]);
        let result = validate_depends_on(&config, &[dep, ticket]);
        assert!(result.is_empty(), "same-target deps should pass, got {result:?}");
    }

    #[test]
    fn validate_depends_on_merge_different_target_fails() {
        let config = strategy_config("merge");
        let dep = make_full_ticket("ff000001", "ready", None, Some("other"), &[]);
        let ticket = make_full_ticket("ff000002", "ready", None, Some("feat"), &["ff000001"]);
        let result = validate_depends_on(&config, &[dep, ticket]);
        assert_eq!(result.len(), 1, "expected one violation, got {result:?}");
        assert!(result[0].1.contains("ff000001"), "message should mention dep ID: {}", result[0].1);
    }

    #[test]
    fn validate_depends_on_pr_strategy_rejects_any_dep() {
        let config = strategy_config("pr");
        let dep = make_full_ticket("gg000001", "ready", None, None, &[]);
        let ticket = make_full_ticket("gg000002", "ready", None, None, &["gg000001"]);
        let result = validate_depends_on(&config, &[dep, ticket]);
        assert_eq!(result.len(), 1, "expected one violation, got {result:?}");
        assert!(result[0].1.contains("pr"), "message should mention strategy: {}", result[0].1);
    }

    fn load_config(toml: &str) -> Config {
        toml::from_str(toml).expect("config parse failed")
    }

    fn state_ids(config: &Config) -> std::collections::HashSet<&str> {
        config.workflow.states.iter().map(|s| s.id.as_str()).collect()
    }

    // Test 1: correct config passes all checks
    #[test]
    fn correct_config_passes() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to = "in_progress"

[[workflow.states]]
id       = "in_progress"
label    = "In Progress"
terminal = false

[[workflow.states.transitions]]
to = "closed"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    // Test 2: transition to non-existent state is detected
    #[test]
    fn transition_to_nonexistent_state_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to = "ghost"
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(errors.iter().any(|e| e.contains("ghost")), "expected ghost error in {errors:?}");
    }

    // Test 3: terminal state with outgoing transitions is detected
    #[test]
    fn terminal_state_with_transitions_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true

[[workflow.states.transitions]]
to = "new"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to = "closed"
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("state.closed") && e.contains("terminal")),
            "expected terminal error in {errors:?}"
        );
    }

    // Test 5: ticket with unknown state is detected
    #[test]
    fn ticket_with_unknown_state_detected() {
        use crate::ticket::Ticket;

        let raw = "+++\nid = 1\ntitle = \"Test\"\nstate = \"phantom\"\n+++\n\n## Spec\n";
        let ticket = Ticket::parse(Path::new("tickets/0001-test.md"), raw).unwrap();

        let known_states: std::collections::HashSet<&str> =
            ["new", "ready", "closed"].iter().copied().collect();

        assert!(!known_states.contains(ticket.frontmatter.state.as_str()));
    }

    // Test 6: dead-end non-terminal state is detected
    #[test]
    fn dead_end_non_terminal_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "stuck"
label = "Stuck"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("state.stuck") && e.contains("no outgoing transitions")),
            "expected dead-end error in {errors:?}"
        );
    }

    // Test 7: context_section mismatch is detected
    #[test]
    fn context_section_mismatch_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[ticket.sections]]
name = "Problem"
type = "free"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to              = "ready"
context_section = "NonExistent"

[[workflow.states]]
id    = "ready"
label = "Ready"

[[workflow.states.transitions]]
to = "closed"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("context_section") && e.contains("NonExistent")),
            "expected context_section error in {errors:?}"
        );
    }

    // Test 8: focus_section mismatch is detected
    #[test]
    fn focus_section_mismatch_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[ticket.sections]]
name = "Problem"
type = "free"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to             = "ready"
focus_section  = "BadSection"

[[workflow.states]]
id    = "ready"
label = "Ready"

[[workflow.states.transitions]]
to = "closed"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("focus_section") && e.contains("BadSection")),
            "expected focus_section error in {errors:?}"
        );
    }

    // Test 9: completion=pr without provider is detected
    #[test]
    fn completion_pr_without_provider_detected() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to         = "closed"
completion = "pr"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("provider")),
            "expected provider error in {errors:?}"
        );
    }

    // Test 10: completion=pr with provider configured passes
    #[test]
    fn completion_pr_with_provider_passes() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[git_host]
provider = "github"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to         = "closed"
completion = "pr"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            !errors.iter().any(|e| e.contains("provider")),
            "unexpected provider error in {errors:?}"
        );
    }

    // Test 11: context_section with empty ticket.sections is skipped
    #[test]
    fn context_section_skipped_when_no_sections_defined() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to              = "closed"
context_section = "AnySection"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#;
        let config = load_config(toml);
        let errors = validate_config(&config, Path::new("/tmp"));
        assert!(
            !errors.iter().any(|e| e.contains("context_section")),
            "unexpected context_section error in {errors:?}"
        );
    }

    // Test: closed state is not flagged as unknown even when absent from config
    #[test]
    fn closed_state_not_flagged_as_unknown() {
        use crate::ticket::Ticket;

        // Config with no "closed" state
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states.transitions]]
to = "done"

[[workflow.states]]
id       = "done"
label    = "Done"
terminal = true
"#;
        let config = load_config(toml);
        let state_ids: std::collections::HashSet<&str> = config.workflow.states.iter()
            .map(|s| s.id.as_str())
            .collect();

        let raw = "+++\nid = 1\ntitle = \"Test\"\nstate = \"closed\"\n+++\n\n## Spec\n";
        let ticket = Ticket::parse(Path::new("tickets/0001-test.md"), raw).unwrap();

        // "closed" is not in state_ids, but the validate logic skips it.
        assert!(!state_ids.contains("closed"));
        // Simulate the validate check: closed should be exempt.
        let fm = &ticket.frontmatter;
        let flagged = !state_ids.is_empty() && fm.state != "closed" && !state_ids.contains(fm.state.as_str());
        assert!(!flagged, "closed state should not be flagged as unknown");
    }

    // Test for state_ids helper (kept for compatibility)
    #[test]
    fn state_ids_helper() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "new"
label = "New"
"#;
        let config = load_config(toml);
        let ids = state_ids(&config);
        assert!(ids.contains("new"));
    }

    #[test]
    fn validate_warnings_no_container() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        let warnings = super::validate_warnings(&config);
        assert!(warnings.is_empty());
    }

    #[test]
    fn valid_collaborator_accepted() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        assert!(super::validate_owner(&config, &LocalConfig::default(), "alice").is_ok());
    }

    #[test]
    fn unknown_user_rejected() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        let err = super::validate_owner(&config, &LocalConfig::default(), "charlie").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown user 'charlie'"), "unexpected message: {msg}");
        assert!(msg.contains("alice, bob"), "unexpected message: {msg}");
    }

    #[test]
    fn empty_collaborators_skips_validation() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        assert!(super::validate_owner(&config, &LocalConfig::default(), "anyone").is_ok());
    }

    #[test]
    fn clear_owner_always_allowed() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice"]

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        assert!(super::validate_owner(&config, &LocalConfig::default(), "-").is_ok());
    }

    #[test]
    fn github_mode_known_user_accepted() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "org/repo"
"#;
        let config = load_config(toml);
        // No token in LocalConfig::default() — falls back to project.collaborators
        assert!(super::validate_owner(&config, &LocalConfig::default(), "alice").is_ok());
    }

    #[test]
    fn github_mode_unknown_user_rejected() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "org/repo"
"#;
        let config = load_config(toml);
        // No token — falls back to project.collaborators; charlie is not in the list
        let err = super::validate_owner(&config, &LocalConfig::default(), "charlie").unwrap_err();
        assert!(err.to_string().contains("charlie"), "expected charlie in: {err}");
    }

    #[test]
    fn github_mode_no_collaborators_skips_check() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "org/repo"
"#;
        let config = load_config(toml);
        // Empty collaborators list — no validation
        assert!(super::validate_owner(&config, &LocalConfig::default(), "anyone").is_ok());
    }

    #[test]
    fn github_mode_clear_owner_accepted() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice"]

[tickets]
dir = "tickets"

[git_host]
provider = "github"
repo = "org/repo"
"#;
        let config = load_config(toml);
        assert!(super::validate_owner(&config, &LocalConfig::default(), "-").is_ok());
    }

    #[test]
    fn non_github_mode_unknown_user_rejected() {
        let toml = r#"
[project]
name = "test"
collaborators = ["alice", "bob"]

[tickets]
dir = "tickets"
"#;
        let config = load_config(toml);
        let err = super::validate_owner(&config, &LocalConfig::default(), "charlie").unwrap_err();
        assert!(err.to_string().contains("charlie"), "expected charlie in: {err}");
    }

    #[test]
    fn validate_warnings_empty_container() {
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[workers]
container = ""
"#;
        let config = load_config(toml);
        let warnings = super::validate_warnings(&config);
        assert!(warnings.is_empty(), "empty container string should not warn");
    }

    #[test]
    fn dead_end_workflow_warning_emitted() {
        // A workflow where the only agent-actionable state cycles back to itself
        // with no completion strategy — no "success" outcome is reachable.
        let toml = r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id         = "start"
label      = "Start"
actionable = ["agent"]

[[workflow.states.transitions]]
to = "middle"

[[workflow.states]]
id    = "middle"
label = "Middle"

[[workflow.states.transitions]]
to = "start"
"#;
        let config = load_config(toml);
        let warnings = super::validate_warnings(&config);
        assert!(
            warnings.iter().any(|w| w.contains("success")),
            "expected dead-end warning containing 'success'; got: {warnings:?}"
        );
    }

    #[test]
    fn default_workflow_no_dead_end_warning() {
        // The default workflow has in_progress → implemented with completion = pr_or_epic_merge,
        // reachable from the agent-actionable "ready" state. No dead-end warning should fire.
        let base = r#"
[project]
name = "test"

[tickets]
dir = "tickets"
"#;
        let combined = format!("{}\n{}", base, crate::init::default_workflow_toml());
        let config: Config = toml::from_str(&combined).unwrap();
        let warnings = super::validate_warnings(&config);
        assert!(
            !warnings.iter().any(|w| w.contains("no reachable") && w.contains("success")),
            "unexpected dead-end warning for default workflow; got: {warnings:?}"
        );
    }

    #[test]
    fn worktree_missing_in_design() {
        let dir = setup_verify_repo();
        let root = dir.path();
        let config = Config::load(root).unwrap();
        let ticket = make_verify_ticket(root, "abcd1234", "in_design", Some("ticket/abcd1234-test"));

        let issues = verify_tickets(root, &config, &[ticket], &HashSet::new());

        let main_root = git_util::main_worktree_root(root).unwrap_or_else(|| root.to_path_buf());
        let wt_path = main_root.join("worktrees").join("ticket-abcd1234-test");
        let expected = format!(
            "#abcd1234 [in_design]: worktree at {} is missing",
            wt_path.display()
        );
        assert!(
            issues.iter().any(|i| i == &expected),
            "expected worktree missing issue; got: {issues:?}"
        );
    }

    #[test]
    fn worktree_present_no_issue() {
        let dir = setup_verify_repo();
        let root = dir.path();
        let config = Config::load(root).unwrap();
        let ticket = make_verify_ticket(root, "abcd1234", "in_design", Some("ticket/abcd1234-test"));

        std::fs::create_dir_all(root.join("worktrees").join("ticket-abcd1234-test")).unwrap();

        let issues = verify_tickets(root, &config, &[ticket], &HashSet::new());
        assert!(
            !issues.iter().any(|i| i.contains("worktree")),
            "unexpected worktree issue; got: {issues:?}"
        );
    }

    #[test]
    fn worktree_check_skipped_for_other_states() {
        let dir = setup_verify_repo();
        let root = dir.path();
        let config = Config::load(root).unwrap();
        let ticket = make_verify_ticket(root, "abcd1234", "specd", Some("ticket/abcd1234-test"));

        let issues = verify_tickets(root, &config, &[ticket], &HashSet::new());
        assert!(
            !issues.iter().any(|i| i.contains("worktree")),
            "unexpected worktree issue for specd state; got: {issues:?}"
        );
    }

    fn in_repo_wt_config(dir: &str) -> Config {
        let toml = format!(
            r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[worktrees]
dir = "{dir}"
"#
        );
        toml::from_str(&toml).expect("config parse failed")
    }

    #[test]
    fn validate_config_gitignore_missing_in_repo_wt() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = in_repo_wt_config("worktrees");
        let errors = validate_config(&config, tmp.path());
        assert!(
            errors.iter().any(|e| e.contains("worktrees") && e.contains(".gitignore")),
            "expected gitignore missing error; got: {errors:?}"
        );
    }

    #[test]
    fn validate_config_gitignore_covered_anchored_slash() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "/worktrees/\n").unwrap();
        let config = in_repo_wt_config("worktrees");
        let errors = validate_config(&config, tmp.path());
        assert!(
            !errors.iter().any(|e| e.contains("gitignore")),
            "unexpected gitignore error; got: {errors:?}"
        );
    }

    #[test]
    fn validate_config_gitignore_covered_anchored_no_slash() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "/worktrees\n").unwrap();
        let config = in_repo_wt_config("worktrees");
        let errors = validate_config(&config, tmp.path());
        assert!(
            !errors.iter().any(|e| e.contains("gitignore")),
            "unexpected gitignore error; got: {errors:?}"
        );
    }

    #[test]
    fn validate_config_gitignore_covered_unanchored_slash() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "worktrees/\n").unwrap();
        let config = in_repo_wt_config("worktrees");
        let errors = validate_config(&config, tmp.path());
        assert!(
            !errors.iter().any(|e| e.contains("gitignore")),
            "unexpected gitignore error; got: {errors:?}"
        );
    }

    #[test]
    fn validate_config_gitignore_covered_bare() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "worktrees\n").unwrap();
        let config = in_repo_wt_config("worktrees");
        let errors = validate_config(&config, tmp.path());
        assert!(
            !errors.iter().any(|e| e.contains("gitignore")),
            "unexpected gitignore error; got: {errors:?}"
        );
    }

    #[test]
    fn validate_config_gitignore_not_covered() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "node_modules\n").unwrap();
        let config = in_repo_wt_config("worktrees");
        let errors = validate_config(&config, tmp.path());
        assert!(
            errors.iter().any(|e| e.contains("worktrees") && e.contains("gitignore")),
            "expected gitignore not covered error; got: {errors:?}"
        );
    }

    #[test]
    fn validate_config_gitignore_no_false_positive() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "wt-old/\n").unwrap();
        let config = in_repo_wt_config("wt");
        let errors = validate_config(&config, tmp.path());
        assert!(
            errors.iter().any(|e| e.contains("wt") && e.contains("gitignore")),
            "wt-old should not match wt; got: {errors:?}"
        );
    }

    #[test]
    fn validate_config_external_dotdot_no_check() {
        let tmp = tempfile::TempDir::new().unwrap();
        // No .gitignore at all
        let config = in_repo_wt_config("../ext");
        let errors = validate_config(&config, tmp.path());
        assert!(
            !errors.iter().any(|e| e.contains("gitignore")),
            "external dotdot path should skip gitignore check; got: {errors:?}"
        );
    }

    #[test]
    fn validate_config_external_absolute_no_check() {
        let tmp = tempfile::TempDir::new().unwrap();
        // No .gitignore at all
        let config = in_repo_wt_config("/abs/path");
        let errors = validate_config(&config, tmp.path());
        assert!(
            !errors.iter().any(|e| e.contains("gitignore")),
            "absolute path should skip gitignore check; got: {errors:?}"
        );
    }

    fn config_with_merge_transition(completion: &str, on_failure: Option<&str>, declare_failure_state: bool) -> Config {
        let on_failure_line = on_failure
            .map(|v| format!("on_failure = \"{v}\"\n"))
            .unwrap_or_default();
        let merge_failed_state = if declare_failure_state {
            r#"
[[workflow.states]]
id       = "merge_failed"
label    = "Merge failed"

[[workflow.states.transitions]]
to = "closed"
"#
        } else {
            ""
        };
        let toml = format!(
            r#"
[project]
name = "test"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

[[workflow.states.transitions]]
to         = "implemented"
completion = "{completion}"
{on_failure_line}
[[workflow.states]]
id       = "implemented"
label    = "Implemented"
terminal = true

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
{merge_failed_state}
"#
        );
        toml::from_str(&toml).expect("config parse failed")
    }

    #[test]
    fn test_on_failure_missing_for_merge() {
        let config = config_with_merge_transition("merge", None, false);
        let errors = validate_config(&config, std::path::Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("missing `on_failure`")),
            "expected missing on_failure error; got: {errors:?}"
        );
    }

    #[test]
    fn test_on_failure_missing_for_pr_or_epic_merge() {
        // No ticket with target_branch — rule fires on transition definition alone.
        let config = config_with_merge_transition("pr_or_epic_merge", None, false);
        let errors = validate_config(&config, std::path::Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("missing `on_failure`")),
            "expected missing on_failure error for pr_or_epic_merge; got: {errors:?}"
        );
    }

    #[test]
    fn test_on_failure_unknown_state() {
        let config = config_with_merge_transition("merge", Some("ghost_state"), false);
        let errors = validate_config(&config, std::path::Path::new("/tmp"));
        assert!(
            errors.iter().any(|e| e.contains("ghost_state")),
            "expected unknown state error for ghost_state; got: {errors:?}"
        );
    }

    #[test]
    fn test_on_failure_valid() {
        let config = config_with_merge_transition("merge", Some("merge_failed"), true);
        let errors = validate_config(&config, std::path::Path::new("/tmp"));
        let on_failure_errors: Vec<&String> = errors.iter()
            .filter(|e| e.contains("on_failure") || e.contains("ghost_state") || e.contains("merge_failed"))
            .collect();
        assert!(
            on_failure_errors.is_empty(),
            "unexpected on_failure errors: {on_failure_errors:?}"
        );
    }

    // --- frontmatter agent validation ---

    fn make_agent_verify_ticket(root: &std::path::Path, id: &str, state: &str, extra_fm: &str) -> Ticket {
        let raw = format!(
            "+++\nid = \"{id}\"\ntitle = \"Test ticket\"\nstate = \"{state}\"\n{extra_fm}+++\n\n## Spec\n\n## History\n"
        );
        let path = root.join("tickets").join(format!("{id}-test.md"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &raw).unwrap();
        Ticket::parse(&path, &raw).unwrap()
    }

    #[test]
    fn validate_unknown_frontmatter_agent_is_error() {
        let dir = setup_verify_repo();
        let root = dir.path();
        let config = Config::load(root).unwrap();
        let ticket = make_agent_verify_ticket(root, "abcd1234", "specd", "agent = \"nonexistent-bot\"\n");

        let issues = verify_tickets(root, &config, &[ticket], &HashSet::new());

        assert!(
            issues.iter().any(|i| i.contains("abcd1234") && i.contains("nonexistent-bot")),
            "expected error with ticket id and agent name; got: {issues:?}"
        );
    }

    #[test]
    fn validate_unknown_agent_in_overrides_is_error() {
        let dir = setup_verify_repo();
        let root = dir.path();
        let config = Config::load(root).unwrap();
        let ticket = make_agent_verify_ticket(
            root,
            "abcd1234",
            "specd",
            "[agent_overrides]\nimpl_agent = \"nonexistent-bot\"\n",
        );

        let issues = verify_tickets(root, &config, &[ticket], &HashSet::new());

        assert!(
            issues.iter().any(|i| i.contains("abcd1234") && i.contains("nonexistent-bot")),
            "expected error with ticket id and agent name; got: {issues:?}"
        );
    }

    #[test]
    fn validate_known_frontmatter_agent_passes() {
        let dir = setup_verify_repo();
        let root = dir.path();
        let config = Config::load(root).unwrap();
        let ticket = make_agent_verify_ticket(root, "abcd1234", "specd", "agent = \"claude\"\n");

        let issues = verify_tickets(root, &config, &[ticket], &HashSet::new());

        assert!(
            !issues.iter().any(|i| i.contains("is not a known built-in")),
            "expected no agent error for known built-in; got: {issues:?}"
        );
    }
}
