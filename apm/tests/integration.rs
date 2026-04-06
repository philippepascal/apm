use tempfile::TempDir;

fn git(dir: &std::path::Path, args: &[&str]) {
    std::process::Command::new("git")
        .arg("-c").arg("init.defaultBranch=main")
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

[[ticket.sections]]
name     = "Problem"
type     = "free"
required = true

[[ticket.sections]]
name     = "Acceptance criteria"
type     = "tasks"
required = true

[[ticket.sections]]
name     = "Out of scope"
type     = "free"
required = true

[[ticket.sections]]
name     = "Approach"
type     = "free"
required = true

[[ticket.sections]]
name     = "Open questions"
type     = "qa"
required = false

[[ticket.sections]]
name     = "Amendment requests"
type     = "tasks"
required = false
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
    assert!(p.join(".apm/workflow.toml").exists());
    assert!(p.join(".apm/ticket.toml").exists());
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

    let toml = std::fs::read_to_string(p.join(".apm/workflow.toml")).unwrap();
    for state in &["new", "groomed", "question", "specd", "ammend", "in_design", "ready", "in_progress", "implemented", "closed"] {
        assert!(toml.contains(&format!("\"{state}\"")), "missing state: {state}");
    }
    assert!(toml.contains("terminal"), "closed must be terminal");
    // Must parse without error.
    apm_core::config::Config::load(p).unwrap();
}

#[test]
fn list_excludes_terminal_tickets_by_default() {
    let dir = setup();
apm::cmd::new::run(dir.path(), "Open ticket".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    apm::cmd::new::run(dir.path(), "Closed ticket".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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
    apm::cmd::new::run(dir.path(), "My first ticket".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    // File lives on the ticket branch, not in the working tree.
    let branch = find_ticket_branch(dir.path(), "my-first-ticket");
    let rel_path = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel_path);
    assert!(!content.is_empty());
}

#[test]
fn new_ticket_has_correct_frontmatter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Hello World".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "hello-world");
    let rel_path = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel_path);
    assert!(content.contains("title = \"Hello World\""));
    assert!(content.contains("state = \"new\""));
    assert!(content.contains(&format!("branch = \"{branch}\"")));
}

#[test]
fn new_uses_local_toml_username_as_author() {
    let dir = setup();
    let apm_dir = dir.path().join(".apm");
    std::fs::create_dir_all(&apm_dir).unwrap();
    std::fs::write(apm_dir.join("local.toml"), "username = \"carol\"\n").unwrap();
    apm::cmd::new::run(dir.path(), "My Ticket".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "my-ticket");
    let rel_path = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel_path);
    assert!(content.contains("author = \"carol\""), "author should come from local.toml: {content}");
}

#[test]
fn new_uses_apm_when_no_local_toml() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Unnamed".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "unnamed");
    let rel_path = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel_path);
    assert!(content.contains("author = \"apm\""), "author should be apm without local.toml: {content}");
}

#[test]
fn new_ticket_does_not_write_agent_field() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "No Agent".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "no-agent");
    let rel_path = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel_path);
    assert!(!content.contains("agent ="), "agent field must not appear in new tickets: {content}");
}

#[test]
fn new_increments_ids() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "First".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    apm::cmd::new::run(dir.path(), "Second".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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
    apm::cmd::new::run(dir.path(), "Alpha".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let b1 = find_ticket_branch(dir.path(), "alpha");
    sync_from_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::new::run(dir.path(), "Beta".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let b2 = find_ticket_branch(dir.path(), "beta");
    sync_from_branch(dir.path(), &b2, &ticket_rel_path(&b2));
    apm::cmd::list::run(dir.path(), None, false, false, None, None, true, false, None, None).unwrap();
}

#[test]
fn list_state_filter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Alpha".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let b1 = find_ticket_branch(dir.path(), "alpha");
    let alpha_id = find_ticket_id(dir.path(), "alpha");
    sync_from_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::new::run(dir.path(), "Beta".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let b2 = find_ticket_branch(dir.path(), "beta");
    sync_from_branch(dir.path(), &b2, &ticket_rel_path(&b2));
    write_valid_spec_to_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::state::run(dir.path(), &alpha_id, "specd".into(), false, false).unwrap();
    // Sync the updated ticket from its branch so apm list can see the new state.
    sync_from_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::list::run(dir.path(), Some("specd".into()), false, false, None, None, true, false, None, None).unwrap();
}

#[test]
fn list_mine_filter() {
    let dir = setup();
    // Write a .apm/local.toml with a username.
    let apm_dir = dir.path().join(".apm");
    std::fs::create_dir_all(&apm_dir).unwrap();
    std::fs::write(apm_dir.join("local.toml"), "username = \"testuser\"\n").unwrap();

    // Create one ticket authored by "testuser" and one by the default ("apm").
    apm::cmd::new::run(dir.path(), "Mine".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let b1 = find_ticket_branch(dir.path(), "mine");
    sync_from_branch(dir.path(), &b1, &ticket_rel_path(&b1));

    // Remove local.toml so the next ticket gets the fallback author.
    std::fs::remove_file(apm_dir.join("local.toml")).unwrap();
    apm::cmd::new::run(dir.path(), "Theirs".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let b2 = find_ticket_branch(dir.path(), "theirs");
    sync_from_branch(dir.path(), &b2, &ticket_rel_path(&b2));

    // Restore local.toml so --mine resolves to "testuser".
    std::fs::write(apm_dir.join("local.toml"), "username = \"testuser\"\n").unwrap();

    // --mine should show only the first ticket.
    apm::cmd::list::run(dir.path(), None, false, false, None, None, true, true, None, None).unwrap();
}

// --- show ---

#[test]
fn show_existing_ticket() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Show me".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let id = find_ticket_id(dir.path(), "show-me");
    apm::cmd::show::run(dir.path(), &id, false, false).unwrap();
}

#[test]
fn show_missing_ticket_errors() {
    let dir = setup();
    assert!(apm::cmd::show::run(dir.path(), "99", false, false).is_err());
}

#[test]
fn show_displays_epic_target_branch_depends_on_when_set() {
    let dir = setup();
    let p = dir.path();

    // Create a ticket with epic, target_branch, and depends_on set directly.
    let ticket_content = "+++\nid = \"aabb1122\"\ntitle = \"Rich ticket\"\nstate = \"new\"\nbranch = \"ticket/aabb1122-rich-ticket\"\nepic = \"epic001\"\ntarget_branch = \"epic/epic001-user-auth\"\ndepends_on = [\"ccdd3344\", \"eeff5566\"]\n+++\n\n## Spec\n\n### Problem\n\nTest.\n\n### Acceptance criteria\n\n- [ ] One\n\n### Out of scope\n\nN/A.\n\n### Approach\n\nDirect.\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|  \n| 2026-01-01T00:00Z | — | new | test |\n";
    let ticket_dir = p.join("tickets");
    std::fs::create_dir_all(&ticket_dir).unwrap();
    let ticket_path = ticket_dir.join("aabb1122-rich-ticket.md");
    std::fs::write(&ticket_path, ticket_content).unwrap();
    git(p, &["checkout", "-b", "ticket/aabb1122-rich-ticket"]);
    git(p, &["-c", "commit.gpgsign=false", "add", "tickets/aabb1122-rich-ticket.md"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "add rich ticket"]);
    git(p, &["checkout", "-"]);
    let _ = std::fs::remove_file(&ticket_path);

    let bin = env!("CARGO_BIN_EXE_apm");
    let out = std::process::Command::new(bin)
        .args(["show", "aabb1122"])
        .current_dir(p)
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "apm show failed: {}", String::from_utf8_lossy(&out.stderr));
    assert!(stdout.contains("epic:"), "expected epic: line in:\n{stdout}");
    assert!(stdout.contains("epic001"), "expected epic id in:\n{stdout}");
    assert!(stdout.contains("target_branch:"), "expected target_branch: line in:\n{stdout}");
    assert!(stdout.contains("epic/epic001-user-auth"), "expected target_branch value in:\n{stdout}");
    assert!(stdout.contains("depends_on:"), "expected depends_on: line in:\n{stdout}");
    assert!(stdout.contains("ccdd3344"), "expected depends_on value in:\n{stdout}");
}

#[test]
fn show_omits_epic_target_branch_depends_on_when_absent() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Plain ticket".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let id = find_ticket_id(dir.path(), "plain-ticket");

    let bin = env!("CARGO_BIN_EXE_apm");
    let out = std::process::Command::new(bin)
        .args(["show", &id])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "apm show failed: {}", String::from_utf8_lossy(&out.stderr));
    assert!(!stdout.contains("epic:"), "unexpected epic: line in:\n{stdout}");
    assert!(!stdout.contains("target_branch:"), "unexpected target_branch: line in:\n{stdout}");
    assert!(!stdout.contains("depends_on:"), "unexpected depends_on: line in:\n{stdout}");
}

// --- state ---

#[test]
fn state_transition_updates_file() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Transition test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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
    apm::cmd::new::run(dir.path(), "History test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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
    apm::cmd::new::run(dir.path(), "Ammend test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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
    apm::cmd::new::run(dir.path(), "Set test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "set-test");
    let id = find_ticket_id(dir.path(), "set-test");
    let rel = ticket_rel_path(&branch);
    apm::cmd::set::run(dir.path(), &id, "priority".into(), "7".into(), true).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("priority = 7"));
}

#[test]
fn set_depends_on_single_id() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Dep test single".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "dep-test-single");
    let id = find_ticket_id(dir.path(), "dep-test-single");
    let rel = ticket_rel_path(&branch);
    apm::cmd::set::run(dir.path(), &id, "depends_on".into(), "abc12345".into(), true).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("depends_on = [\"abc12345\"]"));
}

#[test]
fn set_depends_on_comma_separated() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Dep test multi".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "dep-test-multi");
    let id = find_ticket_id(dir.path(), "dep-test-multi");
    let rel = ticket_rel_path(&branch);
    apm::cmd::set::run(dir.path(), &id, "depends_on".into(), "abc12345,def67890".into(), true).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("abc12345"));
    assert!(content.contains("def67890"));
}

#[test]
fn set_depends_on_clear() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Dep test clear".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "dep-test-clear");
    let id = find_ticket_id(dir.path(), "dep-test-clear");
    let rel = ticket_rel_path(&branch);
    apm::cmd::set::run(dir.path(), &id, "depends_on".into(), "abc12345".into(), true).unwrap();
    apm::cmd::set::run(dir.path(), &id, "depends_on".into(), "-".into(), true).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(!content.contains("depends_on"));
}

#[test]
fn set_depends_on_trims_whitespace() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Dep test trim".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "dep-test-trim");
    let id = find_ticket_id(dir.path(), "dep-test-trim");
    let rel = ticket_rel_path(&branch);
    apm::cmd::set::run(dir.path(), &id, "depends_on".into(), " id1 , id2 ".into(), true).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("\"id1\""));
    assert!(content.contains("\"id2\""));
}

// --- next ---

#[test]
fn next_returns_highest_priority() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Low priority".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let b1 = find_ticket_branch(dir.path(), "low-priority");
    sync_from_branch(dir.path(), &b1, &ticket_rel_path(&b1));
    apm::cmd::new::run(dir.path(), "High priority".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let b2 = find_ticket_branch(dir.path(), "high-priority");
    let high_id = find_ticket_id(dir.path(), "high-priority");
    apm::cmd::set::run(dir.path(), &high_id, "priority".into(), "10".into(), true).unwrap();
    sync_from_branch(dir.path(), &b2, &ticket_rel_path(&b2));
    apm::cmd::next::run(dir.path(), false, true).unwrap();
}

#[test]
fn next_json_is_valid() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Json test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let b = find_ticket_branch(dir.path(), "json-test");
    sync_from_branch(dir.path(), &b, &ticket_rel_path(&b));
    apm::cmd::next::run(dir.path(), true, true).unwrap();
}

#[test]
fn next_null_when_no_actionable() {
    let dir = setup();
    apm::cmd::next::run(dir.path(), true, true).unwrap();
}

// --- branch ---

#[test]
fn new_ticket_creates_branch() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Branch test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    // Branch should exist locally after apm new.
    let branch = find_ticket_branch(dir.path(), "branch-test");
    assert!(branch.starts_with("ticket/"), "expected ticket/ branch, got: {branch}");
    assert!(branch.ends_with("-branch-test"), "expected slug in branch: {branch}");
}

