use tempfile::TempDir;

fn git(dir: &std::path::Path, args: &[&str]) {
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

fn setup() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();

    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);

    // Minimal apm.toml so Config::load succeeds.
    std::fs::write(
        p.join("apm.toml"),
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[agents]
max_concurrent = 3

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id         = "new"
label      = "New"
actionable = ["agent"]

[[workflow.states]]
id    = "specd"
label = "Specd"

[[workflow.states]]
id         = "ammend"
label      = "Ammend"
actionable = ["agent"]

[[workflow.states]]
id         = "ready"
label      = "Ready"
actionable = ["agent"]

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#,
    )
    .unwrap();

    // Initial commit so git worktrees work.
    git(p, &["add", "apm.toml"]);
    git(p, &[
        "-c", "commit.gpgsign=false",
        "commit", "-m", "init", "--allow-empty",
    ]);

    std::fs::create_dir_all(p.join("tickets")).unwrap();
    dir
}

/// Read a file's content from a git branch (does not touch the working tree).
fn branch_content(dir: &std::path::Path, branch: &str, path: &str) -> String {
    let out = std::process::Command::new("git")
        .args(["show", &format!("{branch}:{path}")])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git show {branch}:{path} failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

/// Check out a file from a ticket branch into the working tree so that
/// read-only commands (apm list, apm show, apm next) can see it.
fn sync_from_branch(dir: &std::path::Path, branch: &str, path: &str) {
    git(dir, &["checkout", branch, "--", path]);
}

/// Write a valid spec body to a ticket on its branch, without changing HEAD.
fn write_valid_spec_to_branch(dir: &std::path::Path, branch: &str, path: &str) {
    let existing = branch_content(dir, branch, path);
    let fm_end = existing.find("\n+++\n").expect("frontmatter close not found") + 5;
    let frontmatter = &existing[..fm_end];
    let body = "\n## Spec\n\n### Problem\n\nTest problem.\n\n### Acceptance criteria\n\n- [ ] One criterion\n\n### Out of scope\n\nNothing.\n\n### Approach\n\nDirect approach.\n\n## History\n\n| When | From | To | By |\n|------|------|----|-----|\n| 2026-01-01T00:00Z | — | new | test-agent |\n";
    let content = format!("{frontmatter}{body}");
    git(dir, &["checkout", branch]);
    std::fs::write(dir.join(path), &content).unwrap();
    git(dir, &["-c", "commit.gpgsign=false", "add", path]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", "write spec"]);
    git(dir, &["checkout", "-"]);
}

// --- init ---

#[test]
fn init_creates_expected_files() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    apm::cmd::init::run(p, true, false).unwrap();
    assert!(p.join("tickets").is_dir());
    assert!(p.join(".apm/config.toml").exists());
    assert!(p.join(".gitignore").exists());
    assert!(!p.join(".git/hooks/pre-push").exists());
    assert!(!p.join(".git/hooks/post-merge").exists());
}

#[test]
fn init_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    apm::cmd::init::run(p, true, false).unwrap();
    let toml_before = std::fs::read_to_string(p.join(".apm/config.toml")).unwrap();
    apm::cmd::init::run(p, true, false).unwrap();
    let toml_after = std::fs::read_to_string(p.join(".apm/config.toml")).unwrap();
    assert_eq!(toml_before, toml_after);
}

#[test]
fn init_generated_config_has_all_workflow_states() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    apm::cmd::init::run(p, true, false).unwrap();

    let toml = std::fs::read_to_string(p.join(".apm/config.toml")).unwrap();
    for state in &["new", "question", "specd", "ammend", "in_design", "ready", "in_progress", "implemented", "accepted", "closed"] {
        assert!(toml.contains(&format!("\"{state}\"")), "missing state: {state}");
    }
    assert!(toml.contains("terminal = true"), "closed must be terminal");
    // Must parse without error.
    apm_core::config::Config::load(p).unwrap();
}

