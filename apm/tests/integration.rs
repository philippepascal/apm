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

// --- init ---

#[test]
fn init_creates_expected_files() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    apm::cmd::init::run(p, true).unwrap();
    assert!(p.join("tickets").is_dir());
    assert!(p.join("apm.toml").exists());
    assert!(p.join(".gitignore").exists());
    assert!(p.join(".git/hooks/pre-push").exists());
    assert!(p.join(".git/hooks/post-merge").exists());
}

#[test]
fn init_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    apm::cmd::init::run(p, true).unwrap();
    let toml_before = std::fs::read_to_string(p.join("apm.toml")).unwrap();
    apm::cmd::init::run(p, true).unwrap();
    let toml_after = std::fs::read_to_string(p.join("apm.toml")).unwrap();
    assert_eq!(toml_before, toml_after);
}

#[test]
fn init_generated_config_has_all_workflow_states() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    apm::cmd::init::run(p, true).unwrap();

    let toml = std::fs::read_to_string(p.join("apm.toml")).unwrap();
    for state in &["new", "question", "specd", "ammend", "ready", "in_progress", "implemented", "accepted", "closed"] {
        assert!(toml.contains(&format!("\"{state}\"")), "missing state: {state}");
    }
    assert!(toml.contains("terminal = true"), "closed must be terminal");
    // Must parse without error.
    apm_core::config::Config::load(p).unwrap();
}

#[test]
fn list_excludes_terminal_tickets_by_default() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Open ticket".into()).unwrap();
    apm::cmd::new::run(dir.path(), "Closed ticket".into()).unwrap();
    apm::cmd::state::run(dir.path(), 2, "closed".into()).unwrap();

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
    apm::cmd::new::run(dir.path(), "My first ticket".into()).unwrap();
    // File lives on the ticket branch, not in the working tree.
    let content = branch_content(dir.path(), "ticket/0001-my-first-ticket", "tickets/0001-my-first-ticket.md");
    assert!(!content.is_empty());
}

#[test]
fn new_ticket_has_correct_frontmatter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Hello World".into()).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-hello-world", "tickets/0001-hello-world.md");
    assert!(content.contains("id = 1"));
    assert!(content.contains("title = \"Hello World\""));
    assert!(content.contains("state = \"new\""));
    assert!(content.contains("branch = \"ticket/0001-hello-world\""));
}

#[test]
fn new_increments_ids() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "First".into()).unwrap();
    apm::cmd::new::run(dir.path(), "Second".into()).unwrap();
    let c1 = branch_content(dir.path(), "ticket/0001-first", "tickets/0001-first.md");
    let c2 = branch_content(dir.path(), "ticket/0002-second", "tickets/0002-second.md");
    assert!(c1.contains("id = 1"));
    assert!(c2.contains("id = 2"));
}

// --- list ---

#[test]
fn list_shows_all_tickets() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Alpha".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-alpha", "tickets/0001-alpha.md");
    apm::cmd::new::run(dir.path(), "Beta".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0002-beta", "tickets/0002-beta.md");
    apm::cmd::list::run(dir.path(), None, false, false, None, None).unwrap();
}

#[test]
fn list_state_filter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Alpha".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-alpha", "tickets/0001-alpha.md");
    apm::cmd::new::run(dir.path(), "Beta".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0002-beta", "tickets/0002-beta.md");
    apm::cmd::state::run(dir.path(), 1, "specd".into()).unwrap();
    // Sync the updated ticket from its branch so apm list can see the new state.
    sync_from_branch(dir.path(), "ticket/0001-alpha", "tickets/0001-alpha.md");
    apm::cmd::list::run(dir.path(), Some("specd".into()), false, false, None, None).unwrap();
}

// --- show ---

#[test]
fn show_existing_ticket() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Show me".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-show-me", "tickets/0001-show-me.md");
    apm::cmd::show::run(dir.path(), 1).unwrap();
}

#[test]
fn show_missing_ticket_errors() {
    let dir = setup();
    assert!(apm::cmd::show::run(dir.path(), 99).is_err());
}

// --- state ---

#[test]
fn state_transition_updates_file() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Transition test".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-transition-test", "tickets/0001-transition-test.md");
    apm::cmd::state::run(dir.path(), 1, "specd".into()).unwrap();
    // Read the updated state from the ticket branch (not the working tree).
    let content = branch_content(dir.path(), "ticket/0001-transition-test", "tickets/0001-transition-test.md");
    assert!(content.contains("state = \"specd\""));
}

#[test]
fn state_transition_appends_history_row() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "History test".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-history-test", "tickets/0001-history-test.md");
    apm::cmd::state::run(dir.path(), 1, "specd".into()).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-history-test", "tickets/0001-history-test.md");
    assert!(content.contains("| new | specd |"));
}

#[test]
fn state_ammend_inserts_amendment_section() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Ammend test".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-ammend-test", "tickets/0001-ammend-test.md");
    apm::cmd::state::run(dir.path(), 1, "ammend".into()).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-ammend-test", "tickets/0001-ammend-test.md");
    assert!(content.contains("### Amendment requests"));
}

// --- set ---

#[test]
fn set_priority_updates_frontmatter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Set test".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-set-test", "tickets/0001-set-test.md");
    apm::cmd::set::run(dir.path(), 1, "priority".into(), "7".into()).unwrap();
    let content = branch_content(dir.path(), "ticket/0001-set-test", "tickets/0001-set-test.md");
    assert!(content.contains("priority = 7"));
}

// --- next ---

#[test]
fn next_returns_highest_priority() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Low priority".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0001-low-priority", "tickets/0001-low-priority.md");
    apm::cmd::new::run(dir.path(), "High priority".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0002-high-priority", "tickets/0002-high-priority.md");
    apm::cmd::set::run(dir.path(), 2, "priority".into(), "10".into()).unwrap();
    sync_from_branch(dir.path(), "ticket/0002-high-priority", "tickets/0002-high-priority.md");
    apm::cmd::next::run(dir.path(), false).unwrap();
}

#[test]
fn next_json_is_valid() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Json test".into()).unwrap();
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
    apm::cmd::new::run(dir.path(), "Branch test".into()).unwrap();
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
    apm::cmd::new::run(dir.path(), "Frontmatter branch".into()).unwrap();
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
    apm::cmd::init::run(p, true).unwrap();

    let toml = std::fs::read_to_string(p.join("apm.toml")).unwrap();
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
