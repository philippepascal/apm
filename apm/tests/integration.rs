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

/// Find the ticket branch whose slug matches (after any hex ID prefix).
fn find_ticket_branch(dir: &std::path::Path, slug: &str) -> String {
    let pattern = format!("ticket/*-{slug}");
    let out = std::process::Command::new("git")
        .args(["branch", "--list", &pattern])
        .current_dir(dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    stdout
        .lines()
        .find(|l| !l.trim().is_empty())
        // strip "* " (current branch) and "+ " (branch in another worktree)
        .map(|l| l.trim().trim_start_matches(['*', '+']).trim_start_matches(' ').to_string())
        .unwrap_or_else(|| panic!("no branch found for slug: {slug}"))
}

/// Extract the ticket ID from a branch found by slug.
fn find_ticket_id(dir: &std::path::Path, slug: &str) -> String {
    let branch = find_ticket_branch(dir, slug);
    branch
        .strip_prefix("ticket/")
        .and_then(|s| s.split('-').next())
        .unwrap_or_else(|| panic!("bad branch: {branch}"))
        .to_string()
}

/// Derive the tickets/ relative path from a branch name.
fn ticket_rel_path(branch: &str) -> String {
    let suffix = branch.strip_prefix("ticket/").expect("not a ticket branch");
    format!("tickets/{suffix}.md")
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
    apm::cmd::init::run(p, true, false, false).unwrap();
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
    apm::cmd::init::run(p, true, false, false).unwrap();
    let toml_before = std::fs::read_to_string(p.join(".apm/config.toml")).unwrap();
    apm::cmd::init::run(p, true, false, false).unwrap();
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
    apm::cmd::init::run(p, true, false, false).unwrap();

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
    let closed_id = find_ticket_id(dir.path(), "closed-ticket");
    apm::cmd::state::run(dir.path(), &closed_id, "closed".into(), false, false).unwrap();

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
    assert_eq!(visible[0].frontmatter.title, "Open ticket");

    let all: Vec<_> = tickets.iter().collect();
    assert_eq!(all.len(), 2, "--all should include the closed ticket");
}

// --- new ---

#[test]
fn new_creates_ticket_file() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "My first ticket".into(), true, false, None, None, true).unwrap();
    // File lives on the ticket branch, not in the working tree.
    let branch = find_ticket_branch(dir.path(), "my-first-ticket");
    let rel_path = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel_path);
    assert!(!content.is_empty());
}

#[test]
fn new_ticket_has_correct_frontmatter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Hello World".into(), true, false, None, None, true).unwrap();
    let branch = find_ticket_branch(dir.path(), "hello-world");
    let rel_path = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel_path);
    assert!(content.contains("title = \"Hello World\""));
    assert!(content.contains("state = \"new\""));
    assert!(content.contains(&format!("branch = \"{branch}\"")));
}

#[test]
fn new_increments_ids() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "First".into(), true, false, None, None, true).unwrap();
    apm::cmd::new::run(dir.path(), "Second".into(), true, false, None, None, true).unwrap();
    let id1 = find_ticket_id(dir.path(), "first");
    let id2 = find_ticket_id(dir.path(), "second");
    assert_ne!(id1, id2, "ticket IDs must be unique");
    assert_eq!(id1.len(), 8, "ID must be 8 hex chars");
    assert_eq!(id2.len(), 8, "ID must be 8 hex chars");
    assert!(id1.chars().all(|c| c.is_ascii_hexdigit()), "ID must be hex: {id1}");
    assert!(id2.chars().all(|c| c.is_ascii_hexdigit()), "ID must be hex: {id2}");
}

// --- list ---

#[test]
fn list_shows_all_tickets() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Alpha".into(), true, false, None, None, true).unwrap();
    let b1 = find_ticket_branch(dir.path(), "alpha");
    sync_from_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::new::run(dir.path(), "Beta".into(), true, false, None, None, true).unwrap();
    let b2 = find_ticket_branch(dir.path(), "beta");
    sync_from_branch(dir.path(), &b2, &ticket_rel_path(&b2));
    apm::cmd::list::run(dir.path(), None, false, false, None, None).unwrap();
}

#[test]
fn list_state_filter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Alpha".into(), true, false, None, None, true).unwrap();
    let b1 = find_ticket_branch(dir.path(), "alpha");
    let alpha_id = find_ticket_id(dir.path(), "alpha");
    sync_from_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::new::run(dir.path(), "Beta".into(), true, false, None, None, true).unwrap();
    let b2 = find_ticket_branch(dir.path(), "beta");
    sync_from_branch(dir.path(), &b2, &ticket_rel_path(&b2));
    write_valid_spec_to_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::state::run(dir.path(), &alpha_id, "specd".into(), false, false).unwrap();
    // Sync the updated ticket from its branch so apm list can see the new state.
    sync_from_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::list::run(dir.path(), Some("specd".into()), false, false, None, None).unwrap();
}