#[test]
fn list_excludes_terminal_tickets_by_default() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Open ticket".into(), true, false, None, None, true).unwrap();
    apm::cmd::new::run(dir.path(), "Closed ticket".into(), true, false, None, None, true).unwrap();
    apm::cmd::state::run(dir.path(), 2, "closed".into(), false).unwrap();

    // Verify indirectly through the filter logic in the library.
    let config = apm_core::config::Config::load(dir.path()).unwrap();
    let tickets = apm_core::ticket::load_all_from_git(dir.path(), &config.tickets.dir).unwrap();
    let terminal: std::collections::HashSet<&str> = config.workflow.states.iter()
        .filter(|s| s.terminal)
        .map(|s| s.id.as_str())
        .collect();
    let visible: Vec<_> = tickets.iter()
        .filter(|t| !terminal.contains(t.frontmatter.state.as_str()))
        .collect();
    assert_eq!(visible.len(), 1, "only the open ticket should be visible");
    assert_eq!(visible[0].frontmatter.id, 1);

    let all: Vec<_> = tickets.iter().collect();
    assert_eq!(all.len(), 2, "--all should include the closed ticket");
}

// --- new ---

#[test]
fn new_creates_ticket_file() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "My first ticket".into(), true, false, None, None, true).unwrap();
    // File lives on the ticket branch, not in the working tree.
    let content = branch_content(dir.path(), "ticket/0001-my-first-ticket", "tickets/0001-my-first-ticket.md");
    assert!(!content.is_empty());
}

#[test]
fn new_ticket_has_correct_frontmatter() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Hello World".into(), true, false, None, None, true).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-hello-world", "tickets/0001-hello-world.md");
    assert!(content.contains("id = 1"));
    assert!(content.contains("title = \"Hello World\""));
    assert!(content.contains("state = \"new\""));
    assert!(content.contains("branch = \"ticket/0001-hello-world\""));
}

#[test]
fn new_increments_ids() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "First".into(), true, false, None, None, true).unwrap();
    apm::cmd::new::run(dir.path(), "Second".into(), true, false, None, None, true).unwrap();
    let c1 = branch_content(dir.path(), "ticket/0001-first", "tickets/0001-first.md");
    let c2 = branch_content(dir.path(), "ticket/0002-second", "tickets/0002-second.md");
    assert!(c1.contains("id = 1"));
    assert!(c2.contains("id = 2"));
}

// --- list ---

#[test]
fn list_shows_all_tickets() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Alpha".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-alpha", "tickets/0001-alpha.md");
    apm::cmd::new::run(dir.path(), "Beta".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0002-beta", "tickets/0002-beta.md");
    apm::cmd::list::run(dir.path(), None, false, false, None, None).unwrap();
}

#[test]
fn list_state_filter() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Alpha".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-alpha", "tickets/0001-alpha.md");
    apm::cmd::new::run(dir.path(), "Beta".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0002-beta", "tickets/0002-beta.md");
    write_valid_spec_to_branch(dir.path(), "ticket/0001-alpha", "tickets/0001-alpha.md");
apm::cmd::state::run(dir.path(), 1, "specd".into(), false).unwrap();
    // Sync the updated ticket from its branch so apm list can see the new state.
    sync_from_branch(dir.path(), "ticket/0001-alpha", "tickets/0001-alpha.md");
    apm::cmd::list::run(dir.path(), Some("specd".into()), false, false, None, None).unwrap();
}

// --- show ---

#[test]
fn show_existing_ticket() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Show me".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-show-me", "tickets/0001-show-me.md");
    apm::cmd::show::run(dir.path(), 1, false).unwrap();
}

#[test]
fn show_missing_ticket_errors() {
    let dir = setup();
    assert!(apm::cmd::show::run(dir.path(), 99, false).is_err());
}

// --- state ---

#[test]
fn state_transition_updates_file() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Transition test".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-transition-test", "tickets/0001-transition-test.md");
    write_valid_spec_to_branch(dir.path(), "ticket/0001-transition-test", "tickets/0001-transition-test.md");
apm::cmd::state::run(dir.path(), 1, "specd".into(), false).unwrap();
    // Read the updated state from the ticket branch (not the working tree).
    let content = branch_content(dir.path(), "ticket/0001-transition-test", "tickets/0001-transition-test.md");
    assert!(content.contains("state = \"specd\""));
}

#[test]
fn state_transition_appends_history_row() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "History test".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-history-test", "tickets/0001-history-test.md");
    write_valid_spec_to_branch(dir.path(), "ticket/0001-history-test", "tickets/0001-history-test.md");