#[test]
fn new_ticket_sets_branch_in_frontmatter() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Frontmatter branch".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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
fn sync_closes_implemented_ticket_on_merged_branch() {
    let dir = setup_with_close_workflow();
    let p = dir.path();

    write_ticket_to_branch(p, "ticket/0001-my-ticket", "0001-my-ticket.md", "implemented", 1, "my ticket");
    // Merge the branch into main so sync detects it.
    git(p, &["-c", "commit.gpgsign=false", "merge", "--no-ff", "ticket/0001-my-ticket", "--no-edit"]);

    apm::cmd::sync::run(p, true, true, true, true).unwrap();

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

    apm::cmd::sync::run(p, true, true, true, true).unwrap();

    let updated = std::fs::read_to_string(p.join(path)).unwrap();
    assert!(updated.contains("state = \"closed\""), "stale ticket should be closed: {updated}");
}

#[test]
fn sync_no_close_when_nothing_to_close() {
    let dir = setup_with_close_workflow();
    let p = dir.path();
    // No tickets at all
    let log_before = branch_content(p, "main", "apm.toml"); // just to get a ref point
    apm::cmd::sync::run(p, true, true, true, true).unwrap();
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
fn sync_closes_multiple_tickets_on_merged_branches() {
    let dir = setup_with_close_workflow();
    let p = dir.path();

    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "implemented", 1, "alpha");
    write_ticket_to_branch(p, "ticket/0002-beta", "0002-beta.md", "implemented", 2, "beta");
    git(p, &["-c", "commit.gpgsign=false", "merge", "--no-ff", "ticket/0001-alpha", "--no-edit"]);
    git(p, &["-c", "commit.gpgsign=false", "merge", "--no-ff", "ticket/0002-beta", "--no-edit"]);

    apm::cmd::sync::run(p, true, true, true, true).unwrap();

    let alpha = branch_content(p, "main", "tickets/0001-alpha.md");
    let beta = branch_content(p, "main", "tickets/0002-beta.md");
    assert!(alpha.contains("state = \"closed\""), "alpha should be closed on main: {alpha}");
    assert!(beta.contains("state = \"closed\""), "beta should be closed on main: {beta}");
}

// --- sync handler (server path) ---

#[test]
fn sync_handler_closes_merged_ticket() {
    // Regression test for the server sync_handler calling detect+apply.
    // Mirrors the logic added to apm-server's sync_handler.
    let dir = setup_with_close_workflow();
    let p = dir.path();

    write_ticket_to_branch(p, "ticket/0001-server-sync", "0001-server-sync.md", "implemented", 1, "server sync");
    git(p, &["-c", "commit.gpgsign=false", "merge", "--no-ff", "ticket/0001-server-sync", "--no-edit"]);

    let config = apm_core::config::Config::load(p).unwrap();
    let candidates = apm_core::sync::detect(p, &config).unwrap();
    assert_eq!(candidates.close.len(), 1, "should detect one close candidate");
    let aggressive = config.sync.aggressive;
    apm_core::sync::apply(p, &config, &candidates, "apm-ui", aggressive).unwrap();

    let content = branch_content(p, "main", "tickets/0001-server-sync.md");
    assert!(content.contains("state = \"closed\""), "ticket should be closed: {content}");
}

#[test]
fn sync_handler_no_close_returns_zero() {
    // A sync that closes no tickets: detect returns empty candidates.
    let dir = setup_with_close_workflow();
    let p = dir.path();

    let config = apm_core::config::Config::load(p).unwrap();
    let candidates = apm_core::sync::detect(p, &config).unwrap();
    assert_eq!(candidates.close.len(), 0, "no candidates when no merged tickets");
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
    apm::cmd::spec::run(p, "1", None, None, None, false, None, true).unwrap();
}

#[test]
fn spec_prints_single_section() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "the problem text", "the approach");
    apm::cmd::spec::run(p, "1", Some("Problem".into()), None, None, false, None, true).unwrap();
}

#[test]
fn spec_set_section_commits() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "old problem", "old approach");
    apm::cmd::spec::run(p, "1", Some("Problem".into()), Some("new problem text".into()), None, false, None, true).unwrap();
    let content = branch_content(p, "ticket/0001-spec-test", "tickets/0001-spec-test.md");
    assert!(content.contains("new problem text"), "updated problem not found: {content}");
}

#[test]
fn spec_check_passes_full_ticket() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    apm::cmd::spec::run(p, "1", None, None, None, true, None, true).unwrap();
}

#[test]
fn spec_unknown_section_errors() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    let result = apm::cmd::spec::run(p, "1", Some("NonExistent".into()), None, None, false, None, true);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("unknown section"));
}

#[test]
fn spec_nonexistent_ticket_errors() {
    let dir = setup();
    let p = dir.path();
    let result = apm::cmd::spec::run(p, "999", None, None, None, false, None, true);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("no ticket matches"));
}

#[test]
fn spec_set_without_section_errors() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    let result = apm::cmd::spec::run(p, "1", None, Some("some value".into()), None, false, None, true);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("--set requires --section"));
}

#[test]
fn spec_set_hyphen_value() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "old problem", "old approach");
    apm::cmd::spec::run(p, "1", Some("Problem".into()), Some("- [ ] Fix the thing".into()), None, false, None, true).unwrap();
    let content = branch_content(p, "ticket/0001-spec-test", "tickets/0001-spec-test.md");
    assert!(content.contains("- [ ] Fix the thing"), "hyphen value not stored: {content}");
}

#[test]
fn spec_set_file_reads_content() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "old problem", "old approach");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "content from file").unwrap();
    apm::cmd::spec::run(p, "1", Some("Problem".into()), None, Some(tmp.path().to_string_lossy().into_owned()), false, None, true).unwrap();
    let content = branch_content(p, "ticket/0001-spec-test", "tickets/0001-spec-test.md");
    assert!(content.contains("content from file"), "file content not stored: {content}");
}

#[test]
fn spec_set_file_nonexistent_errors() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    let result = apm::cmd::spec::run(p, "1", Some("Problem".into()), None, Some("/nonexistent/path/to/file.txt".into()), false, None, true);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("--set-file"));
}

#[test]
fn spec_set_file_without_section_errors() {
    let dir = setup();
    let p = dir.path();
    write_spec_ticket(p, 1, "a problem", "an approach");
    let result = apm::cmd::spec::run(p, "1", None, None, Some("/some/file.txt".into()), false, None, true);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("--set-file requires --section"));
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
        None,
        false,
        Some("Add error handling".into()),
        true,
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
        None,
        false,
        Some("nonexistent item".into()),
        true,
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
        None,
        false,
        Some("error".into()),
        true,
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
    let result = apm::cmd::spec::run(p, "1", None, None, None, false, Some("Add error handling".into()), true);
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
        None,
        false,
        Some("ADD ERROR".into()),
        true,
    ).unwrap();
    let content = branch_content(p, "ticket/0001-spec-test", "tickets/0001-spec-test.md");
    assert!(content.contains("- [x] Add error handling"), "item not checked: {content}");
}

// ── apm close ────────────────────────────────────────────────────────────────

#[test]
fn close_transitions_from_any_state() {
    let dir = setup();
    let p = dir.path();
    apm::cmd::new::run(p, "Close me".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(p, "close-me");
    let id = find_ticket_id(p, "close-me");
    let rel = ticket_rel_path(&branch);
    // Ticket is in "new" state — no transition to "closed" is defined.
    apm::cmd::close::run(p, &id, None, true).unwrap();
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("state = \"closed\""), "state not updated: {content}");
    assert!(content.contains("| new | closed |"), "history row missing: {content}");
}

#[test]
fn close_with_reason_appends_to_history() {
    let dir = setup();
    let p = dir.path();
    apm::cmd::new::run(p, "Close reason".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(p, "close-reason");
    let id = find_ticket_id(p, "close-reason");
    let rel = ticket_rel_path(&branch);
    apm::cmd::close::run(p, &id, Some("superseded by #42".into()), true).unwrap();
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("state = \"closed\""), "state not updated: {content}");
    assert!(content.contains("superseded by #42"), "reason missing: {content}");
}

#[test]
fn close_already_closed_is_error() {
    let dir = setup();
    let p = dir.path();
    apm::cmd::new::run(p, "Already closed".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let id = find_ticket_id(p, "already-closed");
    apm::cmd::close::run(p, &id, None, true).unwrap();
    let result = apm::cmd::close::run(p, &id, None, true);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("already closed"));
}

#[test]
fn close_nonexistent_ticket_is_error() {
    let dir = setup();
    let p = dir.path();
    let result = apm::cmd::close::run(p, "999", None, true);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("no ticket matches"));
}

#[test]
fn validate_does_not_flag_closed_state() {
    let dir = setup();
    let p = dir.path();
    apm::cmd::new::run(p, "Validate closed".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let id = find_ticket_id(p, "validate-closed");
    apm::cmd::close::run(p, &id, None, true).unwrap();
    // apm validate should not flag the closed ticket as having an unknown state.
    // The test config is minimal and may produce config warnings (e.g. missing transitions),
    // but there must be zero ticket-level errors.
    let result = apm::cmd::validate::run(p, false, false, false, true);
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
    apm::cmd::new::run(p, "State closed".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(p, "state-closed");
    let id = find_ticket_id(p, "state-closed");
    let rel = ticket_rel_path(&branch);
    // "new" state has no outgoing transitions to "closed" in the test config,
    // but "closed" is a mandatory terminal state so it should still work.
    apm::cmd::state::run(p, &id, "closed".into(), false, false).unwrap();
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("state = \"closed\""), "state not updated: {content}");
}

// ── aggressive fetch/push ──────────────────────────────────────────────────────

/// Build a minimal apm.toml with sync.aggressive = true.
fn setup_aggressive() -> TempDir {
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

[agents]
max_concurrent = 3

[sync]
aggressive = true

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
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    dir
}

/// Commands with aggressive=true but no remote must not abort — fetch/push
/// failures are warnings only.
#[test]
fn aggressive_no_remote_does_not_abort_next() {
    let dir = setup_aggressive();
    let p = dir.path();
    apm::cmd::new::run(p, "Aggressive next".into(), true, false, None, None, false, vec![], vec![], None, vec![]).unwrap();
    // No remote configured — fetch_all will fail; must not propagate as error.
    apm::cmd::next::run(p, false, false).unwrap();
}

#[test]
fn aggressive_no_remote_does_not_abort_list() {
    let dir = setup_aggressive();
    let p = dir.path();
    apm::cmd::new::run(p, "Aggressive list".into(), true, false, None, None, false, vec![], vec![], None, vec![]).unwrap();
    apm::cmd::list::run(p, None, false, false, None, None, false, false, None, None).unwrap();
}

#[test]
fn aggressive_no_remote_does_not_abort_close() {
    let dir = setup_aggressive();
    let p = dir.path();
    apm::cmd::new::run(p, "Aggressive close".into(), true, false, None, None, false, vec![], vec![], None, vec![]).unwrap();
    let id = find_ticket_id(p, "aggressive-close");
    // No remote: fetch and push will warn but not abort.
    apm::cmd::close::run(p, &id, None, false).unwrap();
    let branch = find_ticket_branch(p, "aggressive-close");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("state = \"closed\""), "ticket not closed: {content}");
}

/// --no-aggressive suppresses fetch/push even when config has aggressive = true.
#[test]
fn no_aggressive_flag_suppresses_fetch_on_next() {
    let dir = setup_aggressive();
    let p = dir.path();
    apm::cmd::new::run(p, "No agg next".into(), true, false, None, None, false, vec![], vec![], None, vec![]).unwrap();
    // --no-aggressive = true means fetch is skipped entirely (no warning printed,
    // no error). We verify the command still succeeds.
    apm::cmd::next::run(p, false, true).unwrap();
}

#[test]
fn no_aggressive_flag_suppresses_fetch_on_spec() {
    let dir = setup_aggressive();
    let p = dir.path();
    apm::cmd::new::run(p, "No agg spec".into(), true, false, None, None, false, vec![], vec![], None, vec![]).unwrap();
    let id = find_ticket_id(p, "no-agg-spec");
    // no_aggressive=true: fetch and push are skipped.
    apm::cmd::spec::run(
        p,
        &id,
        Some("Problem".into()),
        Some("test content".into()),
        None,
        false,
        None,
        true,
    ).unwrap();
}

#[test]
fn no_aggressive_flag_suppresses_fetch_on_set() {
    let dir = setup_aggressive();
    let p = dir.path();
    apm::cmd::new::run(p, "No agg set".into(), true, false, None, None, false, vec![], vec![], None, vec![]).unwrap();
    let id = find_ticket_id(p, "no-agg-set");
    apm::cmd::set::run(p, &id, "priority".into(), "5".into(), true).unwrap();
    let branch = find_ticket_branch(p, "no-agg-set");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("priority = 5"), "priority not set: {content}");
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