// --- show ---

#[test]
fn show_existing_ticket() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Show me".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(dir.path(), "show-me");
    apm::cmd::show::run(dir.path(), &id, false).unwrap();
}

#[test]
fn show_missing_ticket_errors() {
    let dir = setup();
    assert!(apm::cmd::show::run(dir.path(), "99", false).is_err());
}

// --- state ---

#[test]
fn state_transition_updates_file() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Transition test".into(), true, false, None, None, true).unwrap();
    let branch = find_ticket_branch(dir.path(), "transition-test");
    let id = find_ticket_id(dir.path(), "transition-test");
    let rel = ticket_rel_path(&branch);
    write_valid_spec_to_branch(dir.path(), &branch, &rel);
    apm::cmd::state::run(dir.path(), &id, "specd".into(), false, false).unwrap();
    // Read the updated state from the ticket branch (not the working tree).
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("state = \"specd\""));
}

#[test]
fn state_transition_appends_history_row() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "History test".into(), true, false, None, None, true).unwrap();
    let branch = find_ticket_branch(dir.path(), "history-test");
    let id = find_ticket_id(dir.path(), "history-test");
    let rel = ticket_rel_path(&branch);
    write_valid_spec_to_branch(dir.path(), &branch, &rel);
    apm::cmd::state::run(dir.path(), &id, "specd".into(), false, false).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("| new | specd |"));
}

#[test]
fn state_ammend_inserts_amendment_section() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Ammend test".into(), true, false, None, None, true).unwrap();
    let branch = find_ticket_branch(dir.path(), "ammend-test");
    let id = find_ticket_id(dir.path(), "ammend-test");
    let rel = ticket_rel_path(&branch);
    apm::cmd::state::run(dir.path(), &id, "ammend".into(), false, false).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("### Amendment requests"));
}

// --- set ---

#[test]
fn set_priority_updates_frontmatter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Set test".into(), true, false, None, None, true).unwrap();
    let branch = find_ticket_branch(dir.path(), "set-test");
    let id = find_ticket_id(dir.path(), "set-test");
    let rel = ticket_rel_path(&branch);
    apm::cmd::set::run(dir.path(), &id, "priority".into(), "7".into()).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("priority = 7"));
}

// --- next ---

#[test]
fn next_returns_highest_priority() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Low priority".into(), true, false, None, None, true).unwrap();
    let b1 = find_ticket_branch(dir.path(), "low-priority");
    sync_from_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::new::run(dir.path(), "High priority".into(), true, false, None, None, true).unwrap();
    let b2 = find_ticket_branch(dir.path(), "high-priority");
    let high_id = find_ticket_id(dir.path(), "high-priority");
    apm::cmd::set::run(dir.path(), &high_id, "priority".into(), "10".into()).unwrap();
    sync_from_branch(dir.path(), &b2, &ticket_rel_path(&b2));
    apm::cmd::next::run(dir.path(), false).unwrap();
}

#[test]
fn next_json_is_valid() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Json test".into(), true, false, None, None, true).unwrap();
    let b = find_ticket_branch(dir.path(), "json-test");
    sync_from_branch(dir.path(), &b, &ticket_rel_path(&b));
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
    let branch = find_ticket_branch(dir.path(), "branch-test");
    assert!(branch.starts_with("ticket/"), "expected ticket/ branch, got: {branch}");
    assert!(branch.ends_with("-branch-test"), "expected slug in branch: {branch}");
}

#[test]
fn new_ticket_sets_branch_in_frontmatter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Frontmatter branch".into(), true, false, None, None, true).unwrap();
    let branch = find_ticket_branch(dir.path(), "frontmatter-branch");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains(&format!("branch = \"{branch}\"")));
}

#[test]
fn init_config_has_default_branch_and_parses() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q", "-b", "trunk"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    apm::cmd::init::run(p, true, false, false).unwrap();

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
fn sync_closes_multiple_accepted_tickets() {
    let dir = setup_with_close_workflow();
    let p = dir.path();

    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "accepted", 1, "alpha");
    write_ticket_to_branch(p, "ticket/0002-beta", "0002-beta.md", "accepted", 2, "beta");

    apm::cmd::sync::run(p, true, true, true, true, false).unwrap();

    let alpha = branch_content(p, "main", "tickets/0001-alpha.md");
    let beta = branch_content(p, "main", "tickets/0002-beta.md");
    assert!(alpha.contains("state = \"closed\""), "alpha should be closed on main: {alpha}");
    assert!(beta.contains("state = \"closed\""), "beta should be closed on main: {beta}");
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
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_with_agent(p, "ticket/0001-ammend-me", "0001-ammend-me.md", "ammend", 1, "ammend me", "old-agent");
    std::env::set_var("APM_AGENT_NAME", "new-agent");
    apm::cmd::take::run(p, "1", true).unwrap();
    let content = branch_content(p, "ticket/0001-ammend-me", "tickets/0001-ammend-me.md");
    assert!(content.contains("agent = \"new-agent\""), "agent should be updated: {content}");
}