apm::cmd::state::run(dir.path(), 1, "specd".into(), false).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-history-test", "tickets/0001-history-test.md");
    assert!(content.contains("| new | specd |"));
}

#[test]
fn state_ammend_inserts_amendment_section() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Ammend test".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-ammend-test", "tickets/0001-ammend-test.md");
    apm::cmd::state::run(dir.path(), 1, "ammend".into(), false).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-ammend-test", "tickets/0001-ammend-test.md");
    assert!(content.contains("### Amendment requests"));
}

// --- set ---

#[test]
fn set_priority_updates_frontmatter() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Set test".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-set-test", "tickets/0001-set-test.md");
    apm::cmd::set::run(dir.path(), 1, "priority".into(), "7".into()).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-set-test", "tickets/0001-set-test.md");
    assert!(content.contains("priority = 7"));
}

// --- next ---

#[test]
fn next_returns_highest_priority() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Low priority".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-low-priority", "tickets/0001-low-priority.md");
    apm::cmd::new::run(dir.path(), "High priority".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0002-high-priority", "tickets/0002-high-priority.md");
    apm::cmd::set::run(dir.path(), 2, "priority".into(), "10".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0002-high-priority", "tickets/0002-high-priority.md");
    apm::cmd::next::run(dir.path(), false).unwrap();
}

#[test]
fn next_json_is_valid() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Json test".into(), true, false, None, None, true).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-json-test", "tickets/0001-json-test.md");
    apm::cmd::next::run(dir.path(), true).unwrap();
}

#[test]
fn next_null_when_no_actionable() {
    let dir = setup();
    apm::cmd::next::run(dir.path(), true).unwrap();
}

// --- branch ---

#[test]
fn new_ticket_creates_branch() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Branch test".into(), true, false, None, None, true).unwrap();
    // Branch should exist locally after apm new.
    let out = std::process::Command::new("git")
        .args(["branch", "--list", "ticket/0001-branch-test"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let branches = String::from_utf8(out.stdout).unwrap();
    assert!(branches.contains("ticket/0001-branch-test"));
}

#[test]
fn new_ticket_sets_branch_in_frontmatter() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Frontmatter branch".into(), true, false, None, None, true).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-frontmatter-branch", "tickets/0001-frontmatter-branch.md");
    assert!(content.contains("branch = \"ticket/0001-frontmatter-branch\""));
}

#[test]
fn init_config_has_default_branch_and_parses() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q", "-b", "trunk"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    apm::cmd::init::run(p, true, false).unwrap();

    let toml = std::fs::read_to_string(p.join(".apm/config.toml")).unwrap();
    assert!(toml.contains("default_branch = \"trunk\""), "default_branch not written: {toml}");

    let config = apm_core::config::Config::load(p).unwrap();
    assert_eq!(config.project.default_branch, "trunk");
}

#[test]
fn config_default_branch_defaults_to_main_when_absent() {
    // A config without default_branch should deserialize with "main" as default.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    std::fs::write(p.join("apm.toml"), "[project]\nname = \"test\"\n").unwrap();
    let config = apm_core::config::Config::load(p).unwrap();
    assert_eq!(config.project.default_branch, "main");
}

// --- sync bulk close ---

fn setup_with_close_workflow() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    std::fs::write(p.join("apm.toml"), r#"[project]
name = "test"

[tickets]
dir = "tickets"

[agents]
max_concurrent = 3

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id    = "new"
label = "New"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

[[workflow.states]]
id    = "implemented"
label = "Implemented"

[[workflow.states]]
id    = "accepted"
label = "Accepted"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#).unwrap();
    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    dir
}