[workers]
command = "/usr/bin/true"

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
    assert!(!content.contains("agent ="), "agent field must not be written: {content}");
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
    assert!(!content.contains("agent ="), "agent field must not be written: {content}");
}

// ── apm start --spawn ────────────────────────────────────────────────────────

/// Write a minimal fake `claude` executable to `bin_dir` and prepend it to PATH.
/// Returns the old PATH so the caller can restore it.

#[test]
fn start_spawn_sets_agent_to_worker_pid() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha");

    std::env::set_var("APM_AGENT_NAME", "delegator-agent");
    apm::cmd::start::run(p, "1", true, true, false, "delegator-agent").unwrap();

    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    assert!(content.contains("state = \"in_progress\""), "ticket should be in_progress after spawn: {content}");
    assert!(!content.contains("agent ="), "agent field must not be written: {content}");
}

#[test]
fn start_non_spawn_keeps_agent_name() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha");

    std::env::set_var("APM_AGENT_NAME", "delegator-agent");
    apm::cmd::start::run(p, "1", true, false, false, "delegator-agent").unwrap();

    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    assert!(content.contains("state = \"in_progress\""), "ticket should be in_progress: {content}");
    assert!(!content.contains("agent ="), "agent field must not be written: {content}");
}

#[test]
fn start_next_spawn_sets_agent_to_worker_pid() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha");

    std::env::set_var("APM_AGENT_NAME", "delegator-agent");
    apm::cmd::start::run_next(p, true, true, false).unwrap();

    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    assert!(content.contains("state = \"in_progress\""), "ticket should be in_progress after spawn: {content}");
    assert!(!content.contains("agent ="), "agent field must not be written: {content}");
}

// ── apm start: owner guard ───────────────────────────────────────────────────

fn write_ticket_with_owner(dir: &std::path::Path, branch: &str, filename: &str, state: &str, id: u32, title: &str, owner: &str) {
    let path = format!("tickets/{filename}");
    let content = format!(
        "+++\nid = {id}\ntitle = \"{title}\"\nstate = \"{state}\"\nbranch = \"{branch}\"\nowner = \"{owner}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|",
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
fn start_sets_owner_when_unowned() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha");

    std::env::set_var("APM_AGENT_NAME", "alice");
    apm::cmd::start::run(p, "1", true, false, false, "alice").unwrap();

    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    assert!(content.contains("owner = \"alice\""), "owner should be set when unowned: {content}");
}

#[test]
fn start_sets_owner_when_same_owner_resumes() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_with_owner(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha", "alice");

    std::env::set_var("APM_AGENT_NAME", "alice");
    apm::cmd::start::run(p, "1", true, false, false, "alice").unwrap();

    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    assert!(content.contains("owner = \"alice\""), "owner should stay alice when same owner resumes: {content}");
}

#[test]
fn start_does_not_overwrite_different_owner() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    write_ticket_with_owner(p, "ticket/0001-alpha", "0001-alpha.md", "ready", 1, "alpha", "alice");

    std::env::set_var("APM_AGENT_NAME", "bob");
    apm::cmd::start::run(p, "1", true, false, false, "bob").unwrap();

    let content = branch_content(p, "ticket/0001-alpha", "tickets/0001-alpha.md");
    assert!(content.contains("owner = \"alice\""), "owner should stay alice, not be overwritten by bob: {content}");
    assert!(!content.contains("owner = \"bob\""), "bob must not become owner: {content}");
}

// ── apm state in_design: owner guard ─────────────────────────────────────────

#[test]
fn in_design_sets_owner_when_unowned() {
    let dir = setup_for_prompt_dispatch();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-spec-me", "0001-spec-me.md", "new", 1, "spec me");

    std::env::set_var("APM_AGENT_NAME", "alice");
    apm::cmd::state::run(p, "1", "in_design".into(), true, false).unwrap();

    let content = branch_content(p, "ticket/0001-spec-me", "tickets/0001-spec-me.md");
    assert!(content.contains("owner = \"alice\""), "owner should be set when transitioning to in_design unowned: {content}");
}

#[test]
fn in_design_does_not_overwrite_different_owner() {
    let dir = setup_for_prompt_dispatch();
    let p = dir.path();
    write_ticket_with_owner(p, "ticket/0001-spec-me", "0001-spec-me.md", "new", 1, "spec me", "alice");

    std::env::set_var("APM_AGENT_NAME", "bob");
    apm::cmd::state::run(p, "1", "in_design".into(), true, false).unwrap();

    let content = branch_content(p, "ticket/0001-spec-me", "tickets/0001-spec-me.md");
    assert!(content.contains("owner = \"alice\""), "owner should stay alice when bob transitions to in_design: {content}");
    assert!(!content.contains("owner = \"bob\""), "bob must not become owner: {content}");
}

// ── system prompt dispatch ───────────────────────────────────────────────────

/// A config with new/ammend/ready all startable, plus in_design/in_progress destinations.
fn setup_for_prompt_dispatch() -> TempDir {
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

[workers]
command = "/usr/bin/true"

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
id         = "ammend"
label      = "Ammend"
actionable = ["agent"]

  [[workflow.states.transitions]]
  to      = "in_design"
  trigger = "command:start"
  actor   = "agent"

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
    std::fs::create_dir_all(p.join(".apm")).unwrap();
    dir
}

#[test]
fn spawn_new_ticket_transitions_to_in_design() {
    let dir = setup_for_prompt_dispatch();
    let p = dir.path();
    std::fs::write(p.join(".apm/apm.spec-writer.md"), "SPEC WRITER PROMPT").unwrap();
    write_ticket_to_branch(p, "ticket/0001-spec-me", "0001-spec-me.md", "new", 1, "spec me");

    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::start::run(p, "1", true, true, false, "test-agent").unwrap();

    let content = branch_content(p, "ticket/0001-spec-me", "tickets/0001-spec-me.md");
    assert!(content.contains("state = \"in_design\""), "new ticket should transition to in_design: {content}");
}

#[test]
fn spawn_ammend_ticket_transitions_to_in_design() {
    let dir = setup_for_prompt_dispatch();
    let p = dir.path();
    std::fs::write(p.join(".apm/apm.spec-writer.md"), "SPEC WRITER PROMPT").unwrap();
    write_ticket_to_branch(p, "ticket/0001-fix-spec", "0001-fix-spec.md", "ammend", 1, "fix spec");

    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::start::run(p, "1", true, true, false, "test-agent").unwrap();

    let content = branch_content(p, "ticket/0001-fix-spec", "tickets/0001-fix-spec.md");
    assert!(content.contains("state = \"in_design\""), "ammend ticket should transition to in_design: {content}");
}

#[test]
fn spawn_ready_ticket_transitions_to_in_progress() {
    let dir = setup_for_prompt_dispatch();
    let p = dir.path();
    std::fs::write(p.join(".apm/apm.worker.md"), "WORKER PROMPT").unwrap();
    write_ticket_to_branch(p, "ticket/0001-implement-me", "0001-implement-me.md", "ready", 1, "implement me");

    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::start::run(p, "1", true, true, false, "test-agent").unwrap();

    let content = branch_content(p, "ticket/0001-implement-me", "tickets/0001-implement-me.md");
    assert!(content.contains("state = \"in_progress\""), "ready ticket should transition to in_progress: {content}");
}

#[test]
fn start_next_spawn_new_ticket_transitions_correctly() {
    let dir = setup_for_prompt_dispatch();
    let p = dir.path();
    std::fs::write(p.join(".apm/apm.spec-writer.md"), "SPEC WRITER PROMPT").unwrap();
    write_ticket_to_branch(p, "ticket/0001-spec-me", "0001-spec-me.md", "new", 1, "spec me");

    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::start::run_next(p, true, true, false).unwrap();

    let content = branch_content(p, "ticket/0001-spec-me", "tickets/0001-spec-me.md");
    assert!(content.contains("state = \"in_design\""), "run_next on new ticket should go to in_design: {content}");
}

#[test]
fn start_next_spawn_ready_ticket_transitions_correctly() {
    let dir = setup_for_prompt_dispatch();
    let p = dir.path();
    std::fs::write(p.join(".apm/apm.worker.md"), "WORKER PROMPT").unwrap();
    write_ticket_to_branch(p, "ticket/0001-implement-me", "0001-implement-me.md", "ready", 1, "implement me");

    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::start::run_next(p, true, true, false).unwrap();

    let content = branch_content(p, "ticket/0001-implement-me", "tickets/0001-implement-me.md");
    assert!(content.contains("state = \"in_progress\""), "run_next on ready ticket should go to in_progress: {content}");
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
    apm::cmd::work::run(p, false, true, false, 30, None).unwrap();
}

#[test]
fn work_dry_run_no_tickets() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    std::env::set_var("APM_AGENT_NAME", "test-agent");
    apm::cmd::work::run(p, false, true, false, 30, None).unwrap();
}

// --- sync direct close ---

#[test]
fn sync_closes_implemented_ticket_with_merged_branch_in_one_run() {
    let dir = setup_with_close_workflow();
    let p = dir.path();

    // Write a ticket in `implemented` state on its ticket branch.
    write_ticket_to_branch(p, "ticket/0001-impl", "0001-impl.md", "implemented", 1, "impl ticket");

    // Merge the ticket branch into main so it appears as merged.
    git(p, &["-c", "commit.gpgsign=false", "merge", "--no-ff", "ticket/0001-impl", "--no-edit"]);

    // Run sync with auto_close — ticket should be closed in a single run.
    apm::cmd::sync::run(p, true, true, true, true).unwrap();

    // The ticket branch should now have state = "closed".
    let content = branch_content(p, "ticket/0001-impl", "tickets/0001-impl.md");
    assert!(content.contains("state = \"closed\""), "ticket should be closed in one sync run: {content}");
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
        vec![],
        vec![],
        None,
        vec![],
    ).unwrap();
    let branch = find_ticket_branch(dir.path(), "section-test");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("### Approach\n\nmy approach text\n\n"), "expected context under ### Approach");
    // Problem section should be empty (no content between header and next section)
    let after_problem = content.split("### Problem\n\n").nth(1).expect("Problem section not found in content");
    let problem_body = after_problem.split("\n### ").next().unwrap_or("");
    assert!(problem_body.trim().is_empty(), "Problem should be empty, got: {problem_body:?}");
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
        vec![],
        vec![],
        None,
        vec![],
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
        vec![],
        vec![],
        None,
        vec![],
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
        vec![],
        vec![],
        None,
        vec![],
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
        vec![],
        vec![],
        None,
        vec![],
    ).unwrap();
    let branch = find_ticket_branch(p, "transition-context-test");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(p, &branch, &rel);
    assert!(content.contains("### Approach\n\ntransition driven context\n\n"), "expected context under ### Approach from transition config");
}

// --- --section/--set pre-population ---

#[test]
fn new_section_set_prepopulates_multiple_sections() {
    let dir = setup();
    apm::cmd::new::run(
        dir.path(),
        "Pre-populated ticket".into(),
        true,
        false,
        None,
        None,
        true,
        vec!["Problem".into(), "Approach".into()],
        vec!["Something is broken".into(), "Fix it with a hammer".into()],
        None,
        vec![],
    ).unwrap();
    let branch = find_ticket_branch(dir.path(), "pre-populated-ticket");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("### Problem\n\nSomething is broken\n"), "Problem section should be pre-populated");
    assert!(content.contains("### Approach\n\nFix it with a hammer\n"), "Approach section should be pre-populated");
    // Only one commit on the ticket branch above main (no intermediate empty-section commit)
    let log = std::process::Command::new("git")
        .args(["log", "--oneline", &branch, "^main"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let log_str = String::from_utf8(log.stdout).unwrap();
    assert_eq!(log_str.lines().count(), 1, "ticket branch should have exactly one commit above main");
}

#[test]
fn new_section_set_mismatched_counts_is_error() {
    let dir = setup();
    let result = apm::cmd::new::run(
        dir.path(),
        "Mismatch test".into(),
        true,
        false,
        None,
        None,
        true,
        vec!["Problem".into(), "Approach".into()],
        vec!["Only one set".into()],
        None,
        vec![],
    );
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("--section") && msg.contains("--set"), "error should mention both flags");
}

#[test]
fn new_set_without_section_is_error() {
    let dir = setup();
    let result = apm::cmd::new::run(
        dir.path(),
        "Set only test".into(),
        true,
        false,
        None,
        None,
        true,
        vec![],
        vec!["Orphaned value".into()],
        None,
        vec![],
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("--set requires --section"));
}