#[test]
fn take_succeeds_on_blocked_state() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_with_agent(p, "ticket/0001-blocked", "0001-blocked.md", "blocked", 1, "blocked", "old-agent");
    std::env::set_var("APM_AGENT_NAME", "new-agent");
    apm::cmd::take::run(p, "1", true).unwrap();
    let content = branch_content(p, "ticket/0001-blocked", "tickets/0001-blocked.md");
    assert!(content.contains("agent = \"new-agent\""), "agent should be updated: {content}");
}

#[test]
fn take_appends_handoff_history() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_with_agent(p, "ticket/0001-handoff", "0001-handoff.md", "in_progress", 1, "handoff", "old-agent");
    std::env::set_var("APM_AGENT_NAME", "new-agent");
    apm::cmd::take::run(p, "1", true).unwrap();
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
    let result = apm::cmd::take::run(p, "1", true);
    assert!(result.is_err(), "take should fail when no agent is assigned");
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("apm start"), "error should mention apm start: {msg}");
}

#[test]
fn take_without_apm_agent_name_falls_back_to_apm() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_with_agent(p, "ticket/0001-fallback", "0001-fallback.md", "in_design", 1, "fallback", "old-agent");
    std::env::remove_var("APM_AGENT_NAME");
    std::env::remove_var("USER");
    std::env::remove_var("USERNAME");
    apm::cmd::take::run(p, "1", true).unwrap();
    let content = branch_content(p, "ticket/0001-fallback", "tickets/0001-fallback.md");
    assert!(content.contains("agent = \"apm\""), "agent should fall back to 'apm': {content}");
}

#[test]
fn take_without_apm_agent_name_uses_user_env() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_with_agent(p, "ticket/0001-user-env", "0001-user-env.md", "in_design", 1, "user env", "old-agent");
    std::env::remove_var("APM_AGENT_NAME");
    std::env::set_var("USER", "alice");
    apm::cmd::take::run(p, "1", true).unwrap();
    let content = branch_content(p, "ticket/0001-user-env", "tickets/0001-user-env.md");
    assert!(content.contains("agent = \"alice\""), "agent should be resolved from USER: {content}");
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
    apm::cmd::spec::run(p, "1", None, None, false, None).unwrap();
}

#[test]
fn spec_prints_single_section() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "the problem text", "the approach");
    apm::cmd::spec::run(p, "1", Some("Problem".into()), None, false, None).unwrap();
}

#[test]
fn spec_set_section_commits() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "old problem", "old approach");
    apm::cmd::spec::run(p, "1", Some("Problem".into()), Some("new problem text".into()), false, None).unwrap();
    let content = branch_content(p, "ticket/0001-spec-test", "tickets/0001-spec-test.md");
    assert!(content.contains("new problem text"), "updated problem not found: {content}");
}

#[test]
fn spec_check_passes_full_ticket() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    apm::cmd::spec::run(p, "1", None, None, true, None).unwrap();
}

#[test]
fn spec_unknown_section_errors() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    let result = apm::cmd::spec::run(p, "1", Some("NonExistent".into()), None, false, None);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("unknown section"));
}

#[test]
fn spec_nonexistent_ticket_errors() {
    let dir = setup();
    let p = dir.path();
    let result = apm::cmd::spec::run(p, "999", None, None, false, None);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("no ticket matches"));
}

#[test]
fn spec_set_without_section_errors() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    let result = apm::cmd::spec::run(p, "1", None, Some("some value".into()), false, None);
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
        "1",
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
        "1",
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
        "1",
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
    let result = apm::cmd::spec::run(p, "1", None, None, false, Some("Add error handling".into()));
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
        "1",
        Some("Amendment requests".into()),
        None,
        false,
        Some("ADD ERROR".into()),
    ).unwrap();
    let content = branch_content(p, "ticket/0001-spec-test", "tickets/0001-spec-test.md");
    assert!(content.contains("- [x] Add error handling"), "item not checked: {content}");
}

// ── apm close ────────────────────────────────────────────────────────────────

#[test]
fn close_transitions_from_any_state() {
    let dir = setup();
    let p = dir.path();
    apm::cmd::new::run(p, "Close me".into(), true, false, None, None, true).unwrap();
    let branch = find_ticket_branch(p, "close-me");
    let id = find_ticket_id(p, "close-me");
    let rel = ticket_rel_path(&branch);
    // Ticket is in "new" state — no transition to "closed" is defined.
    apm::cmd::close::run(p, &id, None).unwrap();
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("state = \"closed\""), "state not updated: {content}");
    assert!(content.contains("| new | closed |"), "history row missing: {content}");
}