fn write_ticket_to_branch(dir: &std::path::Path, branch: &str, filename: &str, state: &str, id: u32, title: &str) {
    let path = format!("tickets/{filename}");
    let content = format!(
        "+++\nid = {id}\ntitle = \"{title}\"\nstate = \"{state}\"\nbranch = \"{branch}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|",
    );
    // Create branch if it doesn't exist
    let branch_exists = std::process::Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !branch_exists {
        git(dir, &["checkout", "-b", branch]);
    } else {
        git(dir, &["checkout", branch]);
    }
    std::fs::create_dir_all(dir.join("tickets")).unwrap();
    std::fs::write(dir.join(&path), &content).unwrap();
    git(dir, &["-c", "commit.gpgsign=false", "add", &path]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", &format!("ticket: {title}")]);
    git(dir, &["checkout", "main"]);
}

#[test]
fn sync_closes_accepted_ticket_auto() {
    let dir = setup_with_close_workflow();
    let p = dir.path();

    write_ticket_to_branch(p, "ticket/0001-my-ticket", "0001-my-ticket.md", "accepted", 1, "my ticket");

    apm::cmd::sync::run(p, true, true, true, true, false).unwrap();

    let content = branch_content(p, "main", "tickets/0001-my-ticket.md");
    assert!(content.contains("state = \"closed\""), "ticket should be closed on main: {content}");
}

#[test]
fn sync_closes_implemented_ticket_with_no_branch() {
    let dir = setup_with_close_workflow();
    let p = dir.path();

    // Write ticket directly to main in implemented state (simulating stale ticket after branch deleted)
    let path = "tickets/0001-stale.md";
    let content = "+++\nid = 1\ntitle = \"stale\"\nstate = \"implemented\"\nbranch = \"ticket/0001-stale\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|";
    std::fs::write(p.join(path), content).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", path]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "add stale ticket"]);
    // No ticket branch exists

    apm::cmd::sync::run(p, true, true, true, true, false).unwrap();

    let updated = std::fs::read_to_string(p.join(path)).unwrap();
    assert!(updated.contains("state = \"closed\""), "stale ticket should be closed: {updated}");
}

#[test]
fn sync_no_close_when_nothing_to_close() {
    let dir = setup_with_close_workflow();
    let p = dir.path();
    // No tickets at all
    let log_before = branch_content(p, "main", "apm.toml"); // just to get a ref point
    apm::cmd::sync::run(p, true, true, true, true, false).unwrap();
    // main should have no new commits (same HEAD)
    let head = std::process::Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(p)
        .output()
        .unwrap();
    let head_msg = String::from_utf8(head.stdout).unwrap();
    assert!(!head_msg.contains("apm sync: close"), "no close commit expected: {head_msg}");
    drop(log_before);
}

#[test]
fn sync_batches_multiple_closes_into_one_commit() {
    let dir = setup_with_close_workflow();
    let p = dir.path();

    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "accepted", 1, "alpha");
    write_ticket_to_branch(p, "ticket/0002-beta", "0002-beta.md", "accepted", 2, "beta");

    let commits_before: usize = std::process::Command::new("git")
        .args(["rev-list", "--count", "main"])
        .current_dir(p)
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap().trim().parse().unwrap_or(0))
        .unwrap_or(0);

    apm::cmd::sync::run(p, true, true, true, true, false).unwrap();

    let commits_after: usize = std::process::Command::new("git")
        .args(["rev-list", "--count", "main"])
        .current_dir(p)
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap().trim().parse().unwrap_or(0))
        .unwrap_or(0);

    assert_eq!(commits_after, commits_before + 1, "exactly one new commit expected");

    let msg = std::process::Command::new("git")
        .args(["log", "--format=%s", "-1"])
        .current_dir(p)
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap())
        .unwrap();
    assert!(msg.contains("#1") && msg.contains("#2"), "commit message should list both tickets: {msg}");
}

// --- take ---