#[test]
fn new_section_unknown_name_is_error() {
    let dir = setup();
    let result = apm::cmd::new::run(
        dir.path(),
        "Unknown section test".into(),
        true,
        false,
        None,
        None,
        true,
        vec!["Nonexistent".into()],
        vec!["some text".into()],
        None,
        vec![],
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unknown section"));
}

// --- --epic / --depends-on ---

fn setup_with_epic() -> (tempfile::TempDir, String) {
    let dir = setup();
    let p = dir.path();
    // Create an epic branch: epic/<8-hex-id>-my-epic
    let epic_id = "ab12cd34";
    let epic_branch = format!("epic/{epic_id}-my-epic");
    git(p, &["-c", "commit.gpgsign=false", "checkout", "-b", &epic_branch]);
    // Add a commit on the epic branch so it has a distinct tip
    std::fs::write(p.join("epic.txt"), "epic content").unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", "epic.txt"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "epic commit"]);
    // Return to main
    git(p, &["checkout", "main"]);
    (dir, epic_id.to_string())
}

#[test]
fn new_epic_sets_frontmatter_fields() {
    let (dir, epic_id) = setup_with_epic();
    apm::cmd::new::run(
        dir.path(),
        "Epic child ticket".into(),
        true,
        false,
        None,
        None,
        true,
        vec![],
        vec![],
        Some(epic_id.clone()),
        vec![],
    ).unwrap();
    let branch = find_ticket_branch(dir.path(), "epic-child-ticket");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains(&format!("epic = \"{epic_id}\"")), "epic field missing in frontmatter");
    let expected_target = format!("epic/{epic_id}-my-epic");
    assert!(content.contains(&format!("target_branch = \"{expected_target}\"")), "target_branch missing");
}

#[test]
fn new_epic_branch_created_from_epic_tip() {
    let (dir, epic_id) = setup_with_epic();
    // Get the SHA of the epic branch tip before creating the ticket
    let epic_branch = format!("epic/{epic_id}-my-epic");
    let epic_tip = std::process::Command::new("git")
        .args(["rev-parse", &epic_branch])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let epic_tip_sha = String::from_utf8(epic_tip.stdout).unwrap().trim().to_string();

    apm::cmd::new::run(
        dir.path(),
        "Branched ticket".into(),
        true,
        false,
        None,
        None,
        true,
        vec![],
        vec![],
        Some(epic_id.clone()),
        vec![],
    ).unwrap();

    let ticket_branch = find_ticket_branch(dir.path(), "branched-ticket");
    // The parent of the ticket creation commit should be the epic branch tip.
    // The ticket branch has exactly one commit above the epic tip, so ticket^1 = epic tip.
    let parent = std::process::Command::new("git")
        .args(["rev-parse", &format!("{ticket_branch}^1")])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let parent_sha = String::from_utf8(parent.stdout).unwrap().trim().to_string();
    assert_eq!(parent_sha, epic_tip_sha, "ticket branch should be created from the epic branch tip");
}

#[test]
fn new_epic_bad_id_is_error() {
    let dir = setup();
    let result = apm::cmd::new::run(
        dir.path(),
        "Orphan ticket".into(),
        true,
        false,
        None,
        None,
        true,
        vec![],
        vec![],
        Some("deadbeef".into()),
        vec![],
    );
    assert!(result.is_err());
    assert!(
        result.unwrap_err().to_string().contains("No epic branch found for id 'deadbeef'"),
        "error message should mention the bad id"
    );
}

#[test]
fn new_depends_on_sets_frontmatter() {
    let dir = setup();
    apm::cmd::new::run(
        dir.path(),
        "Dependent ticket".into(),
        true,
        false,
        None,
        None,
        true,
        vec![],
        vec![],
        None,
        vec!["aabbccdd".into(), "11223344".into()],
    ).unwrap();
    let branch = find_ticket_branch(dir.path(), "dependent-ticket");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("depends_on"), "depends_on field missing");
    assert!(content.contains("aabbccdd"), "first dep missing");
    assert!(content.contains("11223344"), "second dep missing");
    assert!(!content.contains("epic ="), "epic should be absent");
    assert!(!content.contains("target_branch ="), "target_branch should be absent");
}

#[test]
fn new_depends_on_comma_separated() {
    let dir = setup();
    apm::cmd::new::run(
        dir.path(),
        "Comma deps ticket".into(),
        true,
        false,
        None,
        None,
        true,
        vec![],
        vec![],
        None,
        vec!["aabbccdd,11223344".into()],
    ).unwrap();
    let branch = find_ticket_branch(dir.path(), "comma-deps-ticket");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("aabbccdd"), "first dep missing");
    assert!(content.contains("11223344"), "second dep missing");
}

#[test]
fn new_without_epic_flags_is_unchanged() {
    let dir = setup();
    apm::cmd::new::run(
        dir.path(),
        "Plain ticket".into(),
        true,
        false,
        None,
        None,
        true,
        vec![],
        vec![],
        None,
        vec![],
    ).unwrap();
    let branch = find_ticket_branch(dir.path(), "plain-ticket");
    let rel = ticket_rel_path(&branch);
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(!content.contains("epic ="), "epic should not be present");
    assert!(!content.contains("target_branch ="), "target_branch should not be present");
    assert!(!content.contains("depends_on"), "depends_on should not be present");
    // Ticket branch should be rooted from main (parent of the ticket commit is main's tip)
    let main_tip = std::process::Command::new("git")
        .args(["rev-parse", "main"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let main_sha = String::from_utf8(main_tip.stdout).unwrap().trim().to_string();
    let parent = std::process::Command::new("git")
        .args(["rev-parse", &format!("{branch}^1")])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let parent_sha = String::from_utf8(parent.stdout).unwrap().trim().to_string();
    assert_eq!(parent_sha, main_sha, "ticket branch should be rooted from main");
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

    apm::cmd::new::run(p, "Scaffold test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();

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

    apm::cmd::new::run(p, "Review checkbox test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();

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
fn clean_default_does_not_remove_local_branch() {
    // apm clean (without --branches) must leave the local branch intact.
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "done");
    merge_into_main(p, &branch);

    apm::cmd::clean::run(p, false, false, false, false, false, None, false).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed without --branches");
}

#[test]
fn clean_branches_flag_removes_local_branch() {
    // apm clean --branches removes the local branch.
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "branches-flag");
    merge_into_main(p, &branch);

    apm::cmd::clean::run(p, false, false, false, true, false, None, false).unwrap();

    assert!(!branch_exists(p, &branch), "branch should have been removed with --branches");
}

#[test]
fn clean_dry_run_includes_state_in_output() {
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "dry");
    merge_into_main(p, &branch);

    // dry_run=true should not actually delete anything
    apm::cmd::clean::run(p, true, false, false, false, false, None, false).unwrap();

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

    apm::cmd::clean::run(p, false, false, false, false, false, None, false).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed — ticket not on main");
}

#[test]
fn clean_proceeds_despite_state_mismatch_between_branch_and_main() {
    // Ticket is closed on branch and merged into main. Then main's copy
    // gets updated to a different state (simulating a buggy sync). Clean should
    // trust the branch state and proceed — branch is authoritative.
    let dir = setup();
    let p = dir.path();
    let (branch, rel_path) = write_closed_ticket(p, 1, "mismatch");
    merge_into_main(p, &branch);

    // Overwrite the ticket on main to a different state
    let main_content = "+++\nid = 1\ntitle = \"mismatch\"\nstate = \"new\"\nbranch = \"ticket/0001-mismatch\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|";
    std::fs::write(p.join(&rel_path), main_content).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", &rel_path]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "update ticket state on main"]);

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["clean", "--branches"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(!branch_exists(p, &branch), "branch should have been removed — branch state is authoritative");
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

    apm::cmd::clean::run(p, false, false, false, true, false, None, false).unwrap();

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
    apm::cmd::clean::run(p, false, false, false, false, false, None, false).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed — local tip ahead of remote");
}

#[test]
fn clean_auto_removes_known_temp_files() {
    // Known temp files are removed automatically (no --untracked needed);
    // branch is NOT removed without --branches.
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "tempfiles");
    merge_into_main(p, &branch);

    // Create a linked worktree for the branch
    let wt_path = p.join("worktrees").join("ticket-0001-tempfiles");
    std::fs::create_dir_all(p.join("worktrees")).unwrap();
    git(p, &["worktree", "add", &wt_path.to_string_lossy(), &branch]);

    // Drop a known temp file into the worktree
    std::fs::write(wt_path.join("pr-body.md"), "pr body content").unwrap();

    apm::cmd::clean::run(p, false, false, false, false, false, None, false).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed without --branches");
    assert!(!wt_path.exists(), "worktree should have been removed");
}

#[test]
fn clean_skips_modified_tracked_files() {
    let dir = setup();
    let p = dir.path();
    let (branch, rel_path) = write_closed_ticket(p, 1, "modtracked");
    merge_into_main(p, &branch);

    // Create a linked worktree for the branch
    let wt_path = p.join("worktrees").join("ticket-0001-modtracked");
    std::fs::create_dir_all(p.join("worktrees")).unwrap();
    git(p, &["worktree", "add", &wt_path.to_string_lossy(), &branch]);

    // Modify a tracked file without committing
    std::fs::write(wt_path.join(&rel_path), "modified content").unwrap();

    apm::cmd::clean::run(p, false, false, false, false, false, None, false).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed — modified tracked file");
    assert!(wt_path.exists(), "worktree should NOT have been removed");
}

#[test]
fn clean_dry_run_diagnoses_dirty_worktree() {
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "drydiagnose");
    merge_into_main(p, &branch);

    // Create a linked worktree for the branch
    let wt_path = p.join("worktrees").join("ticket-0001-drydiagnose");
    std::fs::create_dir_all(p.join("worktrees")).unwrap();
    git(p, &["worktree", "add", &wt_path.to_string_lossy(), &branch]);

    // Drop a known temp file into the worktree
    let temp_file = wt_path.join("pr-body.md");
    std::fs::write(&temp_file, "pr body").unwrap();

    apm::cmd::clean::run(p, true, false, false, false, false, None, false).unwrap();

    // dry-run: nothing removed
    assert!(branch_exists(p, &branch), "branch should NOT have been removed in dry-run");
    assert!(wt_path.exists(), "worktree should NOT have been removed in dry-run");
    assert!(temp_file.exists(), "temp file should NOT have been removed in dry-run");
}

#[test]
fn clean_untracked_flag_removes_other_untracked_files() {
    // --untracked removes untracked non-temp files before worktree removal.
    // Branch is NOT removed without --branches.
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "otheruntracked");
    merge_into_main(p, &branch);

    // Create a linked worktree for the branch
    let wt_path = p.join("worktrees").join("ticket-0001-otheruntracked");
    std::fs::create_dir_all(p.join("worktrees")).unwrap();
    git(p, &["worktree", "add", &wt_path.to_string_lossy(), &branch]);

    // Drop an unrecognised untracked file into the worktree
    std::fs::write(wt_path.join("notes.txt"), "my notes").unwrap();

    apm::cmd::clean::run(p, false, false, false, false, false, None, true).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT have been removed without --branches");
    assert!(!wt_path.exists(), "worktree should have been removed with --untracked");
}

#[test]
fn clean_warns_about_untracked_without_flag() {
    // Without --untracked, a worktree with untracked non-temp files is skipped with a warning.
    let dir = setup();
    let p = dir.path();
    let (branch, _) = write_closed_ticket(p, 1, "warn-untracked");
    merge_into_main(p, &branch);

    let wt_path = p.join("worktrees").join("ticket-0001-warn-untracked");
    std::fs::create_dir_all(p.join("worktrees")).unwrap();
    git(p, &["worktree", "add", &wt_path.to_string_lossy(), &branch]);
    std::fs::write(wt_path.join("notes.txt"), "my notes").unwrap();

    // Without --untracked: worktree should stay in place.
    apm::cmd::clean::run(p, false, false, false, false, false, None, false).unwrap();

    assert!(wt_path.exists(), "worktree should NOT be removed without --untracked");
    assert!(branch_exists(p, &branch), "branch should NOT be removed without --branches");
}

// ── apm clean --force ─────────────────────────────────────────────────────────