#[test]
fn close_with_reason_appends_to_history() {
    let dir = setup();
    let p = dir.path();
    apm::cmd::new::run(p, "Close reason".into(), true, false, None, None, true).unwrap();
    let branch = find_ticket_branch(p, "close-reason");
    let id = find_ticket_id(p, "close-reason");
    let rel = ticket_rel_path(&branch);
    apm::cmd::close::run(p, &id, Some("superseded by #42".into())).unwrap();
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("state = \"closed\""), "state not updated: {content}");
    assert!(content.contains("superseded by #42"), "reason missing: {content}");
}

#[test]
fn close_already_closed_is_error() {
    let dir = setup();
    let p = dir.path();
    apm::cmd::new::run(p, "Already closed".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(p, "already-closed");
    apm::cmd::close::run(p, &id, None).unwrap();
    let result = apm::cmd::close::run(p, &id, None);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("already closed"));
}

#[test]
fn close_nonexistent_ticket_is_error() {
    let dir = setup();
    let p = dir.path();
    let result = apm::cmd::close::run(p, "999", None);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("no ticket matches"));
}

#[test]
fn validate_does_not_flag_closed_state() {
    let dir = setup();
    let p = dir.path();
    apm::cmd::new::run(p, "Validate closed".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(p, "validate-closed");
    apm::cmd::close::run(p, &id, None).unwrap();
    // apm validate should not flag the closed ticket as having an unknown state.
    // The test config is minimal and may produce config warnings (e.g. missing transitions),
    // but there must be zero ticket-level errors.
    let result = apm::cmd::validate::run(p, false, false, false);
    if let Err(e) = &result {
        let msg = e.to_string();
        assert!(
            msg.contains("0 ticket errors"),
            "validate flagged a ticket-level error (possibly unknown state): {msg}"
        );
    }
}

#[test]
fn state_to_closed_bypasses_transition_rules() {
    let dir = setup();
    let p = dir.path();
    apm::cmd::new::run(p, "State closed".into(), true, false, None, None, true).unwrap();
    let branch = find_ticket_branch(p, "state-closed");
    let id = find_ticket_id(p, "state-closed");
    let rel = ticket_rel_path(&branch);
    // "new" state has no outgoing transitions to "closed" in the test config,
    // but "closed" is a mandatory terminal state so it should still work.
    apm::cmd::state::run(p, &id, "closed".into(), false, false).unwrap();
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("state = \"closed\""), "state not updated: {content}");
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

// ── apm start --spawn ────────────────────────────────────────────────────────

/// Write a minimal fake `claude` executable to `bin_dir` and prepend it to PATH.
/// Returns the old PATH so the caller can restore it.
fn fake_claude_in_path(bin_dir: &std::path::Path) -> String {
    let old_path = std::env::var("PATH").unwrap_or_default();
    let script = "#!/bin/sh\nexit 0\n";
    let exe = bin_dir.join("claude");
    std::fs::write(&exe, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let new_path = format!("{}:{old_path}", bin_dir.display());
    std::env::set_var("PATH", &new_path);
    old_path
}

#[test]
fn start_spawn_sets_agent_to_worker_pid() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha");

    let bin_dir = tempfile::tempdir().unwrap();
    let old_path = fake_claude_in_path(bin_dir.path());

    std::env::set_var("APM_AGENT_NAME", "delegator-agent");
    apm::cmd::start::run(p, "1", true, true, false, "delegator-agent").unwrap();

    std::env::set_var("PATH", &old_path);

    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    // agent must be a decimal PID, not the delegator name
    assert!(!content.contains("agent = \"delegator-agent\""), "agent should not be delegator: {content}");
    let agent_val = content.lines()
        .find(|l| l.starts_with("agent = "))
        .and_then(|l| l.strip_prefix("agent = \""))
        .and_then(|l| l.strip_suffix('"'))
        .unwrap_or_else(|| panic!("agent field not found in: {content}"));
    assert!(agent_val.parse::<u32>().is_ok(), "agent should be a PID number, got: {agent_val}");
}

#[test]
fn start_non_spawn_keeps_agent_name() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha");

    std::env::set_var("APM_AGENT_NAME", "delegator-agent");
    apm::cmd::start::run(p, "1", true, false, false, "delegator-agent").unwrap();

    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    assert!(content.contains("agent = \"delegator-agent\""), "non-spawn should keep APM_AGENT_NAME: {content}");
}

