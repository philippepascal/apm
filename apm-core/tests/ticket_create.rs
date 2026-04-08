use apm_core::{config::Config, ticket};
use std::process::Command;
use tempfile::TempDir;

fn git(dir: &std::path::Path, args: &[&str]) {
    Command::new("git")
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

[[workflow.states.transitions]]
to      = "in_design"
trigger = "manual"

[[workflow.states]]
id    = "in_design"
label = "In Design"
"#,
    )
    .unwrap();

    // Initial commit so the repo is not empty.
    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init"]);

    dir
}

#[test]
fn create_returns_ticket_with_correct_fields() {
    let dir = setup();
    let root = dir.path();
    let config = Config::load(root).unwrap();

    let mut warnings = Vec::new();
    let t = ticket::create(
        root,
        &config,
        "My test ticket".to_string(),
        "agent-x".to_string(),
        None,
        None,
        false,
        vec![],
        None,
        None,
        None,
        None,
        &mut warnings,
    )
    .unwrap();

    assert_eq!(t.frontmatter.state, "new");
    assert_eq!(t.frontmatter.title, "My test ticket");
    assert_eq!(t.frontmatter.author.as_deref(), Some("agent-x"));
    assert!(t.frontmatter.branch.as_deref().unwrap_or("").starts_with("ticket/"));
    assert!(t.frontmatter.created_at.is_some());
    let branch = t.frontmatter.branch.unwrap();
    assert!(branch.contains("my-test-ticket"));
}

#[test]
fn create_sets_owner_to_author() {
    let dir = setup();
    let root = dir.path();
    let config = Config::load(root).unwrap();

    let mut warnings = Vec::new();
    let t = ticket::create(
        root,
        &config,
        "Owner test ticket".to_string(),
        "agent-owner".to_string(),
        None,
        None,
        false,
        vec![],
        None,
        None,
        None,
        None,
        &mut warnings,
    )
    .unwrap();

    assert_eq!(t.frontmatter.owner.as_deref(), Some("agent-owner"));

    // Re-parse the persisted content from git and verify owner is in the frontmatter.
    let branch = t.frontmatter.branch.as_deref().unwrap();
    let slug = branch.strip_prefix("ticket/").unwrap();
    let git_path = format!("tickets/{slug}.md");
    let out = Command::new("git")
        .args(["show", &format!("{branch}:{git_path}")])
        .current_dir(root)
        .output()
        .unwrap();
    let content = String::from_utf8(out.stdout).unwrap();
    assert!(content.contains("owner = \"agent-owner\""), "owner not in persisted frontmatter: {content}");
}

#[test]
fn create_branch_exists_in_repo() {
    let dir = setup();
    let root = dir.path();
    let config = Config::load(root).unwrap();

    let mut warnings = Vec::new();
    let t = ticket::create(
        root,
        &config,
        "Branch check".to_string(),
        "agent-y".to_string(),
        None,
        None,
        false,
        vec![],
        None,
        None,
        None,
        None,
        &mut warnings,
    )
    .unwrap();

    let branch = t.frontmatter.branch.unwrap();
    let out = Command::new("git")
        .args(["branch", "--list", &branch])
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(&branch), "branch {branch} not found in repo");
}

#[test]
fn create_context_injected_into_problem() {
    let dir = setup();
    let root = dir.path();
    let config = Config::load(root).unwrap();

    let mut warnings = Vec::new();
    let t = ticket::create(
        root,
        &config,
        "Context ticket".to_string(),
        "agent-z".to_string(),
        Some("the context text".to_string()),
        None,
        false,
        vec![],
        None,
        None,
        None,
        None,
        &mut warnings,
    )
    .unwrap();

    assert!(t.body.contains("the context text"), "context not injected into body");
    assert!(t.body.contains("### Problem\n\nthe context text"), "context not in Problem section");
}

#[test]
fn create_no_push_when_not_aggressive() {
    // With no remote configured, aggressive=false means no push attempt.
    // This just ensures create() succeeds and doesn't error.
    let dir = setup();
    let root = dir.path();
    let config = Config::load(root).unwrap();

    let mut warnings = Vec::new();
    ticket::create(
        root,
        &config,
        "No push ticket".to_string(),
        "agent-q".to_string(),
        None,
        None,
        false,
        vec![],
        None,
        None,
        None,
        None,
        &mut warnings,
    )
    .unwrap();
}