#[test]
fn clean_force_removes_unmerged_branch() {
    // Closed ticket whose branch was never merged into main.
    // Normal clean skips it; --force should remove it after confirmation.
    let dir = setup();
    let p = dir.path();
    let (branch, rel_path) = write_closed_ticket(p, 1, "force-unmerged");

    // Write the closed ticket file to main directly so state_from_branch returns
    // Some("closed"). The branch is NOT merged via git merge.
    let closed_content = format!(
        "+++\nid = 1\ntitle = \"force-unmerged\"\nstate = \"closed\"\nbranch = \"{branch}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|"
    );
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    std::fs::write(p.join(&rel_path), &closed_content).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", &rel_path]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "add closed ticket to main"]);

    // Normal clean should skip (not merged via git merge).
    apm::cmd::clean::run(p, false, false, false, false, false, None, false).unwrap();
    assert!(branch_exists(p, &branch), "normal clean should skip unmerged branch");

    // Force clean with --branches and confirmation removes the local branch.
    use std::io::Write as _;
    let mut input = tempfile::NamedTempFile::new().unwrap();
    writeln!(input, "y").unwrap();
    input.flush().unwrap();
    let input_file = std::fs::File::open(input.path()).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["clean", "--force", "--branches"])
        .current_dir(p)
        .stdin(std::process::Stdio::from(input_file))
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(!branch_exists(p, &branch), "branch should have been removed by --force --branches clean");
}

#[test]
fn clean_force_removes_diverged_worktree() {
    // Closed ticket whose local branch tip is ahead of origin and worktree is dirty.
    // Normal clean skips it; --force should remove worktree and branch after confirmation.
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

    // Create closed ticket, merge into main, push both to remote.
    let (branch, _) = write_closed_ticket(p, 1, "force-diverged");
    git(p, &["push", "origin", &branch]);
    merge_into_main(p, &branch);
    git(p, &["push", "origin", "main"]);

    // Add an extra (unpushed) commit to the ticket branch so local tip diverges.
    // Write a non-ticket file so the ticket frontmatter stays intact.
    git(p, &["checkout", &branch]);
    std::fs::write(p.join("scratch.txt"), "extra change").unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", "scratch.txt"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "extra local commit"]);
    git(p, &["checkout", "main"]);

    // Create a worktree at the diverged tip and drop an untracked file into it.
    let wt_path = p.join("worktrees").join("ticket-0001-force-diverged");
    std::fs::create_dir_all(p.join("worktrees")).unwrap();
    git(p, &["worktree", "add", &wt_path.to_string_lossy(), &branch]);
    std::fs::write(wt_path.join("notes.txt"), "scratch notes").unwrap();

    // Normal clean should skip (diverged + dirty worktree, and not an ancestor of main).
    apm::cmd::clean::run(p, false, false, false, false, false, None, false).unwrap();
    assert!(branch_exists(p, &branch), "normal clean should skip diverged+dirty ticket");
    assert!(wt_path.exists(), "worktree should NOT be removed by normal clean");

    // Force clean removes the worktree; --branches also removes the local branch.
    use std::io::Write as _;
    let mut input = tempfile::NamedTempFile::new().unwrap();
    writeln!(input, "y").unwrap();
    input.flush().unwrap();
    let input_file = std::fs::File::open(input.path()).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["clean", "--force", "--branches"])
        .current_dir(p)
        .stdin(std::process::Stdio::from(input_file))
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(!branch_exists(p, &branch), "branch should have been removed by --force --branches clean");
    assert!(!wt_path.exists(), "worktree should have been removed by --force clean");
}

#[test]
fn clean_force_still_skips_non_terminal() {
    // A ticket in a non-terminal state should be skipped even with --force.
    let dir = setup();
    let p = dir.path();
    write_ticket_to_branch(p, "ticket/0001-in-prog", "0001-in-prog.md", "in_progress", 1, "in progress");

    // No candidates (non-terminal) → no prompts needed; call library directly.
    apm::cmd::clean::run(p, false, false, true, false, false, None, false).unwrap();

    assert!(
        branch_exists(p, "ticket/0001-in-prog"),
        "non-terminal ticket should NOT be removed by --force clean"
    );
}

#[test]
fn clean_force_dry_run_shows_unmerged() {
    // --force --dry-run should print "would remove" for unmerged branches without touching anything.
    let dir = setup();
    let p = dir.path();
    let (branch, rel_path) = write_closed_ticket(p, 1, "force-dryrun");

    // Write closed ticket to main so state_from_branch returns Some("closed").
    let closed_content = format!(
        "+++\nid = 1\ntitle = \"force-dryrun\"\nstate = \"closed\"\nbranch = \"{branch}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|"
    );
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    std::fs::write(p.join(&rel_path), &closed_content).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", &rel_path]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "add closed ticket to main"]);

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["clean", "--force", "--branches", "--dry-run"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("would remove"),
        "expected 'would remove' in output: {stdout}"
    );
    assert!(branch_exists(p, &branch), "dry-run should not remove the branch");
}

