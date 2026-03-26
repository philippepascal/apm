use tempfile::TempDir;

fn setup() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .status()
        .unwrap();
    // minimal apm.toml so Config::load succeeds
    std::fs::write(
        dir.path().join("apm.toml"),
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[agents]
max_concurrent = 3
actionable_states = ["new", "ammend", "ready"]

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0
"#,
    )
    .unwrap();
    dir
}

// --- init ---

#[test]
fn init_creates_expected_files() {
    let dir = tempfile::tempdir().unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .status()
        .unwrap();
    apm::cmd::init::run(dir.path()).unwrap();
    assert!(dir.path().join("tickets").is_dir());
    assert!(dir.path().join("apm.toml").exists());
    assert!(dir.path().join(".gitignore").exists());
    assert!(dir.path().join(".git/hooks/pre-push").exists());
    assert!(dir.path().join(".git/hooks/post-merge").exists());
}

#[test]
fn init_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .status()
        .unwrap();
    apm::cmd::init::run(dir.path()).unwrap();
    // second run should not error or overwrite existing apm.toml
    let toml_before = std::fs::read_to_string(dir.path().join("apm.toml")).unwrap();
    apm::cmd::init::run(dir.path()).unwrap();
    let toml_after = std::fs::read_to_string(dir.path().join("apm.toml")).unwrap();
    assert_eq!(toml_before, toml_after);
}

// --- new ---

#[test]
fn new_creates_ticket_file() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "My first ticket".into()).unwrap();
    assert!(dir.path().join("tickets/0001-my-first-ticket.md").exists());
}

#[test]
fn new_ticket_has_correct_frontmatter() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "Hello World".into()).unwrap();
    let content = std::fs::read_to_string(dir.path().join("tickets/0001-hello-world.md")).unwrap();
    assert!(content.contains("id = 1"));
    assert!(content.contains("title = \"Hello World\""));
    assert!(content.contains("state = \"new\""));
}

#[test]
fn new_increments_ids() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "First".into()).unwrap();
    apm::cmd::new::run(dir.path(), "Second".into()).unwrap();
    assert!(dir.path().join("tickets/0001-first.md").exists());
    assert!(dir.path().join("tickets/0002-second.md").exists());
}

// --- list ---

#[test]
fn list_shows_all_tickets() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "Alpha".into()).unwrap();
    apm::cmd::new::run(dir.path(), "Beta".into()).unwrap();
    // just ensure it doesn't error; output goes to stdout
    apm::cmd::list::run(dir.path(), None, false).unwrap();
}

#[test]
fn list_state_filter() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "Alpha".into()).unwrap();
    apm::cmd::new::run(dir.path(), "Beta".into()).unwrap();
    apm::cmd::state::run(dir.path(), 1, "specd".into()).unwrap();
    // only specd — should not error
    apm::cmd::list::run(dir.path(), Some("specd".into()), false).unwrap();
}

// --- show ---

#[test]
fn show_existing_ticket() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "Show me".into()).unwrap();
    apm::cmd::show::run(dir.path(), 1).unwrap();
}

#[test]
fn show_missing_ticket_errors() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    assert!(apm::cmd::show::run(dir.path(), 99).is_err());
}

// --- state ---

#[test]
fn state_transition_updates_file() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "Transition test".into()).unwrap();
    apm::cmd::state::run(dir.path(), 1, "specd".into()).unwrap();
    let content = std::fs::read_to_string(dir.path().join("tickets/0001-transition-test.md")).unwrap();
    assert!(content.contains("state = \"specd\""));
}

#[test]
fn state_transition_appends_history_row() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "History test".into()).unwrap();
    apm::cmd::state::run(dir.path(), 1, "specd".into()).unwrap();
    let content = std::fs::read_to_string(dir.path().join("tickets/0001-history-test.md")).unwrap();
    assert!(content.contains("new → specd"));
}

#[test]
fn state_ammend_inserts_amendment_section() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "Ammend test".into()).unwrap();
    apm::cmd::state::run(dir.path(), 1, "ammend".into()).unwrap();
    let content = std::fs::read_to_string(dir.path().join("tickets/0001-ammend-test.md")).unwrap();
    assert!(content.contains("### Amendment requests"));
}

// --- set ---

#[test]
fn set_priority_updates_frontmatter() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "Set test".into()).unwrap();
    apm::cmd::set::run(dir.path(), 1, "priority".into(), "7".into()).unwrap();
    let content = std::fs::read_to_string(dir.path().join("tickets/0001-set-test.md")).unwrap();
    assert!(content.contains("priority = 7"));
}

// --- next ---

#[test]
fn next_returns_highest_priority() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "Low priority".into()).unwrap();
    apm::cmd::new::run(dir.path(), "High priority".into()).unwrap();
    apm::cmd::set::run(dir.path(), 2, "priority".into(), "10".into()).unwrap();
    // should not error; highest priority ticket is #2
    apm::cmd::next::run(dir.path(), false).unwrap();
}

#[test]
fn next_json_is_valid() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    apm::cmd::new::run(dir.path(), "Json test".into()).unwrap();
    // just ensure it doesn't error
    apm::cmd::next::run(dir.path(), true).unwrap();
}

#[test]
fn next_null_when_no_actionable() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tickets")).unwrap();
    // no tickets at all
    apm::cmd::next::run(dir.path(), true).unwrap();
}