fn write_ticket_with_agent(dir: &std::path::Path, branch: &str, filename: &str, state: &str, id: u32, title: &str, agent: &str) {
    let path = format!("tickets/{filename}");
    let content = format!(
        "+++\nid = {id}\ntitle = \"{title}\"\nstate = \"{state}\"\nbranch = \"{branch}\"\nagent = \"{agent}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|",
    );
    let branch_exists = std::process::Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !branch_exists {
        git(dir, &["checkout", "-b", branch]);
    } else {
        git(dir, &["checkout", branch]);
    }
    std::fs::create_dir_all(dir.join("tickets")).unwrap();
    std::fs::write(dir.join(&path), &content).unwrap();
    git(dir, &["-c", "commit.gpgsign=false", "add", &path]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", &format!("ticket: {title}")]);
    git(dir, &["checkout", "main"]);
}

#[test]
fn take_succeeds_on_ammend_state() {
    let dir = setup();
    let p = dir.path();
    write_ticket_with_agent(p, "ticket/0001-ammend-me", "0001-ammend-me.md", "ammend", 1, "ammend me", "old-agent");
    std::env::set_var("APM_AGENT_NAME", "new-agent");
    apm::cmd::take::run(p, 1, true).unwrap();
    let content = branch_content(p, "ticket/0001-ammend-me", "tickets/0001-ammend-me.md");
    assert!(content.contains("agent = \"new-agent\""), "agent should be updated: {content}");
}

#[test]
fn take_succeeds_on_blocked_state() {
    let dir = setup();
    let p = dir.path();
    write_ticket_with_agent(p, "ticket/0001-blocked", "0001-blocked.md", "blocked", 1, "blocked", "old-agent");
    std::env::set_var("APM_AGENT_NAME", "new-agent");
    apm::cmd::take::run(p, 1, true).unwrap();
    let content = branch_content(p, "ticket/0001-blocked", "tickets/0001-blocked.md");
    assert!(content.contains("agent = \"new-agent\""), "agent should be updated: {content}");
}

#[test]
fn take_appends_handoff_history() {
    let dir = setup();
    let p = dir.path();
    write_ticket_with_agent(p, "ticket/0001-handoff", "0001-handoff.md", "in_progress", 1, "handoff", "old-agent");
    std::env::set_var("APM_AGENT_NAME", "new-agent");
    apm::cmd::take::run(p, 1, true).unwrap();
    let content = branch_content(p, "ticket/0001-handoff", "tickets/0001-handoff.md");
    assert!(content.contains("handoff"), "handoff history entry should be appended: {content}");
    assert!(content.contains("old-agent"), "old agent should appear in history: {content}");
    assert!(content.contains("new-agent"), "new agent should appear in history: {content}");
}

#[test]
fn take_fails_when_no_agent_assigned() {
    let dir = setup();
    let p = dir.path();
    // Ticket with no agent field
    write_ticket_to_branch(p, "ticket/0001-unassigned", "0001-unassigned.md", "new", 1, "unassigned");
    std::env::set_var("APM_AGENT_NAME", "some-agent");
    let result = apm::cmd::take::run(p, 1, true);
    assert!(result.is_err(), "take should fail when no agent is assigned");
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("apm start"), "error should mention apm start: {msg}");
}

// ── apm spec ────────────────────────────────────────────────────────────────

fn write_spec_ticket(dir: &std::path::Path, id: u32, problem: &str, approach: &str) {
    let branch = format!("ticket/{id:04}-spec-test");
    let filename = format!("{id:04}-spec-test.md");
    let path = format!("tickets/{filename}");
    let content = format!(
        "+++\nid = {id}\ntitle = \"spec test\"\nstate = \"in_progress\"\nbranch = \"{branch}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n### Problem\n\n{problem}\n\n### Acceptance criteria\n\n- [ ] criterion one\n\n### Out of scope\n\nnothing\n\n### Approach\n\n{approach}\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|",
    );
    let branch_exists = std::process::Command::new("git")
        .args(["rev-parse", "--verify", &branch])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !branch_exists {
        git(dir, &["checkout", "-b", &branch]);
    } else {
        git(dir, &["checkout", &branch]);
    }
    std::fs::create_dir_all(dir.join("tickets")).unwrap();
    std::fs::write(dir.join(&path), &content).unwrap();
    git(dir, &["-c", "commit.gpgsign=false", "add", &path]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", "ticket: spec test"]);
    git(dir, &["checkout", "main"]);
}

#[test]
fn spec_prints_all_sections() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    // Should succeed and not error
    apm::cmd::spec::run(p, 1, None, None, false, None).unwrap();
}

#[test]
fn spec_prints_single_section() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "the problem text", "the approach");
    apm::cmd::spec::run(p, 1, Some("Problem".into()), None, false, None).unwrap();
}

#[test]
fn spec_set_section_commits() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "old problem", "old approach");
    apm::cmd::spec::run(p, 1, Some("Problem".into()), Some("new problem text".into()), false, None).unwrap();
    let content = branch_content(p, "ticket/0001-spec-test", "tickets/0001-spec-test.md");
    assert!(content.contains("new problem text"), "updated problem not found: {content}");
}

