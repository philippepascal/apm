use apm_core::{config::Config, git_util, ticket::Ticket, verify::verify_tickets};
use std::collections::HashSet;
use std::process::Command;
use tempfile::TempDir;

fn git(dir: &std::path::Path, args: &[&str]) {
    Command::new("git")
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

    git(p, &["init", "-q", "-b", "main"]);
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

    git(p, &["add", "apm.toml"]);
    git(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init"]);

    dir
}

fn make_ticket(root: &std::path::Path, id: &str, state: &str, branch: Option<&str>) -> Ticket {
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

#[test]
fn worktree_missing_in_design() {
    let dir = setup();
    let root = dir.path();
    let config = Config::load(root).unwrap();
    let ticket = make_ticket(root, "abcd1234", "in_design", Some("ticket/abcd1234-test"));

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
    let dir = setup();
    let root = dir.path();
    let config = Config::load(root).unwrap();
    let ticket = make_ticket(root, "abcd1234", "in_design", Some("ticket/abcd1234-test"));

    std::fs::create_dir_all(root.join("worktrees").join("ticket-abcd1234-test")).unwrap();

    let issues = verify_tickets(root, &config, &[ticket], &HashSet::new());
    assert!(
        !issues.iter().any(|i| i.contains("worktree")),
        "unexpected worktree issue; got: {issues:?}"
    );
}

#[test]
fn worktree_check_skipped_for_other_states() {
    let dir = setup();
    let root = dir.path();
    let config = Config::load(root).unwrap();
    let ticket = make_ticket(root, "abcd1234", "specd", Some("ticket/abcd1234-test"));

    let issues = verify_tickets(root, &config, &[ticket], &HashSet::new());
    assert!(
        !issues.iter().any(|i| i.contains("worktree")),
        "unexpected worktree issue for specd state; got: {issues:?}"
    );
}