#[test]
fn start_next_spawn_sets_agent_to_worker_pid() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha");

    let bin_dir = tempfile::tempdir().unwrap();
    let old_path = fake_claude_in_path(bin_dir.path());

    std::env::set_var("APM_AGENT_NAME", "delegator-agent");
    apm::cmd::start::run_next(p, true, true, false).unwrap();

    std::env::set_var("PATH", &old_path);

    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    assert!(!content.contains("agent = \"delegator-agent\""), "agent should not be delegator after spawn: {content}");
    let agent_val = content.lines()
        .find(|l| l.starts_with("agent = "))
        .and_then(|l| l.strip_prefix("agent = \""))
        .and_then(|l| l.strip_suffix('"'))
        .unwrap_or_else(|| panic!("agent field not found in: {content}"));
    assert!(agent_val.parse::<u32>().is_ok(), "agent should be a PID number, got: {agent_val}");
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
    let branch = find_ticket_branch(dir.path(), "section-test");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel);
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
    let branch = find_ticket_branch(dir.path(), "default-section-test");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel);
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
    let branch = find_ticket_branch(p, "transition-context-test");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("### Approach\n\ntransition driven context\n\n"), "expected context under ### Approach from transition config");
}

#[test]
fn new_body_scaffold_from_ticket_sections_config() {
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

[[ticket.sections]]
name        = "Summary"
type        = "free"
placeholder = "What does this do?"

[[ticket.sections]]
name     = "Tasks"
type     = "tasks"
required = true

[[ticket.sections]]
name = "Notes"
type = "free"
"#).unwrap();
    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();

    apm::cmd::new::run(p, "Scaffold test".into(), true, false, None, None, true).unwrap();

    let scaffold_branch = find_ticket_branch(p, "scaffold-test");
    let scaffold_rel = ticket_rel_path(&scaffold_branch);
    let content = branch_content(p, &scaffold_branch, &scaffold_rel);
    assert!(content.contains("### Summary\n\nWhat does this do?\n\n"), "placeholder should appear");
    assert!(content.contains("### Tasks\n\n\n\n"), "empty section should appear");
    assert!(content.contains("### Notes\n\n\n\n"), "Notes section should appear");
    assert!(!content.contains("### Problem\n"), "hardcoded Problem should not appear");
    assert!(!content.contains("### Acceptance criteria\n"), "hardcoded AC should not appear");
}

// --- validate ---

#[test]
fn validate_config_missing_instructions_and_bad_context_section() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();

    // Init a minimal git repo.
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);

    // Config with:
    //   - state `new` with instructions pointing to a non-existent file
    //   - transition with context_section that doesn't exist in ticket.sections
    std::fs::write(
        p.join("apm.toml"),
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[[ticket.sections]]
name = "Problem"
type = "free"

[[workflow.states]]
id           = "new"
label        = "New"
instructions = "missing-file.md"

[[workflow.states.transitions]]
to              = "closed"
context_section = "NonExistentSection"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#,
    )
    .unwrap();

    let config = apm_core::config::Config::load(p).unwrap();
    let errors = apm::cmd::validate::validate_config(&config, p);

    assert_eq!(errors.len(), 2, "expected exactly 2 errors, got: {errors:?}");

    let has_missing_file = errors.iter().any(|e| {
        e.contains("state.new.instructions") && e.contains("file not found")
    });
    assert!(has_missing_file, "expected missing instructions error in {errors:?}");

    let has_bad_section = errors.iter().any(|e| {
        e.contains("context_section") && e.contains("NonExistentSection")
    });
    assert!(has_bad_section, "expected context_section mismatch error in {errors:?}");
}

// --- review ---

#[test]
fn review_ammend_normalises_plain_bullets_to_checkboxes() {
    let dir = setup();
    let p = dir.path();

    apm::cmd::new::run(p, "Review checkbox test".into(), true, false, None, None, true).unwrap();

    let branch = find_ticket_branch(p, "review-checkbox-test");
    let ticket_path = ticket_rel_path(&branch);
    let id = find_ticket_id(p, "review-checkbox-test");

    // Write a spec with a ### Amendment requests section containing plain bullets.
    let existing = branch_content(p, &branch, &ticket_path);
    let fm_end = existing.find("\n+++\n").expect("frontmatter close not found") + 5;
    let frontmatter = &existing[..fm_end];
    let body = "\n## Spec\n\n### Problem\n\nTest.\n\n### Acceptance criteria\n\n- [ ] AC one\n\n### Out of scope\n\nNothing.\n\n### Approach\n\nDirect.\n\n### Amendment requests\n\n- plain item\n- [ ] already a checkbox\n- [x] already checked\n\n## History\n\n| When | From | To | By |\n|------|------|----|-----|\n| 2026-01-01T00:00Z | — | new | test-agent |\n";
    let content = format!("{frontmatter}{body}");
    git(p, &["checkout", &branch]);
    std::fs::write(p.join(&ticket_path), &content).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", &ticket_path]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "add amendment requests"]);
    git(p, &["checkout", "-"]);

    // Use a no-op editor so the spec is not modified interactively.
    std::env::set_var("EDITOR", "true");
    apm::cmd::review::run(p, &id, Some("ammend".to_string()), true).unwrap();
    std::env::remove_var("EDITOR");

    let committed = branch_content(p, &branch, &ticket_path);
    assert!(committed.contains("- [ ] plain item"), "plain bullet should be converted to checkbox");
    assert!(!committed.contains("\n- plain item\n"), "plain bullet should no longer appear as-is");
    assert!(committed.contains("- [ ] already a checkbox"), "existing checkbox should be unchanged");
    assert!(committed.contains("- [x] already checked"), "checked item should be unchanged");
}