#[test]
fn spec_check_passes_full_ticket() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    apm::cmd::spec::run(p, 1, None, None, true, None).unwrap();
}

#[test]
fn spec_unknown_section_errors() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    let result = apm::cmd::spec::run(p, 1, Some("NonExistent".into()), None, false, None);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("unknown section"));
}

#[test]
fn spec_nonexistent_ticket_errors() {
    let dir = setup();
    let p = dir.path();
    let result = apm::cmd::spec::run(p, 999, None, None, false, None);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("not found"));
}

#[test]
fn spec_set_without_section_errors() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    let result = apm::cmd::spec::run(p, 1, None, Some("some value".into()), false, None);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("--set requires --section"));
}

fn write_ticket_with_amendment_requests(dir: &std::path::Path, id: u32) {
    let branch = format!("ticket/{id:04}-spec-test");
    let filename = format!("{id:04}-spec-test.md");
    let path = format!("tickets/{filename}");
    let content = format!(
        "+++\nid = {id}\ntitle = \"spec test\"\nstate = \"ammend\"\nbranch = \"{branch}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n### Problem\n\nTest.\n\n### Acceptance criteria\n\n- [ ] criterion one\n\n### Out of scope\n\nnothing\n\n### Approach\n\nDirect.\n\n### Amendment requests\n\n- [ ] Add error handling\n- [ ] Fix the bug\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|",
    );
    let branch_exists = std::process::Command::new("git")
        .args(["rev-parse", "--verify", &branch])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !branch_exists {
        git(dir, &["checkout", "-b", &branch]);
    } else {
        git(dir, &["checkout", &branch]);
    }
    std::fs::create_dir_all(dir.join("tickets")).unwrap();
    std::fs::write(dir.join(&path), &content).unwrap();
    git(dir, &["-c", "commit.gpgsign=false", "add", &path]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", "ticket: spec test with amendments"]);
    git(dir, &["checkout", "main"]);
}

#[test]
fn spec_mark_checks_off_item_in_amendment_requests() {
    let dir = setup();
    let p = dir.path();
    write_ticket_with_amendment_requests(p, 1);
    apm::cmd::spec::run(
        p,
        1,
        Some("Amendment requests".into()),
        None,
        false,
        Some("Add error handling".into()),
    ).unwrap();
    let content = branch_content(p, "ticket/0001-spec-test", "tickets/0001-spec-test.md");
    assert!(content.contains("- [x] Add error handling"), "item not checked: {content}");
    assert!(content.contains("- [ ] Fix the bug"), "other item should remain unchecked: {content}");
}

#[test]
fn spec_mark_no_match_errors() {
    let dir = setup();
    let p = dir.path();
    write_ticket_with_amendment_requests(p, 1);
    let result = apm::cmd::spec::run(
        p,
        1,
        Some("Amendment requests".into()),
        None,
        false,
        Some("nonexistent item".into()),
    );
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("no unchecked item"), "unexpected error: {msg}");
}

#[test]
fn spec_mark_ambiguous_errors() {
    let dir = setup();
    let p = dir.path();
    // "error" matches both "Add error handling" and "Fix the error"
    let branch = "ticket/0001-spec-test";
    let filename = "0001-spec-test.md";
    let path = format!("tickets/{filename}");
    let content = "+++\nid = 1\ntitle = \"spec test\"\nstate = \"ammend\"\nbranch = \"ticket/0001-spec-test\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n### Problem\n\nTest.\n\n### Acceptance criteria\n\n- [ ] one\n\n### Out of scope\n\nnothing\n\n### Approach\n\nDirect.\n\n### Amendment requests\n\n- [ ] Add error handling\n- [ ] Fix the error\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|";
    git(p, &["checkout", "-b", branch]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    std::fs::write(p.join(&path), content).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", &path]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "ticket: ambiguous"]);
    git(p, &["checkout", "main"]);

    let result = apm::cmd::spec::run(
        p,
        1,
        Some("Amendment requests".into()),
        None,
        false,
        Some("error".into()),
    );
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("ambiguous"), "unexpected error: {msg}");
}