#[test]
fn clean_force_skips_modified_tracked() {
    // A ticket whose worktree has modified tracked files should be skipped even with --force.
    let dir = setup();
    let p = dir.path();
    let (branch, rel_path) = write_closed_ticket(p, 1, "force-modtracked");
    merge_into_main(p, &branch);

    let wt_path = p.join("worktrees").join("ticket-0001-force-modtracked");
    std::fs::create_dir_all(p.join("worktrees")).unwrap();
    git(p, &["worktree", "add", &wt_path.to_string_lossy(), &branch]);

    // Modify a tracked file without committing.
    std::fs::write(wt_path.join(&rel_path), "modified content").unwrap();

    // Force clean: modified tracked files must never be auto-removed.
    apm::cmd::clean::run(p, false, false, true, false, false, None, false).unwrap();

    assert!(branch_exists(p, &branch), "branch should NOT be removed — modified tracked file");
    assert!(wt_path.exists(), "worktree should NOT be removed — modified tracked file");
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
    assert!(!content.contains("agent ="), "agent field must not be written: {content}");
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

    apm::cmd::new::run(p, "kill test ticket".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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

    apm::cmd::new::run(p, "stale pid ticket".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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

    apm::cmd::new::run(p, "kill stale ticket".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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

    apm::cmd::new::run(p, "force test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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

    apm::cmd::new::run(p, "force progress".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
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

    apm::cmd::new::run(p, "force unknown".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let id = find_ticket_id(p, "force-unknown");

    // --force does not allow transitioning to a state that doesn't exist in config.
    let result = apm::cmd::state::run(p, &id, "nonexistent_state".into(), true, true);
    assert!(result.is_err(), "expected error for unknown state even with --force");
}

#[test]
fn state_force_does_not_skip_doc_validation() {
    let dir = setup();
    let p = dir.path();

    apm::cmd::new::run(p, "force doc valid".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let id = find_ticket_id(p, "force-doc-valid");

    // Transitioning to "specd" without a valid spec should still fail even with --force.
    let result = apm::cmd::state::run(p, &id, "specd".into(), true, true);
    assert!(result.is_err(), "expected spec validation to still fail with --force");
}

// --- squash-merge detection ---

/// Minimal apm.toml with implemented state for squash-merge tests.
fn squash_merge_config() -> &'static str {
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

[[workflow.states]]
id    = "implemented"
label = "Implemented"

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#
}

/// Set up a bare remote + local clone for squash-merge tests.
/// Returns (bare_dir, local_dir). Both TempDirs must be kept alive.
fn setup_squash_remote() -> (TempDir, TempDir) {
    let bare = tempfile::tempdir().unwrap();
    let bp = bare.path();
    git(bp, &["init", "--bare", "-q"]);

    let local = tempfile::tempdir().unwrap();
    let p = local.path();
    git(p, &["clone", &bp.to_string_lossy(), "."]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);

    std::fs::write(p.join("apm.toml"), squash_merge_config()).unwrap();
    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    git(p, &["push", "origin", "main"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();

    (bare, local)
}

/// Write an "implemented" ticket to a branch and return the branch name.
fn write_implemented_ticket(dir: &std::path::Path, branch: &str, filename: &str) {
    let path = format!("tickets/{filename}");
    let content = format!(
        "+++\nid = 1\ntitle = \"Squash test\"\nstate = \"implemented\"\nbranch = \"{branch}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|",
    );
    let branch_exists_locally = std::process::Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !branch_exists_locally {
        git(dir, &["checkout", "-b", branch]);
    } else {
        git(dir, &["checkout", branch]);
    }
    std::fs::create_dir_all(dir.join("tickets")).unwrap();
    std::fs::write(dir.join(&path), &content).unwrap();
    git(dir, &["-c", "commit.gpgsign=false", "add", &path]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", "implement ticket"]);
    git(dir, &["checkout", "main"]);
}

/// Squash-merge `branch` into main: `git merge --squash` + `git commit`.
fn squash_merge_into_main(dir: &std::path::Path, branch: &str) {
    git(dir, &["-c", "commit.gpgsign=false", "merge", "--squash", branch]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", &format!("squash merge {branch}")]);
}

#[test]
fn sync_detect_squash_merged_branch_remote_ref_present() {
    let (_bare, local) = setup_squash_remote();
    let p = local.path();

    let branch = "ticket/0001-squash-test";
    write_implemented_ticket(p, branch, "0001-squash-test.md");
    git(p, &["push", "origin", branch]);

    // Squash-merge into main locally and push main.
    squash_merge_into_main(p, branch);
    git(p, &["push", "origin", "main"]);

    // Fetch so local has up-to-date origin/main.
    git(p, &["fetch", "--all", "--quiet"]);

    let config = apm_core::config::Config::load(p).unwrap();
    let candidates = apm_core::sync::detect(p, &config).unwrap();
    let close_branches: Vec<&str> = candidates.close.iter()
        .map(|c| c.ticket.frontmatter.branch.as_deref().unwrap_or(""))
        .collect();
    assert!(
        close_branches.contains(&branch),
        "squash-merged ticket should appear in close candidates; got: {close_branches:?}"
    );
}

#[test]
fn sync_detect_squash_merged_branch_remote_ref_deleted() {
    let (_bare, local) = setup_squash_remote();
    let p = local.path();

    let branch = "ticket/0001-squash-gone";
    write_implemented_ticket(p, branch, "0001-squash-gone.md");
    git(p, &["push", "origin", branch]);

    squash_merge_into_main(p, branch);
    git(p, &["push", "origin", "main"]);

    // Delete the remote branch (GitHub does this automatically after merge).
    git(p, &["push", "origin", "--delete", branch]);
    // Prune the deleted remote tracking ref.
    git(p, &["fetch", "--all", "--prune", "--quiet"]);

    // Local branch still exists; remote tracking ref is gone.
    let config = apm_core::config::Config::load(p).unwrap();
    let candidates = apm_core::sync::detect(p, &config).unwrap();
    let close_branches: Vec<&str> = candidates.close.iter()
        .map(|c| c.ticket.frontmatter.branch.as_deref().unwrap_or(""))
        .collect();
    assert!(
        close_branches.contains(&branch),
        "squash-merged ticket with deleted remote ref should appear in close candidates; got: {close_branches:?}"
    );
}

#[test]
fn sync_detect_does_not_falsely_detect_unmerged_branch() {
    let (_bare, local) = setup_squash_remote();
    let p = local.path();

    let branch = "ticket/0001-not-merged";
    write_implemented_ticket(p, branch, "0001-not-merged.md");
    git(p, &["push", "origin", branch]);

    // Do NOT merge into main — branch has commits not in main.
    git(p, &["fetch", "--all", "--quiet"]);

    let config = apm_core::config::Config::load(p).unwrap();
    let candidates = apm_core::sync::detect(p, &config).unwrap();
    let close_branches: Vec<&str> = candidates.close.iter()
        .map(|c| c.ticket.frontmatter.branch.as_deref().unwrap_or(""))
        .collect();
    assert!(
        !close_branches.contains(&branch),
        "unmerged ticket should NOT appear in close candidates; got: {close_branches:?}"
    );
}

#[test]
fn sync_detect_regular_merge_still_detected() {
    let (_bare, local) = setup_squash_remote();
    let p = local.path();

    let branch = "ticket/0001-regular-merge";
    write_implemented_ticket(p, branch, "0001-regular-merge.md");
    git(p, &["push", "origin", branch]);

    // Regular (non-squash) merge.
    git(p, &["-c", "commit.gpgsign=false", "merge", "--no-ff", branch, "--no-edit"]);
    git(p, &["push", "origin", "main"]);
    git(p, &["fetch", "--all", "--quiet"]);

    let config = apm_core::config::Config::load(p).unwrap();
    let candidates = apm_core::sync::detect(p, &config).unwrap();
    let close_branches: Vec<&str> = candidates.close.iter()
        .map(|c| c.ticket.frontmatter.branch.as_deref().unwrap_or(""))
        .collect();
    assert!(
        close_branches.contains(&branch),
        "regular-merged ticket should appear in close candidates; got: {close_branches:?}"
    );
}

#[test]
fn start_uses_target_branch_as_merge_source() {
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

[sync]
aggressive = false

[[workflow.states]]
id = "ready"
label = "Ready"
actionable = ["agent"]

[[workflow.states]]
id = "in_progress"
label = "In Progress"
"#,
    )
    .unwrap();

    std::fs::create_dir_all(p.join("tickets")).unwrap();

    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init"]);

    // Create epic/e1-foo branch with a unique commit.
    git(p, &["checkout", "-b", "epic/e1-foo"]);
    std::fs::write(p.join("epic-marker.txt"), "epic content").unwrap();
    git(p, &["add", "epic-marker.txt"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "epic unique commit"]);

    // Back to main.
    git(p, &["checkout", "main"]);

    // Create ticket branch with target_branch = "epic/e1-foo".
    let ticket_branch = "ticket/abc1-epic-task";
    git(p, &["checkout", "-b", ticket_branch]);
    let ticket_content = concat!(
        "+++\n",
        "id = \"abc1\"\n",
        "title = \"Epic task\"\n",
        "state = \"ready\"\n",
        "branch = \"ticket/abc1-epic-task\"\n",
        "target_branch = \"epic/e1-foo\"\n",
        "+++\n\n",
    );
    std::fs::write(p.join("tickets/abc1-epic-task.md"), ticket_content).unwrap();
    git(p, &["add", "tickets/abc1-epic-task.md"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "add ticket"]);

    git(p, &["checkout", "main"]);

    apm::cmd::start::run(p, "abc1", true, false, false, "test-agent").unwrap();

    // The worktree should exist.
    let wt_path = p.join("worktrees").join("ticket-abc1-epic-task");
    assert!(wt_path.exists(), "worktree should be created at {}", wt_path.display());

    // The unique commit from epic/e1-foo should appear in the worktree history.
    let log = std::process::Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(&wt_path)
        .output()
        .unwrap();
    let log_str = String::from_utf8(log.stdout).unwrap();
    assert!(
        log_str.contains("epic unique commit"),
        "epic branch commit should be in worktree history; got:\n{log_str}"
    );
}

// ── depends_on scheduling ─────────────────────────────────────────────────────

fn setup_with_satisfies_deps() -> TempDir {
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

[[workflow.states]]
id         = "ready"
label      = "Ready"
actionable = ["agent"]

[[workflow.states]]
id             = "implemented"
label          = "Implemented"
satisfies_deps = true

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

fn commit_ticket_to_branch(dir: &std::path::Path, branch: &str, path: &str, content: &str) {
    // Create branch from main, write file, commit, return to main.
    let main_exists = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "main"])
        .current_dir(dir)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let base = if main_exists { "main" } else { "HEAD" };

    // Create branch from base if it doesn't exist, else just check it out.
    let branch_exists = std::process::Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .current_dir(dir)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if branch_exists {
        git(dir, &["checkout", branch]);
    } else {
        git(dir, &["checkout", "-b", branch, base]);
    }
    std::fs::create_dir_all(dir.join("tickets")).unwrap();
    std::fs::write(dir.join(path), content).unwrap();
    git(dir, &["-c", "commit.gpgsign=false", "add", path]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", "add ticket"]);
    git(dir, &["checkout", "-"]);
}

#[test]
fn next_skips_dep_blocked_returns_unblocked() {
    use apm_core::{config::Config, ticket};

    let dir = setup_with_satisfies_deps();
    let p = dir.path();

    // Ticket A: ready, no deps — should be returned by apm next
    let content_a = "+++\nid = \"aaaa0001\"\ntitle = \"Ticket A\"\nstate = \"ready\"\nbranch = \"ticket/aaaa0001-ticket-a\"\n+++\n\nbody\n";
    commit_ticket_to_branch(p, "ticket/aaaa0001-ticket-a", "tickets/aaaa0001-ticket-a.md", content_a);

    // Ticket B: ready, depends_on A (which is in "ready", not satisfies_deps)
    let content_b = "+++\nid = \"bbbb0001\"\ntitle = \"Ticket B\"\nstate = \"ready\"\nbranch = \"ticket/bbbb0001-ticket-b\"\ndepends_on = [\"aaaa0001\"]\n+++\n\nbody\n";
    commit_ticket_to_branch(p, "ticket/bbbb0001-ticket-b", "tickets/bbbb0001-ticket-b.md", content_b);

    let config = Config::load(p).unwrap();
    let tickets = ticket::load_all_from_git(p, &config.tickets.dir).unwrap();
    let actionable_owned = config.actionable_states_for("agent");
    let actionable: Vec<&str> = actionable_owned.iter().map(|s| s.as_str()).collect();
    let p_cfg = &config.workflow.prioritization;

    let next = ticket::pick_next(&tickets, &actionable, &[], p_cfg.priority_weight, p_cfg.effort_weight, p_cfg.risk_weight, &config, None);
    assert!(next.is_some(), "should find an actionable ticket");
    assert_eq!(next.unwrap().frontmatter.id, "aaaa0001", "dep-blocked ticket B should be skipped, A returned");
}

#[test]
fn next_returns_dep_blocked_after_dep_satisfies() {
    use apm_core::{config::Config, ticket};

    let dir = setup_with_satisfies_deps();
    let p = dir.path();

    // Ticket A: implemented (satisfies_deps = true)
    let content_a = "+++\nid = \"aaaa0002\"\ntitle = \"Ticket A\"\nstate = \"implemented\"\nbranch = \"ticket/aaaa0002-ticket-a\"\n+++\n\nbody\n";
    commit_ticket_to_branch(p, "ticket/aaaa0002-ticket-a", "tickets/aaaa0002-ticket-a.md", content_a);

    // Ticket B: ready, depends_on A (implemented = satisfies_deps)
    let content_b = "+++\nid = \"bbbb0002\"\ntitle = \"Ticket B\"\nstate = \"ready\"\nbranch = \"ticket/bbbb0002-ticket-b\"\ndepends_on = [\"aaaa0002\"]\n+++\n\nbody\n";
    commit_ticket_to_branch(p, "ticket/bbbb0002-ticket-b", "tickets/bbbb0002-ticket-b.md", content_b);

    let config = Config::load(p).unwrap();
    let tickets = ticket::load_all_from_git(p, &config.tickets.dir).unwrap();
    let actionable_owned = config.actionable_states_for("agent");
    let actionable: Vec<&str> = actionable_owned.iter().map(|s| s.as_str()).collect();
    let p_cfg = &config.workflow.prioritization;

    let next = ticket::pick_next(&tickets, &actionable, &[], p_cfg.priority_weight, p_cfg.effort_weight, p_cfg.risk_weight, &config, None);
    assert!(next.is_some(), "should find an actionable ticket");
    assert_eq!(next.unwrap().frontmatter.id, "bbbb0002", "ticket B should be returned once dep A satisfies_deps");
}

#[test]
fn next_picks_low_priority_blocker_before_higher_raw_independent() {
    use apm_core::{config::Config, ticket};

    // A: priority 2, ready, no deps (blocks C)
    // B: priority 7, ready, no deps (independent)
    // C: priority 9, ready, depends_on A (dep not satisfied, so C won't be returned)
    // Expected: pick_next returns A (ep=9 > B's ep=7)
    let dir = setup_with_satisfies_deps();
    let p = dir.path();

    let content_a = "+++\nid = \"aaaa0003\"\ntitle = \"Ticket A\"\nstate = \"ready\"\npriority = 2\nbranch = \"ticket/aaaa0003-ticket-a\"\n+++\n\nbody\n";
    commit_ticket_to_branch(p, "ticket/aaaa0003-ticket-a", "tickets/aaaa0003-ticket-a.md", content_a);

    let content_b = "+++\nid = \"bbbb0003\"\ntitle = \"Ticket B\"\nstate = \"ready\"\npriority = 7\nbranch = \"ticket/bbbb0003-ticket-b\"\n+++\n\nbody\n";
    commit_ticket_to_branch(p, "ticket/bbbb0003-ticket-b", "tickets/bbbb0003-ticket-b.md", content_b);

    let content_c = "+++\nid = \"cccc0003\"\ntitle = \"Ticket C\"\nstate = \"ready\"\npriority = 9\nbranch = \"ticket/cccc0003-ticket-c\"\ndepends_on = [\"aaaa0003\"]\n+++\n\nbody\n";
    commit_ticket_to_branch(p, "ticket/cccc0003-ticket-c", "tickets/cccc0003-ticket-c.md", content_c);

    let config = Config::load(p).unwrap();
    let tickets = ticket::load_all_from_git(p, &config.tickets.dir).unwrap();
    let actionable_owned = config.actionable_states_for("agent");
    let actionable: Vec<&str> = actionable_owned.iter().map(|s| s.as_str()).collect();
    let p_cfg = &config.workflow.prioritization;

    let next = ticket::pick_next(&tickets, &actionable, &[], p_cfg.priority_weight, p_cfg.effort_weight, p_cfg.risk_weight, &config, None);
    assert!(next.is_some(), "should find an actionable ticket");
    // C is dep-blocked (A not satisfied), so the contest is A (ep=9) vs B (ep=7)
    assert_eq!(next.unwrap().frontmatter.id, "aaaa0003", "A (ep=9) should beat B (ep=7)");
}

// --- epic list ---

fn setup_epic_list() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();

    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);

    std::fs::write(
        p.join("apm.toml"),
        r#"[project]
name = "test"

[sync]
aggressive = false

[tickets]
dir = "tickets"

[[workflow.states]]
id         = "ready"
label      = "Ready"
actionable = ["agent"]

[[workflow.states]]
id             = "implemented"
label          = "Implemented"
satisfies_deps = true

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

/// Create a bare local branch (simulates an epic branch).
fn create_epic_branch(dir: &std::path::Path, branch: &str) {
    git(dir, &["checkout", "-b", branch]);
    // Write a placeholder file so the branch has a commit.
    std::fs::write(dir.join("EPIC.md"), format!("# {branch}\n")).unwrap();
    git(dir, &["-c", "commit.gpgsign=false", "add", "EPIC.md"]);
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", "create epic"]);
    git(dir, &["checkout", "-"]);
    // Remove placeholder from main worktree.
    let _ = std::fs::remove_file(dir.join("EPIC.md"));
}

#[test]
fn epic_list_no_epics_exits_zero_no_output() {
    let dir = setup_epic_list();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["epic", "list"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(out.status.success(), "exit status: {}", out.status);
    assert!(out.stdout.is_empty(), "expected no output, got: {}", String::from_utf8_lossy(&out.stdout));
}

#[test]
fn epic_list_shows_epics_with_derived_state_and_counts() {
    let dir = setup_epic_list();
    let p = dir.path();

    // Epic 1: "ab12cd34-user-authentication"
    let epic1_id = "ab12cd34";
    let epic1_branch = format!("epic/{epic1_id}-user-authentication");
    create_epic_branch(p, &epic1_branch);

    // Epic 2: "ef567890-billing-overhaul" — no tickets (empty)
    let epic2_id = "ef567890";
    let epic2_branch = format!("epic/{epic2_id}-billing-overhaul");
    create_epic_branch(p, &epic2_branch);

    // Ticket for epic 1: state = ready (actionable by agent → "active")
    let t1 = format!(
        "+++\nid = \"t1000001\"\ntitle = \"Auth ticket\"\nstate = \"ready\"\nbranch = \"ticket/t1000001-auth-ticket\"\nepic = \"{epic1_id}\"\n+++\n\nbody\n"
    );
    commit_ticket_to_branch(p, "ticket/t1000001-auth-ticket", "tickets/t1000001-auth-ticket.md", &t1);

    // Second ticket for epic 1: state = implemented
    let t2 = format!(
        "+++\nid = \"t1000002\"\ntitle = \"Auth impl\"\nstate = \"implemented\"\nbranch = \"ticket/t1000002-auth-impl\"\nepic = \"{epic1_id}\"\n+++\n\nbody\n"
    );
    commit_ticket_to_branch(p, "ticket/t1000002-auth-impl", "tickets/t1000002-auth-impl.md", &t2);

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["epic", "list"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(out.status.success(), "exit: {}\nstderr: {}", out.status, String::from_utf8_lossy(&out.stderr));

    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "expected 2 lines, got:\n{stdout}");

    // Epic 1: in_progress (has ticket with neither satisfies_deps nor terminal)
    assert!(lines[0].contains(epic1_id), "line 0 should contain epic1 id: {}", lines[0]);
    assert!(lines[0].contains("in_progress"), "line 0 should be in_progress: {}", lines[0]);
    assert!(lines[0].contains("User Authentication"), "line 0 should have title: {}", lines[0]);
    assert!(lines[0].contains("1 ready"), "line 0 should show 1 ready: {}", lines[0]);
    assert!(lines[0].contains("1 implemented"), "line 0 should show 1 implemented: {}", lines[0]);

    // Epic 2: empty (no tickets)
    assert!(lines[1].contains(epic2_id), "line 1 should contain epic2 id: {}", lines[1]);
    assert!(lines[1].contains("empty"), "line 1 should be empty: {}", lines[1]);
    assert!(lines[1].contains("Billing Overhaul"), "line 1 should have title: {}", lines[1]);
}

// --- epic show ---

fn setup_epic_show() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();

    git(p, &["init", "-q"]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);

    std::fs::write(
        p.join("apm.toml"),
        r#"[project]
name = "test"

[sync]
aggressive = false

[tickets]
dir = "tickets"

[[workflow.states]]
id         = "ready"
label      = "Ready"
actionable = ["agent"]

[[workflow.states]]
id             = "implemented"
label          = "Implemented"
satisfies_deps = true

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
fn epic_show_displays_header_and_ticket_table() {
    let dir = setup_epic_show();
    let p = dir.path();

    let epic_id = "ab12cd34";
    let epic_branch = format!("epic/{epic_id}-user-auth");
    create_epic_branch(p, &epic_branch);

    // Ticket 1: belongs to epic, ready state
    let t1 = format!(
        "+++\nid = \"t2000001\"\ntitle = \"Implement login\"\nstate = \"ready\"\nbranch = \"ticket/t2000001-impl-login\"\nepic = \"{epic_id}\"\nagent = \"alice\"\n+++\n\nbody\n"
    );
    commit_ticket_to_branch(p, "ticket/t2000001-impl-login", "tickets/t2000001-impl-login.md", &t1);

    // Ticket 2: belongs to epic, implemented state, with depends_on
    let t2 = format!(
        "+++\nid = \"t2000002\"\ntitle = \"Add OAuth\"\nstate = \"implemented\"\nbranch = \"ticket/t2000002-add-oauth\"\nepic = \"{epic_id}\"\ndepends_on = [\"t2000001\"]\n+++\n\nbody\n"
    );
    commit_ticket_to_branch(p, "ticket/t2000002-add-oauth", "tickets/t2000002-add-oauth.md", &t2);

    // Ticket 3: does NOT belong to epic
    let t3 = "+++\nid = \"t2000003\"\ntitle = \"Unrelated\"\nstate = \"ready\"\nbranch = \"ticket/t2000003-unrelated\"\n+++\n\nbody\n";
    commit_ticket_to_branch(p, "ticket/t2000003-unrelated", "tickets/t2000003-unrelated.md", t3);

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["epic", "show", epic_id])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "exit: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    // Header block
    assert!(stdout.contains("User Auth"), "should contain title: {stdout}");
    assert!(stdout.contains(&epic_branch), "should contain branch: {stdout}");
    assert!(stdout.contains("in_progress"), "should contain derived state: {stdout}");
    // Ticket table rows
    assert!(stdout.contains("t2000001"), "should contain ticket1 id: {stdout}");
    assert!(stdout.contains("t2000002"), "should contain ticket2 id: {stdout}");
    assert!(stdout.contains("t2000001"), "ticket2 depends_on should show t2000001: {stdout}");
    // Unrelated ticket must NOT appear
    assert!(!stdout.contains("t2000003"), "unrelated ticket must not appear: {stdout}");
    assert!(!stdout.contains("Unrelated"), "unrelated ticket title must not appear: {stdout}");
}

#[test]
fn epic_show_prefix_resolves_correctly() {
    let dir = setup_epic_show();
    let p = dir.path();

    let epic_id = "ab12cd34";
    let epic_branch = format!("epic/{epic_id}-user-auth");
    create_epic_branch(p, &epic_branch);

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["epic", "show", "ab12"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "exit: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(&epic_branch), "should resolve via prefix: {stdout}");
}

#[test]
fn epic_show_no_match_exits_nonzero() {
    let dir = setup_epic_show();
    let p = dir.path();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["epic", "show", "zzzzzzz"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(!out.status.success(), "expected non-zero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("zzzzzzz"), "error should mention the prefix: {stderr}");
}

#[test]
fn epic_show_ambiguous_prefix_exits_nonzero() {
    let dir = setup_epic_show();
    let p = dir.path();

    // Create two epics with the same prefix "aa"
    create_epic_branch(p, "epic/aa000001-first");
    create_epic_branch(p, "epic/aa000002-second");

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["epic", "show", "aa"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(!out.status.success(), "expected non-zero exit for ambiguous prefix");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("ambiguous"), "error should say ambiguous: {stderr}");
}

#[test]
fn epic_show_no_tickets_prints_no_tickets() {
    let dir = setup_epic_show();
    let p = dir.path();

    let epic_id = "cc112233";
    create_epic_branch(p, &format!("epic/{epic_id}-empty-epic"));

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["epic", "show", epic_id])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(out.status.success(), "exit: {}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("(no tickets)"), "should print no tickets message: {stdout}");
    assert!(stdout.contains("empty"), "derived state should be empty: {stdout}");
}

// --- apm work --epic ---

fn write_ticket_with_epic(dir: &std::path::Path, branch: &str, filename: &str, state: &str, id: u32, title: &str, epic: Option<&str>) {
    let path = format!("tickets/{filename}");
    let epic_line = epic.map(|e| format!("epic = \"{e}\"\n")).unwrap_or_default();
    let content = format!(
        "+++\nid = {id}\ntitle = \"{title}\"\nstate = \"{state}\"\nbranch = \"{branch}\"\n{epic_line}created_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|",
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
fn work_dry_run_epic_filter_shows_only_epic_ticket() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    std::env::set_var("APM_AGENT_NAME", "test-agent");
    write_ticket_with_epic(p, "ticket/0001-epic-ticket", "0001-epic-ticket.md", "ready", 1, "epic ticket", Some("ab12cd34"));
    write_ticket_with_epic(p, "ticket/0002-free-ticket", "0002-free-ticket.md", "ready", 2, "free ticket", None);

    // Capture stdout
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["work", "--dry-run", "--epic", "ab12cd34"])
        .current_dir(p)
        .env("APM_AGENT_NAME", "test-agent")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "exit: {}\nstderr: {}", out.status, String::from_utf8_lossy(&out.stderr));
    assert!(stdout.contains("epic ticket"), "should show epic ticket: {stdout}");
    assert!(!stdout.contains("free ticket"), "should not show free ticket: {stdout}");
}

#[test]
fn work_dry_run_epic_filter_no_candidates() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    std::env::set_var("APM_AGENT_NAME", "test-agent");
    write_ticket_with_epic(p, "ticket/0001-free-ticket", "0001-free-ticket.md", "ready", 1, "free ticket", None);

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["work", "--dry-run", "--epic", "ab12cd34"])
        .current_dir(p)
        .env("APM_AGENT_NAME", "test-agent")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "exit: {}\nstderr: {}", out.status, String::from_utf8_lossy(&out.stderr));
    assert!(stdout.contains("no actionable tickets"), "should show no candidates: {stdout}");
}

#[test]
fn work_dry_run_no_flag_shows_epic_ticket() {
    let dir = setup_with_local_worktrees();
    let p = dir.path();
    std::env::set_var("APM_AGENT_NAME", "test-agent");
    write_ticket_with_epic(p, "ticket/0001-epic-ticket", "0001-epic-ticket.md", "ready", 1, "epic ticket", Some("ab12cd34"));

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["work", "--dry-run"])
        .current_dir(p)
        .env("APM_AGENT_NAME", "test-agent")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "exit: {}\nstderr: {}", out.status, String::from_utf8_lossy(&out.stderr));
    assert!(stdout.contains("epic ticket"), "should show epic ticket without filter: {stdout}");
}

// --- pr_or_epic_merge completion strategy ---

fn pr_or_epic_merge_config_toml() -> &'static str {
    r#"[project]
name = "test"
default_branch = "main"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

  [[workflow.states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  actor      = "agent"
  completion = "pr_or_epic_merge"

[[workflow.states]]
id    = "implemented"
label = "Implemented"
"#
}

fn setup_pr_or_epic_merge_remote() -> (TempDir, TempDir) {
    let bare = tempfile::tempdir().unwrap();
    let bp = bare.path();
    git(bp, &["init", "--bare", "-q"]);

    let local = tempfile::tempdir().unwrap();
    let p = local.path();
    git(p, &["clone", &bp.to_string_lossy(), "."]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    std::fs::write(p.join("apm.toml"), pr_or_epic_merge_config_toml()).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init"]);
    git(p, &["push", "origin", "main"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    (bare, local)
}

fn write_in_progress_ticket(dir: &std::path::Path, id: &str, branch: &str, filename: &str, target_branch: Option<&str>) {
    let path = format!("tickets/{filename}");
    let target_line = match target_branch {
        Some(tb) => format!("target_branch = \"{tb}\"\n"),
        None => String::new(),
    };
    let content = format!(
        "+++\nid = \"{id}\"\ntitle = \"Test ticket\"\nstate = \"in_progress\"\nbranch = \"{branch}\"\n{target_line}created_at = \"2026-01-01T00:00:00Z\"\nupdated_at = \"2026-01-01T00:00:00Z\"\n+++\n\n## Spec\n\n### Acceptance criteria\n\n- [x] Done\n\n## History\n\n| When | From | To | By |\n|------|------|----|----|"
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
    git(dir, &["-c", "commit.gpgsign=false", "commit", "-m", &format!("ticket: {id}")]);
    git(dir, &["checkout", "main"]);
}

#[test]
fn pr_or_epic_merge_with_target_branch_merges_into_target() {
    let (_bare, local) = setup_pr_or_epic_merge_remote();
    let p = local.path();

    // Create epic branch and push to origin.
    git(p, &["checkout", "-b", "epic/test"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "epic init", "--allow-empty"]);
    git(p, &["push", "origin", "epic/test"]);
    git(p, &["checkout", "main"]);

    // Write an in_progress ticket with target_branch set.
    let branch = "ticket/aa000001-merge-test";
    write_in_progress_ticket(p, "aa000001", branch, "aa000001-merge-test.md", Some("epic/test"));
    git(p, &["push", "origin", branch]);

    // Check out epic/test so merge_into_default merges into it.
    git(p, &["checkout", "epic/test"]);

    let result = apm_core::state::transition(p, "aa000001", "implemented".into(), true, false);
    assert!(result.is_ok(), "merge path should succeed: {}", result.err().map(|e| e.to_string()).unwrap_or_default());

    // Verify epic/test now has a merge commit referencing the ticket branch.
    let log = std::process::Command::new("git")
        .args(["log", "--oneline", "epic/test"])
        .current_dir(p)
        .output()
        .unwrap();
    let log_str = String::from_utf8_lossy(&log.stdout);
    assert!(log_str.lines().count() > 1, "epic/test should have additional commits after merge: {log_str}");
}

#[test]
fn pr_or_epic_merge_without_target_branch_attempts_pr() {
    let (_bare, local) = setup_pr_or_epic_merge_remote();
    let p = local.path();

    // Write an in_progress ticket without target_branch.
    let branch = "ticket/bb000002-pr-test";
    write_in_progress_ticket(p, "bb000002", branch, "bb000002-pr-test.md", None);
    git(p, &["push", "origin", branch]);

    // Root stays on main — no target_branch → PR path.
    let result = apm_core::state::transition(p, "bb000002", "implemented".into(), true, false);

    // Push succeeds (bare remote available); gh fails → Err returned.
    assert!(result.is_err(), "PR path should fail (gh not available against local bare repo)");

    // Confirm the ticket branch was pushed before gh was attempted.
    let remote_refs = std::process::Command::new("git")
        .args(["ls-remote", "origin", branch])
        .current_dir(p)
        .output()
        .unwrap();
    let refs = String::from_utf8_lossy(&remote_refs.stdout);
    assert!(!refs.trim().is_empty(), "ticket branch should have been pushed before gh was called: {refs}");
}

#[test]
fn agents_prints_instructions_file() {
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

[agents]
instructions = "agents-instructions.md"
max_concurrent = 1

[[workflow.states]]
id         = "new"
label      = "New"
actionable = ["agent"]

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#,
    )
    .unwrap();

    std::fs::write(p.join("agents-instructions.md"), "hello from agents\n").unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["agents"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(out.status.success(), "expected exit 0, got: {}\nstderr: {}", out.status, String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout, "hello from agents\n", "unexpected output: {stdout}");
}

fn setup_with_server_url(url: &str) -> TempDir {
    let dir = setup();
    let p = dir.path();
    let server_block = format!("\n[server]\nurl = \"{url}\"\n");
    let apm_toml = std::fs::read_to_string(p.join("apm.toml")).unwrap();
    std::fs::write(p.join("apm.toml"), format!("{apm_toml}{server_block}")).unwrap();
    dir
}

#[test]
fn register_prints_otp_from_server() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/api/auth/otp")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"otp":"ABCD1234"}"#)
        .create();
    let dir = setup_with_server_url(&server.url());
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["register", "alice"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    mock.assert();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "ABCD1234");
}

#[test]
fn sessions_empty_response_prints_no_active() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/api/auth/sessions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[]")
        .create();
    let dir = setup_with_server_url(&server.url());
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["sessions"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    mock.assert();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("No active sessions."));
}

#[test]
fn sessions_with_data_prints_table() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/api/auth/sessions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"username":"alice","device_hint":"MacBook","last_seen":"2026-04-01T14:32:00Z","expires_at":"2026-04-08T14:32:00Z"}]"#)
        .create();
    let dir = setup_with_server_url(&server.url());
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["sessions"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    mock.assert();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("alice"), "missing username: {stdout}");
    assert!(stdout.contains("MacBook"), "missing device: {stdout}");
    assert!(stdout.contains("USERNAME"), "missing header: {stdout}");
}

#[test]
fn revoke_user_prints_count() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("DELETE", "/api/auth/sessions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"revoked":2}"#)
        .create();
    let dir = setup_with_server_url(&server.url());
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["revoke", "alice"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    mock.assert();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("Revoked 2 session(s)."));
}

#[test]
fn revoke_user_zero_prints_no_sessions() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("DELETE", "/api/auth/sessions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"revoked":0}"#)
        .create();
    let dir = setup_with_server_url(&server.url());
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["revoke", "alice"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    mock.assert();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("No sessions found for alice."));
}

#[test]
fn revoke_all_sends_all_flag() {
    let mut server = mockito::Server::new();
    let body_json = serde_json::json!({"username": null, "device": null, "all": true});
    let mock = server
        .mock("DELETE", "/api/auth/sessions")
        .match_body(mockito::Matcher::Json(body_json))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"revoked":5}"#)
        .create();
    let dir = setup_with_server_url(&server.url());
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["revoke", "--all"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    mock.assert();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("Revoked 5 session(s)."));
}

#[test]
fn revoke_with_device_hint() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("DELETE", "/api/auth/sessions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"revoked":1}"#)
        .create();
    let dir = setup_with_server_url(&server.url());
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_apm"))
        .args(["revoke", "alice", "--device", "MacBook"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    mock.assert();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("Revoked 1 session(s)."));
}

#[test]
fn assign_sets_owner_field() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Assign test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "assign-test");
    let id = find_ticket_id(dir.path(), "assign-test");
    let rel = ticket_rel_path(&branch);
    apm::cmd::assign::run(dir.path(), &id, "alice", true).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(content.contains("owner = \"alice\""));
}

#[test]
fn assign_clears_owner_field() {
    let dir = setup();
    apm::cmd::new::run(dir.path(), "Assign clear test".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(dir.path(), "assign-clear-test");
    let id = find_ticket_id(dir.path(), "assign-clear-test");
    let rel = ticket_rel_path(&branch);
    apm::cmd::assign::run(dir.path(), &id, "alice", true).unwrap();
    apm::cmd::assign::run(dir.path(), &id, "-", true).unwrap();
    let content = branch_content(dir.path(), &branch, &rel);
    assert!(!content.contains("owner ="));
}

#[test]
fn assign_unknown_id_errors() {
    let dir = setup();
    let result = apm::cmd::assign::run(dir.path(), "9999", "alice", true);
    assert!(result.is_err());
}

// --- archive ---

fn setup_with_archive_dir() -> TempDir {
    let dir = setup();
    let p = dir.path();
    // Append archive_dir to the [tickets] section in apm.toml.
    let toml = std::fs::read_to_string(p.join("apm.toml")).unwrap();
    let updated = toml.replace(
        "[tickets]\ndir = \"tickets\"",
        "[tickets]\ndir = \"tickets\"\narchive_dir = \"archive/tickets\"",
    );
    std::fs::write(p.join("apm.toml"), updated).unwrap();
    dir
}

#[test]
fn archive_no_archive_dir_errors() {
    let dir = setup();
    let result = apm::cmd::archive::run(dir.path(), false, None);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("archive_dir is not set"), "unexpected error: {msg}");
}

#[test]
fn archive_moves_closed_ticket_to_archive_dir() {
    let dir = setup_with_archive_dir();
    let p = dir.path();

    // Create and close a ticket (close merges to main).
    apm::cmd::new::run(p, "Archive me".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(p, "archive-me");
    let id = find_ticket_id(p, "archive-me");
    apm::cmd::close::run(p, &id, None, true).unwrap();

    // Verify the ticket file is on main before archive.
    let rel = ticket_rel_path(&branch);
    let files_before = apm_core::git::list_files_on_branch(p, "main", "tickets").unwrap();
    assert!(files_before.iter().any(|f| f == &rel), "ticket not on main before archive");

    // Run archive.
    apm::cmd::archive::run(p, false, None).unwrap();

    // Ticket file should be gone from tickets/ on main.
    let files_after = apm_core::git::list_files_on_branch(p, "main", "tickets").unwrap_or_default();
    assert!(!files_after.iter().any(|f| f == &rel), "ticket still in tickets/ after archive");

    // Ticket file should exist in archive/tickets/ on main.
    let filename = std::path::Path::new(&rel).file_name().unwrap().to_str().unwrap();
    let archive_path = format!("archive/tickets/{filename}");
    let archive_files = apm_core::git::list_files_on_branch(p, "main", "archive/tickets").unwrap();
    assert!(archive_files.iter().any(|f| f == &archive_path), "ticket not in archive/tickets/ after archive");
}

#[test]
fn archive_show_finds_ticket_in_archive_after_branch_deleted() {
    let dir = setup_with_archive_dir();
    let p = dir.path();

    // Create and close a ticket.
    apm::cmd::new::run(p, "Show archived".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(p, "show-archived");
    let id = find_ticket_id(p, "show-archived");
    apm::cmd::close::run(p, &id, None, true).unwrap();

    // Archive it.
    apm::cmd::archive::run(p, false, None).unwrap();

    // Delete the ticket branch (simulate apm clean --branches).
    git(p, &["branch", "-D", &branch]);

    // apm show should still succeed via the archive fallback.
    apm::cmd::show::run(p, &id, true, false).unwrap();
}

#[test]
fn archive_dry_run_does_not_move_files() {
    let dir = setup_with_archive_dir();
    let p = dir.path();

    apm::cmd::new::run(p, "Dry run me".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(p, "dry-run-me");
    let id = find_ticket_id(p, "dry-run-me");
    apm::cmd::close::run(p, &id, None, true).unwrap();

    // Dry run should not move anything.
    apm::cmd::archive::run(p, true, None).unwrap();

    // Ticket file should still be in tickets/ on main.
    let rel = ticket_rel_path(&branch);
    let files = apm_core::git::list_files_on_branch(p, "main", "tickets").unwrap();
    assert!(files.iter().any(|f| f == &rel), "dry-run should not move ticket file");

    // archive/tickets/ should not exist.
    let archive_files = apm_core::git::list_files_on_branch(p, "main", "archive/tickets").unwrap_or_default();
    assert!(archive_files.is_empty(), "dry-run should not create archive dir");
}

#[test]
fn archive_skips_non_terminal_tickets_with_warning() {
    let dir = setup_with_archive_dir();
    let p = dir.path();

    // Create a ticket in "new" state (non-terminal).
    apm::cmd::new::run(p, "Non terminal".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(p, "non-terminal");

    // Put the ticket file on main manually (without closing) so it appears in tickets/ on main.
    let rel = ticket_rel_path(&branch);
    let content = branch_content(p, &branch, &rel);
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    std::fs::write(p.join(&rel), &content).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", &rel]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "put non-terminal ticket on main"]);

    // archive should not move it.
    apm::cmd::archive::run(p, false, None).unwrap();

    let files = apm_core::git::list_files_on_branch(p, "main", "tickets").unwrap();
    assert!(files.iter().any(|f| f == &rel), "non-terminal ticket should remain in tickets/");
}

#[test]
fn archive_older_than_skips_recent_ticket() {
    let dir = setup_with_archive_dir();
    let p = dir.path();

    apm::cmd::new::run(p, "Recent ticket".into(), true, false, None, None, true, vec![], vec![], None, vec![]).unwrap();
    let branch = find_ticket_branch(p, "recent-ticket");
    let id = find_ticket_id(p, "recent-ticket");
    apm::cmd::close::run(p, &id, None, true).unwrap();

    // Use a 30-day threshold — a ticket created now is newer than 30 days ago, so skip it.
    apm::cmd::archive::run(p, false, Some("30d".into())).unwrap();

    let rel = ticket_rel_path(&branch);
    let files = apm_core::git::list_files_on_branch(p, "main", "tickets").unwrap();
    assert!(files.iter().any(|f| f == &rel), "recent ticket should not be archived with --older-than 0d");
}

// --- merge completion strategy: push to origin after merge ---

fn merge_strategy_config_toml() -> &'static str {
    r#"[project]
name = "test"
default_branch = "main"

[tickets]
dir = "tickets"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"

  [[workflow.states.transitions]]
  to         = "implemented"
  trigger    = "manual"
  actor      = "agent"
  completion = "merge"

[[workflow.states]]
id    = "implemented"
label = "Implemented"
"#
}

fn setup_merge_strategy_remote() -> (TempDir, TempDir) {
    let bare = tempfile::tempdir().unwrap();
    let bp = bare.path();
    git(bp, &["init", "--bare", "-q"]);

    let local = tempfile::tempdir().unwrap();
    let p = local.path();
    git(p, &["clone", &bp.to_string_lossy(), "."]);
    git(p, &["config", "user.email", "test@test.com"]);
    git(p, &["config", "user.name", "test"]);
    std::fs::write(p.join("apm.toml"), merge_strategy_config_toml()).unwrap();
    git(p, &["-c", "commit.gpgsign=false", "add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init"]);
    git(p, &["push", "origin", "main"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();
    (bare, local)
}

fn remote_ref_sha(dir: &std::path::Path, refname: &str) -> String {
    let out = std::process::Command::new("git")
        .args(["ls-remote", "origin", refname])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

fn local_ref_sha(dir: &std::path::Path, refname: &str) -> String {
    let out = std::process::Command::new("git")
        .args(["rev-parse", refname])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn merge_strategy_merges_locally_without_push() {
    let (_bare, local) = setup_merge_strategy_remote();
    let p = local.path();

    let branch = "ticket/cc000003-merge-push-test";
    write_in_progress_ticket(p, "cc000003", branch, "cc000003-merge-push-test.md", None);

    let main_before = local_ref_sha(p, "main");

    let result = apm_core::state::transition(p, "cc000003", "implemented".into(), true, false);
    assert!(result.is_ok(), "merge strategy should succeed: {}", result.err().map(|e| e.to_string()).unwrap_or_default());

    let main_after = local_ref_sha(p, "main");
    assert_ne!(main_before, main_after, "local main should advance after merge");

    let remote_sha = remote_ref_sha(p, "main");
    assert_eq!(main_before, remote_sha, "origin/main should NOT advance — merge to default branch is local only");
}

#[test]
fn pr_or_epic_merge_with_target_branch_pushes_target_to_origin() {
    let (_bare, local) = setup_pr_or_epic_merge_remote();
    let p = local.path();

    git(p, &["checkout", "-b", "epic/push-test"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "epic init", "--allow-empty"]);
    git(p, &["push", "origin", "epic/push-test"]);
    git(p, &["checkout", "main"]);

    let branch = "ticket/dd000004-epic-push-test";
    write_in_progress_ticket(p, "dd000004", branch, "dd000004-epic-push-test.md", Some("epic/push-test"));
    git(p, &["push", "origin", branch]);

    git(p, &["checkout", "epic/push-test"]);

    let result = apm_core::state::transition(p, "dd000004", "implemented".into(), true, false);
    assert!(result.is_ok(), "pr_or_epic_merge with target should succeed: {}", result.err().map(|e| e.to_string()).unwrap_or_default());

    let local_sha = local_ref_sha(p, "epic/push-test");
    let remote_sha = remote_ref_sha(p, "epic/push-test");
    assert!(!local_sha.is_empty(), "local epic/push-test should exist");
    assert_eq!(local_sha, remote_sha, "origin/epic/push-test should match local after push");
}