// ── apm clean ────────────────────────────────────────────────────────────────

/// Write a closed ticket to a branch. Returns the branch name and rel_path.
fn write_closed_ticket(dir: &std::path::Path, id: u32, slug: &str) -> (String, String) {
    let branch = format!("ticket/{id:04}-{slug}");
    let filename = format!("{id:04}-{slug}.md");
    let rel_path = format!("tickets/{filename}");
    let content = format!(
        "+++\nid = {id}\ntitle = \"{slug}\"\nstate = \"closed\"\nbranch = \"{branch}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|"
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
    std::fs::write(dir.join(&rel_path), &content).unwrap();
    git(dir, &["-c", "commit.gpgsign=false", "add", &rel_path]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", &format!("ticket({id}): close")]);
    git(dir, &["checkout", "main"]);
    (branch, rel_path)
}

/// Merge a branch into main via --no-ff.
fn merge_into_main(dir: &std::path::Path, branch: &str) {
    git(dir, &["-c", "commit.gpgsign=false", "merge", "--no-ff", branch, "-m", &format!("Merge {branch}")]);
}

fn branch_exists(dir: &std::path::Path, branch: &str) -> bool {
    std::process::Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn clean_happy_path_removes_closed_branch() {
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "done");
    merge_into_main(p, &branch);

    apm::cmd::clean::run(p, false).unwrap();

    assert!(!branch_exists(p, &branch), "branch should have been removed");
}

#[test]
fn clean_dry_run_includes_state_in_output() {
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "dry");
    merge_into_main(p, &branch);

    // dry_run=true should not actually delete anything
    apm::cmd::clean::run(p, true).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed in dry-run");
}

#[test]
fn clean_skips_ticket_not_on_main() {
    // Ticket branch is "merged" via -s ours (tip reachable from main),
    // but the ticket file is NOT present on main — should warn and skip.
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "ghost");

    // -s ours: makes branch tip reachable from main without bringing content
    git(p, &["-c", "commit.gpgsign=false", "merge", "-s", "ours", &branch, "-m", "ours merge"]);

    apm::cmd::clean::run(p, false).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed — ticket not on main");
}

#[test]
fn clean_skips_state_mismatch_between_branch_and_main() {
    // Ticket is closed on branch and merged into main. Then main's copy
    // gets updated to a different state (simulating a buggy sync). Clean should
    // detect the mismatch and skip.
    let dir = setup();
    let p = dir.path();
    let (branch, rel_path) = write_closed_ticket(p, 1, "mismatch");
    merge_into_main(p, &branch);

    // Overwrite the ticket on main to a different state
    let main_content = "+++\nid = 1\ntitle = \"mismatch\"\nstate = \"new\"\nbranch = \"ticket/0001-mismatch\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|";
    std::fs::write(p.join(&rel_path), main_content).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", &rel_path]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "update ticket state on main"]);

    apm::cmd::clean::run(p, false).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed — state mismatch");
}

#[test]
fn clean_treats_closed_as_terminal_without_config_entry() {
    // Config has no "closed" state entry at all, but clean should still treat it as terminal.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    // Config with no terminal states defined
    std::fs::write(
        p.join("apm.toml"),
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[agents]
max_concurrent = 1

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id    = "new"
label = "New"
"#,
    ).unwrap();
    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();

    let (branch, _) = write_closed_ticket(p, 1, "no-terminal-config");
    merge_into_main(p, &branch);

    apm::cmd::clean::run(p, false).unwrap();

    assert!(!branch_exists(p, &branch), "closed should be treated as terminal even without config entry");
}