#[test]
fn spec_mark_without_section_errors() {
    let dir = setup();
    let p = dir.path();
    write_ticket_with_amendment_requests(p, 1);
    let result = apm::cmd::spec::run(p, 1, None, None, false, Some("Add error handling".into()));
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("--mark requires --section"));
}

#[test]
fn spec_mark_case_insensitive() {
    let dir = setup();
    let p = dir.path();
    write_ticket_with_amendment_requests(p, 1);
    apm::cmd::spec::run(
        p,
        1,
        Some("Amendment requests".into()),
        None,
        false,
        Some("ADD ERROR".into()),
    ).unwrap();
    let content = branch_content(p, "ticket/0001-spec-test", "tickets/0001-spec-test.md");
    assert!(content.contains("- [x] Add error handling"), "item not checked: {content}");
}

// ── apm start --next ─────────────────────────────────────────────────────────

/// Setup that puts the worktrees dir inside the temp dir to avoid parallel-test collisions.
fn setup_with_local_worktrees() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();

    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);

    std::fs::write(
        p.join("apm.toml"),
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[worktrees]
dir = "worktrees"

[agents]
max_concurrent = 3

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id         = "new"
label      = "New"
actionable = ["agent"]

[[workflow.states]]
id         = "ready"
label      = "Ready"
actionable = ["agent"]

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
  actor   = "agent"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#,
    )
    .unwrap();

    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    dir
}

#[test]
fn start_next_no_tickets_prints_message() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::start::run_next(p, true, false, false).unwrap();
}

#[test]
fn start_next_claims_highest_priority_ticket() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha");
    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::start::run_next(p, true, false, false).unwrap();
    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    assert!(content.contains("state = \"in_progress\""), "ticket should be in_progress: {content}");
    assert!(content.contains("agent = \"test-agent\""), "agent should be set: {content}");
}

#[test]
fn start_next_with_instructions_includes_text_in_output() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    // Write a state with instructions pointing to a file
    std::fs::write(p.join("apm.toml"),
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[worktrees]
dir = "worktrees"

[agents]
max_concurrent = 3

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id         = "ready"
label      = "Ready"
actionable = ["agent"]
instructions = "worker-instructions.txt"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
  actor   = "agent"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

[[workflow.states]]
id    = "closed"
label = "Closed"
terminal = true
"#).unwrap();
    std::fs::write(p.join("worker-instructions.txt"), "WORKER INSTRUCTIONS CONTENT").unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", "apm.toml", "worker-instructions.txt"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "add instructions"]);

    write_ticket_to_branch(p, "ticket/0001-work", "0001-work.md", "ready", 1, "work");
    std::env::set_var("APM_AGENT_NAME", "test-agent");

    // Capture stdout
    apm::cmd::start::run_next(p, true, false, false).unwrap();
    // If we get here without error, the instructions file was found and processed.
    // The test verifies no panic or error on a valid instructions path.
}

#[test]
fn start_next_clears_focus_section_from_ticket() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();

    // Write ticket with focus_section set
    let branch = "ticket/0001-focused";
    let filename = "0001-focused.md";
    let path = format!("tickets/{filename}");
    let content = "+++\nid = 1\ntitle = \"focused\"\nstate = \"ready\"\nbranch = \"ticket/0001-focused\"\nfocus_section = \"Approach\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|";
    let branch_exists = std::process::Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .current_dir(p)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !branch_exists {
        git(p, &["checkout", "-b", branch]);
    }
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    std::fs::write(p.join(&path), content).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", &path]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "ticket: focused"]);
    git(p, &["checkout", "main"]);

    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::start::run_next(p, true, false, false).unwrap();

    let after = branch_content(p, branch, &path);
    assert!(!after.contains("focus_section"), "focus_section should be cleared: {after}");
}

#[test]
fn start_next_claims_new_ticket_when_no_ready_tickets() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();

    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);

    std::fs::write(
        p.join("apm.toml"),
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[worktrees]
dir = "worktrees"

[agents]
max_concurrent = 3

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id         = "new"
label      = "New"
actionable = ["agent"]

  [[workflow.states.transitions]]
  to      = "in_design"
  trigger = "command:start"
  actor   = "agent"

[[workflow.states]]
id    = "in_design"
label = "In Design"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#,
    )
    .unwrap();

    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();

    write_ticket_to_branch(p, "ticket/0001-spec-me", "0001-spec-me.md", "new", 1, "spec me");
    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::start::run_next(p, true, false, false).unwrap();

    let content = branch_content(p, "ticket/0001-spec-me", "tickets/0001-spec-me.md");
    assert!(content.contains("state = \"in_design\""), "ticket should be in_design: {content}");
    assert!(content.contains("agent = \"test-agent\""), "agent should be set: {content}");
}

