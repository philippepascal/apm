use apm_core::git;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-c").arg("init.defaultBranch=main")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .status()
        .unwrap();
    assert!(status.success(), "git {:?} failed", args);
}

fn git_out(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// Create a bare origin and two clones. Returns (origin_dir, clone_a_dir, clone_b_dir).
/// The bare origin has one commit on main.
fn setup_two_clones() -> (TempDir, TempDir, TempDir) {
    let origin = tempfile::tempdir().unwrap();
    let clone_a = tempfile::tempdir().unwrap();
    let clone_b = tempfile::tempdir().unwrap();

    // Init bare origin.
    git(origin.path(), &["init", "--bare", "-q"]);

    // Clone A: init, add remote, push initial commit.
    git(clone_a.path(), &["init", "-q", "-b", "main"]);
    git(clone_a.path(), &["config", "user.email", "test@test.com"]);
    git(clone_a.path(), &["config", "user.name", "test"]);
    git(clone_a.path(), &["remote", "add", "origin", origin.path().to_str().unwrap()]);
    std::fs::write(clone_a.path().join("README"), "init").unwrap();
    git(clone_a.path(), &["add", "README"]);
    git(clone_a.path(), &["-c", "commit.gpgsign=false", "commit", "-m", "init"]);
    git(clone_a.path(), &["push", "origin", "HEAD:main"]);

    // Clone B: clone from origin.
    git(clone_b.path(), &["init", "-q", "-b", "main"]);
    git(clone_b.path(), &["config", "user.email", "test@test.com"]);
    git(clone_b.path(), &["config", "user.name", "test"]);
    git(clone_b.path(), &["remote", "add", "origin", origin.path().to_str().unwrap()]);
    git(clone_b.path(), &["fetch", "origin"]);
    git(clone_b.path(), &["checkout", "-b", "main", "origin/main"]);

    (origin, clone_a, clone_b)
}

/// After clone A pushes a new commit on a ticket branch, clone B fetches and calls
/// sync_local_ticket_refs. The local ref on clone B should match origin's tip.
#[test]
fn new_origin_branch_gains_local_ref() {
    let (_origin, clone_a, clone_b) = setup_two_clones();
    let a = clone_a.path();
    let b = clone_b.path();

    // Clone A creates and pushes ticket/abc-test.
    git(a, &["checkout", "-b", "ticket/abc-test"]);
    std::fs::write(a.join("ticket.txt"), "data").unwrap();
    git(a, &["add", "ticket.txt"]);
    git(a, &["-c", "commit.gpgsign=false", "commit", "-m", "add ticket"]);
    git(a, &["push", "origin", "ticket/abc-test"]);

    let origin_sha = git_out(a, &["rev-parse", "refs/remotes/origin/ticket/abc-test"]);

    // Clone B has no local ref for ticket/abc-test yet.
    git::fetch_all(b).unwrap();
    let mut _w = Vec::new();
    git::sync_local_ticket_refs(b, &mut _w);

    let local_sha = git_out(b, &["rev-parse", "refs/heads/ticket/abc-test"]);
    assert_eq!(local_sha, origin_sha, "local ref should match origin after sync");
}

/// A ticket branch already equal to origin is left unchanged (no error, same SHA).
#[test]
fn existing_local_ref_equal_to_origin_unchanged() {
    let (_origin, clone_a, clone_b) = setup_two_clones();
    let a = clone_a.path();
    let b = clone_b.path();

    git(a, &["checkout", "-b", "ticket/eql-test"]);
    std::fs::write(a.join("ticket.txt"), "data").unwrap();
    git(a, &["add", "ticket.txt"]);
    git(a, &["-c", "commit.gpgsign=false", "commit", "-m", "add ticket"]);
    git(a, &["push", "origin", "ticket/eql-test"]);

    // Clone B fetches and creates its own local ref equal to origin.
    git::fetch_all(b).unwrap();
    git(b, &["branch", "ticket/eql-test", "refs/remotes/origin/ticket/eql-test"]);

    let before = git_out(b, &["rev-parse", "refs/heads/ticket/eql-test"]);
    let mut _w = Vec::new();
    git::sync_local_ticket_refs(b, &mut _w);
    let after = git_out(b, &["rev-parse", "refs/heads/ticket/eql-test"]);

    assert_eq!(before, after, "ref should be unchanged when already equal to origin");
}

/// After clone A pushes a second commit, clone B fetches and sync advances the local ref.
#[test]
fn origin_ahead_local_ref_is_advanced() {
    let (_origin, clone_a, clone_b) = setup_two_clones();
    let a = clone_a.path();
    let b = clone_b.path();

    git(a, &["checkout", "-b", "ticket/adv-test"]);
    std::fs::write(a.join("f1.txt"), "v1").unwrap();
    git(a, &["add", "f1.txt"]);
    git(a, &["-c", "commit.gpgsign=false", "commit", "-m", "first"]);
    git(a, &["push", "origin", "ticket/adv-test"]);

    // Clone B gets the first commit.
    git::fetch_all(b).unwrap();
    let mut _w = Vec::new();
    git::sync_local_ticket_refs(b, &mut _w);
    let sha_after_first = git_out(b, &["rev-parse", "refs/heads/ticket/adv-test"]);

    // Clone A pushes a second commit.
    std::fs::write(a.join("f2.txt"), "v2").unwrap();
    git(a, &["add", "f2.txt"]);
    git(a, &["-c", "commit.gpgsign=false", "commit", "-m", "second"]);
    git(a, &["push", "origin", "ticket/adv-test"]);
    let origin_sha = git_out(a, &["rev-parse", "refs/remotes/origin/ticket/adv-test"]);

    // Clone B fetches and syncs again.
    git::fetch_all(b).unwrap();
    let mut _w = Vec::new();
    git::sync_local_ticket_refs(b, &mut _w);
    let sha_after_second = git_out(b, &["rev-parse", "refs/heads/ticket/adv-test"]);

    assert_ne!(sha_after_first, sha_after_second, "ref should have advanced");
    assert_eq!(sha_after_second, origin_sha, "local ref should match origin");
}

/// A branch checked out in a permanent worktree must not be updated.
#[test]
fn checked_out_in_worktree_is_skipped() {
    let (_origin, clone_a, clone_b) = setup_two_clones();
    let a = clone_a.path();
    let b = clone_b.path();

    // Clone A: push two commits to ticket/wt-test.
    git(a, &["checkout", "-b", "ticket/wt-test"]);
    std::fs::write(a.join("f1.txt"), "v1").unwrap();
    git(a, &["add", "f1.txt"]);
    git(a, &["-c", "commit.gpgsign=false", "commit", "-m", "first"]);
    git(a, &["push", "origin", "ticket/wt-test"]);

    // Clone B fetches the first commit and creates a worktree for the branch.
    git::fetch_all(b).unwrap();
    git(b, &["branch", "ticket/wt-test", "refs/remotes/origin/ticket/wt-test"]);
    let wt = tempfile::tempdir().unwrap();
    git(b, &["worktree", "add", wt.path().to_str().unwrap(), "ticket/wt-test"]);

    let local_sha_before = git_out(b, &["rev-parse", "refs/heads/ticket/wt-test"]);

    // Clone A pushes a second commit.
    std::fs::write(a.join("f2.txt"), "v2").unwrap();
    git(a, &["add", "f2.txt"]);
    git(a, &["-c", "commit.gpgsign=false", "commit", "-m", "second"]);
    git(a, &["push", "origin", "ticket/wt-test"]);

    // Clone B fetches and calls sync — branch is checked out in worktree, must be skipped.
    git::fetch_all(b).unwrap();
    let mut _w = Vec::new();
    git::sync_local_ticket_refs(b, &mut _w);

    let local_sha_after = git_out(b, &["rev-parse", "refs/heads/ticket/wt-test"]);
    assert_eq!(local_sha_before, local_sha_after, "checked-out branch ref must not be updated");

    // Clean up the worktree so TempDir can be dropped.
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force", wt.path().to_str().unwrap()])
        .current_dir(b)
        .status();
}

/// A branch checked out in the main worktree must not be updated.
#[test]
fn checked_out_in_main_worktree_is_skipped() {
    let (_origin, clone_a, clone_b) = setup_two_clones();
    let a = clone_a.path();
    let b = clone_b.path();

    // Clone A: push a commit to ticket/main-co-test.
    git(a, &["checkout", "-b", "ticket/main-co-test"]);
    std::fs::write(a.join("f1.txt"), "v1").unwrap();
    git(a, &["add", "f1.txt"]);
    git(a, &["-c", "commit.gpgsign=false", "commit", "-m", "first"]);
    git(a, &["push", "origin", "ticket/main-co-test"]);

    // Clone B checks out the ticket branch directly.
    git::fetch_all(b).unwrap();
    git(b, &["checkout", "-b", "ticket/main-co-test", "refs/remotes/origin/ticket/main-co-test"]);

    let local_sha_before = git_out(b, &["rev-parse", "refs/heads/ticket/main-co-test"]);

    // Clone A pushes a second commit.
    std::fs::write(a.join("f2.txt"), "v2").unwrap();
    git(a, &["add", "f2.txt"]);
    git(a, &["-c", "commit.gpgsign=false", "commit", "-m", "second"]);
    git(a, &["push", "origin", "ticket/main-co-test"]);

    git::fetch_all(b).unwrap();
    let mut _w = Vec::new();
    git::sync_local_ticket_refs(b, &mut _w);

    let local_sha_after = git_out(b, &["rev-parse", "refs/heads/ticket/main-co-test"]);
    assert_eq!(local_sha_before, local_sha_after, "main-worktree checked-out branch must not be updated");
}