#[test]
fn clean_skips_local_tip_ahead_of_remote() {
    // Set up a bare remote, clone from it, merge a closed ticket, then make an
    // extra local commit so local tip ≠ remote tip. Clean should skip.
    let bare = tempfile::tempdir().unwrap();
    let bp = bare.path();
    git(bp, &["init", "--bare", "-q"]);

    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["clone", &bp.to_string_lossy(), "."]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);

    std::fs::write(
        p.join("apm.toml"),
        r#"[project]
name = "test"

[tickets]
dir = "tickets"

[agents]
max_concurrent = 1

[workflow.prioritization]
priority_weight = 10.0
effort_weight = -2.0
risk_weight = -1.0

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#,
    ).unwrap();
    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    git(p, &["push", "origin", "main"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();

    // Create closed ticket branch, merge into main, push both
    let (branch, _) = write_closed_ticket(p, 1, "diverged");
    git(p, &["push", "origin", &branch]);
    merge_into_main(p, &branch);
    git(p, &["push", "origin", "main"]);

    // Now make an extra commit on the ticket branch (without pushing)
    git(p, &["checkout", &branch]);
    std::fs::write(p.join("tickets/0001-diverged.md"), "extra change").unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", "tickets/0001-diverged.md"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "extra local commit"]);
    git(p, &["checkout", "main"]);

    // Local tip ≠ remote tip → should skip
    apm::cmd::clean::run(p, false).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed — local tip ahead of remote");
}

// --- resolve_agent_name fallback ---

#[test]
fn start_without_apm_agent_name_uses_fallback() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-fallback", "0001-fallback.md", "ready", 1, "fallback");
    // Use a pre-resolved name to avoid env var manipulation in a concurrent test context.
    // The important thing is that run() accepts an explicit agent_name and stores it.
    apm::cmd::start::run(p, "0001", true, false, false, "ci-agent").unwrap();
    let content = branch_content(p, "ticket/0001-fallback", "tickets/0001-fallback.md");
    assert!(content.contains("state = \"in_progress\""), "ticket should be in_progress: {content}");
    assert!(content.contains("agent = \"ci-agent\""), "agent should be ci-agent: {content}");
}

// ---------------------------------------------------------------------------
// workers
// ---------------------------------------------------------------------------

fn setup_with_worktrees() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();

    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);

    // worktrees dir inside the tempdir to keep tests self-contained.
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

    git(p, &["add", "apm.toml"]);
    git(p, &[
        "-c", "commit.gpgsign=false",
        "commit", "-m", "init", "--allow-empty",
    ]);

    std::fs::create_dir_all(p.join("tickets")).unwrap();
    dir
}

#[test]
fn workers_no_worktrees_returns_ok() {
    let dir = setup();
    let p = dir.path();
    // No ticket worktrees, so workers list should succeed (and print nothing).
    apm::cmd::workers::run(p, None, None).unwrap();
}