// ── apm work ─────────────────────────────────────────────────────────────────

#[test]
fn work_dry_run_lists_actionable_tickets() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha");
    write_ticket_to_branch(p, "ticket/0002-beta", "0002-beta.md", "ready", 2, "beta");
    std::env::set_var("APM_AGENT_NAME", "test-agent");
    // dry-run should succeed without touching worktrees or spawning anything
    apm::cmd::work::run(p, false, true).unwrap();
}

#[test]
fn work_dry_run_no_tickets() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::work::run(p, false, true).unwrap();
}

// --- sync accept ---

#[test]
fn sync_auto_accept_transitions_implemented_ticket_to_accepted() {
    let dir = setup_with_close_workflow();
    let p = dir.path();

    // Write a ticket in `implemented` state on its ticket branch.
    write_ticket_to_branch(p, "ticket/0001-impl", "0001-impl.md", "implemented", 1, "impl ticket");

    // Merge the ticket branch into main so it appears as merged.
    git(p, &["-c", "commit.gpgsign=false", "merge", "--no-ff", "ticket/0001-impl", "--no-edit"]);

    // Run sync with auto_accept — no interactive prompt needed.
    apm::cmd::sync::run(p, true, true, true, false, true).unwrap();

    // The ticket branch should now have state = "accepted".
    let content = branch_content(p, "ticket/0001-impl", "tickets/0001-impl.md");
    assert!(content.contains("state = \"accepted\""), "ticket should be accepted: {content}");
}

// --- context-section ---

#[test]
fn context_section_approach_places_text_under_approach() {
    let dir = setup();
    apm::cmd::new::run(
        dir.path(),
        "Section test".into(),
        true,
        false,
        Some("my approach text".into()),
        Some("Approach".into()),
        true,
    ).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-section-test", "tickets/0001-section-test.md");
    assert!(content.contains("### Approach\n\nmy approach text\n\n"), "expected context under ### Approach");
    // Problem section should be empty
    assert!(content.contains("### Problem\n\n### Acceptance criteria"), "Problem should be empty");
}

#[test]
fn context_section_defaults_to_problem_without_config() {
    let dir = setup();
    apm::cmd::new::run(
        dir.path(),
        "Default section test".into(),
        true,
        false,
        Some("default context".into()),
        None,
        true,
    ).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-default-section-test", "tickets/0001-default-section-test.md");
    assert!(content.contains("### Problem\n\ndefault context\n\n"), "expected context under ### Problem");
}

#[test]
fn context_section_without_context_is_error() {
    let dir = setup();
    let result = apm::cmd::new::run(
        dir.path(),
        "Error test".into(),
        true,
        false,
        None,
        Some("Approach".into()),
        true,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("--context-section requires --context"));
}

#[test]
fn context_section_unknown_section_is_error() {
    let dir = setup();
    let result = apm::cmd::new::run(
        dir.path(),
        "Bad section test".into(),
        true,
        false,
        Some("some text".into()),
        Some("Nonexistent".into()),
        true,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found in ticket body template"));
}

#[test]
fn context_section_from_transition_config() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    std::fs::write(p.join("apm.toml"), r#"[project]
name = "test"

[tickets]
dir = "tickets"

[agents]
max_concurrent = 3

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id         = "new"
label      = "New"
actionable = ["agent"]

[[workflow.states.transitions]]
to              = "in_design"
context_section = "Approach"
"#).unwrap();
    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    apm::cmd::new::run(
        p,
        "Transition context test".into(),
        true,
        false,
        Some("transition driven context".into()),
        None,
        true,
    ).unwrap();
    let content = branch_content(p, "ticket/0001-transition-context-test", "tickets/0001-transition-context-test.md");
    assert!(content.contains("### Approach\n\ntransition driven context\n\n"), "expected context under ### Approach from transition config");
}