#[test]
fn workers_kill_no_pid_file_errors() {
    let dir = setup_with_worktrees();
    let p = dir.path();
    std::env::set_var("APM_AGENT_NAME", "test-agent");

    apm::cmd::new::run(p, "kill test ticket".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(p, "kill-test-ticket");
    apm::cmd::state::run(p, &id, "ready".into(), true, false).unwrap();

    // Provision a worktree without writing a pid file.
    apm_core::git::ensure_worktree(p, &p.join("worktrees"), &find_ticket_branch(p, "kill-test-ticket")).unwrap();

    // --kill should return an error since there is no pid file.
    let result = apm::cmd::workers::run(p, None, Some(&id));
    assert!(result.is_err(), "expected error when no pid file present");
    let msg = format!("{:#}", result.unwrap_err());
    assert!(
        msg.contains("not running") || msg.contains(".apm-worker.pid"),
        "unexpected error message: {msg}"
    );
}

#[test]
fn workers_stale_pid_file_detected() {
    let dir = setup_with_worktrees();
    let p = dir.path();
    std::env::set_var("APM_AGENT_NAME", "test-agent");

    apm::cmd::new::run(p, "stale pid ticket".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(p, "stale-pid-ticket");
    apm::cmd::state::run(p, &id, "ready".into(), true, false).unwrap();
    let branch = find_ticket_branch(p, "stale-pid-ticket");
    apm_core::git::ensure_worktree(p, &p.join("worktrees"), &branch).unwrap();
    let wt_name = branch.replace('/', "-");
    let wt_path = p.join("worktrees").join(&wt_name);
    std::fs::create_dir_all(&wt_path).unwrap();

    // Write a stale pid file (PID 99999999 is almost certainly not running).
    let pid_file = wt_path.join(".apm-worker.pid");
    std::fs::write(
        &pid_file,
        r#"{"pid":99999999,"ticket_id":"XXXX","started_at":"2026-01-01T00:00:00+00:00"}"#,
    )
    .unwrap();

    // workers list should succeed (stale entry is shown as "crashed", not an error).
    apm::cmd::workers::run(p, None, None).unwrap();
}

#[test]
fn workers_kill_stale_pid_errors() {
    let dir = setup_with_worktrees();
    let p = dir.path();
    std::env::set_var("APM_AGENT_NAME", "test-agent");

    apm::cmd::new::run(p, "kill stale ticket".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(p, "kill-stale-ticket");
    apm::cmd::state::run(p, &id, "ready".into(), true, false).unwrap();
    let branch = find_ticket_branch(p, "kill-stale-ticket");
    apm_core::git::ensure_worktree(p, &p.join("worktrees"), &branch).unwrap();
    let wt_name = branch.replace('/', "-");
    let wt_dir = p.join("worktrees").join(&wt_name);
    std::fs::create_dir_all(&wt_dir).unwrap();
    let pid_file = wt_dir.join(".apm-worker.pid");
    std::fs::write(
        &pid_file,
        r#"{"pid":99999999,"ticket_id":"XXXX","started_at":"2026-01-01T00:00:00+00:00"}"#,
    )
    .unwrap();

    // --kill on a stale pid should return an error.
    let result = apm::cmd::workers::run(p, None, Some(&id));
    let err_msg = match &result {
        Err(e) => format!("{e:#}"),
        Ok(_) => String::new(),
    };
    assert!(result.is_err(), "expected error when pid is stale");
    assert!(
        err_msg.contains("not running"),
        "unexpected error (expected 'not running'): {err_msg}"
    );
    // The stale pid file should be cleaned up.  Check via git's recorded path
    // to handle macOS symlink resolution (/var -> /private/var).
    let real_wt = apm_core::git::find_worktree_for_branch(p, &branch)
        .expect("worktree must still be registered");
    assert!(
        !real_wt.join(".apm-worker.pid").exists(),
        "stale pid file should be removed on failed kill"
    );
}

fn setup_with_strict_transitions() -> TempDir {
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

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "manual"

[[workflow.states]]
id    = "specd"
label = "Specd"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

  [[workflow.states.transitions]]
  to      = "implemented"
  trigger = "manual"

[[workflow.states]]
id    = "implemented"
label = "Implemented"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"

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

#[test]
fn state_force_bypasses_transition_rules() {
    let dir = setup_with_strict_transitions();
    let p = dir.path();

    apm::cmd::new::run(p, "force test".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(p, "force-test");

    // Advance to in_progress (new → in_progress is valid).
    apm::cmd::state::run(p, &id, "in_progress".into(), true, false).unwrap();

    // Without --force, going back to new from in_progress should fail (not in allowed transitions).
    let result = apm::cmd::state::run(p, &id, "new".into(), true, false);
    assert!(result.is_err(), "expected transition rejection without --force");

    // With --force, the same transition must succeed.
    apm::cmd::state::run(p, &id, "new".into(), true, true).unwrap();

    // Verify the ticket is back in "new".
    let tickets = apm_core::ticket::load_all_from_git(p, std::path::Path::new("tickets")).unwrap();
    let t = tickets.iter().find(|t| t.frontmatter.id == id).unwrap();
    assert_eq!(t.frontmatter.state, "new");
}

#[test]
fn state_force_implemented_from_in_progress() {
    let dir = setup_with_strict_transitions();
    let p = dir.path();

    apm::cmd::new::run(p, "force progress".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(p, "force-progress");

    // Advance to in_progress.
    apm::cmd::state::run(p, &id, "in_progress".into(), true, false).unwrap();

    // Without --force, going to new from in_progress should fail.
    let result = apm::cmd::state::run(p, &id, "new".into(), true, false);
    assert!(result.is_err(), "expected transition rejection without --force");

    // With --force, new is reachable from in_progress.
    apm::cmd::state::run(p, &id, "new".into(), true, true).unwrap();

    let tickets = apm_core::ticket::load_all_from_git(p, std::path::Path::new("tickets")).unwrap();
    let t = tickets.iter().find(|t| t.frontmatter.id == id).unwrap();
    assert_eq!(t.frontmatter.state, "new");
}

#[test]
fn state_force_still_rejects_unknown_state() {
    let dir = setup();
    let p = dir.path();

    apm::cmd::new::run(p, "force unknown".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(p, "force-unknown");

    // --force does not allow transitioning to a state that doesn't exist in config.
    let result = apm::cmd::state::run(p, &id, "nonexistent_state".into(), true, true);
    assert!(result.is_err(), "expected error for unknown state even with --force");
}

#[test]
fn state_force_does_not_skip_doc_validation() {
    let dir = setup();
    let p = dir.path();

    apm::cmd::new::run(p, "force doc valid".into(), true, false, None, None, true).unwrap();
    let id = find_ticket_id(p, "force-doc-valid");

    // Transitioning to "specd" without a valid spec should still fail even with --force.
    let result = apm::cmd::state::run(p, &id, "specd".into(), true, true);
    assert!(result.is_err(), "expected spec validation to still fail with --force");
}
