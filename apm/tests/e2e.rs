/// End-to-end tests that run the real `apm` binary and real git commands.
///
/// Each test simulates a human or agent workflow step-by-step and asserts
/// the expected state of git, files, and apm output at every transition.
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

const APM: &str = env!("CARGO_BIN_EXE_apm");

// ---------------------------------------------------------------------------
// Test environment
// ---------------------------------------------------------------------------

struct Env {
    dir: TempDir,
}

impl Env {
    /// Create a fresh git repo with testdata source files committed on main
    /// and apm fully initialized.
    fn setup() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();

        // Init git with main as the default branch.
        git_ok(p, &["init", "-q", "-b", "main"]);
        git_ok(p, &["config", "user.email", "test@test.com"]);
        git_ok(p, &["config", "user.name", "test"]);

        // Copy testdata source files so tickets can reference real paths.
        let src_dir = p.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::copy(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../testdata/src/parser.rs"),
            src_dir.join("parser.rs"),
        ).unwrap();
        std::fs::copy(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../testdata/src/main.rs"),
            src_dir.join("main.rs"),
        ).unwrap();

        // Write apm.toml before init so worktrees dir stays inside the tempdir.
        std::fs::write(p.join("apm.toml"), r#"[project]
name = "test-repo"

[tickets]
dir = "tickets"

[worktrees]
dir = ".worktrees"

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
id         = "question"
label      = "Question"
actionable = ["supervisor"]

[[workflow.states]]
id         = "specd"
label      = "Specd"
actionable = ["supervisor"]

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
id         = "implemented"
label      = "Implemented"
actionable = ["supervisor"]

[[workflow.states]]
id         = "accepted"
label      = "Accepted"
actionable = ["supervisor"]

[[workflow.states]]
id       = "closed"
label    = "Closed"
terminal = true
"#).unwrap();

        // Commit source files and apm.toml to main before apm init.
        git_ok(p, &["add", "src/", "apm.toml"]);
        git_ok(p, &["-c", "commit.gpgsign=false", "commit", "-m", "Add source files"]);

        // apm init (--no-claude skips the interactive settings.json prompt).
        let out = apm(p, "apm", &["init", "--no-claude"]);
        assert!(out.status.success(), "apm init failed:\n{}", stderr(&out));

        Env { dir }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    /// Run an apm command as a given agent.
    fn apm_as(&self, agent: &str, args: &[&str]) -> Output {
        apm_env(self.root(), agent, args)
    }

    /// Run an apm command with no agent identity (APM_AGENT_NAME unset).
    fn apm(&self, args: &[&str]) -> Output {
        apm(self.root(), "apm", args)
    }

    /// Read a file from a git branch without touching the working tree.
    fn branch_content(&self, branch: &str, path: &str) -> String {
        let out = git(self.root(), &["show", &format!("{branch}:{path}")]);
        assert!(
            out.status.success(),
            "git show {branch}:{path} failed:\n{}",
            stderr(&out)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    /// Read a file from the working tree.
    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root().join(rel)).unwrap()
    }

    /// Write a file in the working tree.
    fn write(&self, rel: &str, content: &str) {
        let full = self.root().join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(full, content).unwrap();
    }

    /// Return the name of the currently checked-out branch.
    fn current_branch(&self) -> String {
        let out = git(self.root(), &["rev-parse", "--abbrev-ref", "HEAD"]);
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// Return true if a local branch exists.
    fn branch_exists(&self, branch: &str) -> bool {
        git(self.root(), &["rev-parse", "--verify", &format!("refs/heads/{branch}")])
            .status
            .success()
    }

    /// Return commits on `branch` that are not on `base`, most-recent first.
    fn commits_on_branch(&self, branch: &str, base: &str) -> Vec<String> {
        let out = git(
            self.root(),
            &["log", "--oneline", &format!("{base}..{branch}")],
        );
        String::from_utf8(out.stdout)
            .unwrap()
            .lines()
            .map(|l| l.to_string())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn apm(dir: &Path, _name: &str, args: &[&str]) -> Output {
    Command::new(APM)
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", dir) // isolate git config
        .env("VISUAL", "true") // prevent vi from blocking in tests
        .output()
        .unwrap()
}

fn apm_env(dir: &Path, agent: &str, args: &[&str]) -> Output {
    Command::new(APM)
        .args(args)
        .current_dir(dir)
        .env("APM_AGENT_NAME", agent)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", dir)
        .env("VISUAL", "true") // prevent vi from blocking in tests
        .output()
        .unwrap()
}

fn git(dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap()
}

fn git_ok(dir: &Path, args: &[&str]) {
    let out = git(dir, args);
    assert!(out.status.success(), "git {:?} failed:\n{}", args, stderr(&out));
}

/// Check out ticket branch, write a valid spec body, commit, return to main.
fn write_valid_spec_for_test(dir: &Path, branch: &str, ticket_path: &str) {
    git_ok(dir, &["checkout", branch]);
    let existing = std::fs::read_to_string(dir.join(ticket_path)).unwrap();
    let fm_end = existing.find("\n+++\n").expect("frontmatter close not found") + 5;
    let frontmatter = &existing[..fm_end];
    let body = "\n## Spec\n\n### Problem\n\nTest problem.\n\n### Acceptance criteria\n\n- [ ] One criterion\n\n### Out of scope\n\nNothing.\n\n### Approach\n\nDirect approach.\n\n## History\n\n| When | From | To | By |\n|------|------|----|-----|\n| 2026-01-01T00:00Z | — | new | test-agent |\n";
    std::fs::write(dir.join(ticket_path), format!("{frontmatter}{body}")).unwrap();
    git_ok(dir, &["-c", "commit.gpgsign=false", "add", ticket_path]);
    git_ok(dir, &["-c", "commit.gpgsign=false", "commit", "-m", "write spec"]);
    git_ok(dir, &["checkout", "main"]);
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Parse the ticket ID from `apm new` output.
/// Output format: "Created ticket {id}: {filename} (branch: {branch})"
fn parse_new_ticket_id(out: &Output) -> String {
    let s = stdout(out);
    s.lines()
        .find(|l| l.starts_with("Created ticket "))
        .and_then(|l| l.strip_prefix("Created ticket "))
        .and_then(|s| s.split(':').next())
        .unwrap_or_else(|| panic!("could not parse ticket ID from: {s}"))
        .trim()
        .to_string()
}

/// Parse the branch name from `apm new` output.
fn parse_new_ticket_branch(out: &Output) -> String {
    let s = stdout(out);
    s.lines()
        .find(|l| l.starts_with("Created ticket "))
        .and_then(|l| l.split("(branch: ").nth(1))
        .and_then(|s| s.strip_suffix(')').or_else(|| s.trim_end_matches('\n').strip_suffix(')')))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| panic!("could not parse branch from: {s}"))
}

// ---------------------------------------------------------------------------
// Full ticket lifecycle: new → specd → ready → in_progress → implemented →
//                        (sync detects merge) → accepted
// ---------------------------------------------------------------------------

#[test]
fn full_ticket_lifecycle() {
    let env = Env::setup();

    // ── Step 1: apm init ────────────────────────────────────────────────────
    // setup() already ran init. Verify the expected files are in place.

    assert!(env.root().join("CLAUDE.md").exists(), "CLAUDE.md missing");
    assert!(env.root().join(".apm/agents.md").exists(), ".apm/agents.md missing");
    assert!(env.root().join(".apm/config.toml").exists(), ".apm/config.toml missing");
    assert!(!env.root().join(".git/hooks/pre-push").exists(), "pre-push hook should not be installed");
    assert!(!env.root().join(".git/hooks/post-merge").exists(), "post-merge hook should not be installed");

    let claude = env.read("CLAUDE.md");
    assert!(claude.contains("@.apm/agents.md"), "CLAUDE.md missing @.apm/agents.md import");

    // ── Step 2: create a ticket ─────────────────────────────────────────────
    // Agent creates a ticket for the parse_count bug.

    // Write local identity so resolve_identity returns "test-agent".
    env.write(".apm/local.toml", "username = \"test-agent\"\n");
    let out = env.apm_as("test-agent", &["new", "Fix parse_count off-by-one"]);
    assert!(out.status.success(), "apm new failed:\n{}", stderr(&out));
    let out_text = stdout(&out);
    assert!(out_text.contains("Created ticket "), "unexpected output: {out_text}");
    assert!(out_text.contains("fix-parse-count-off-by-one"), "slug missing: {out_text}");

    let ticket_id = parse_new_ticket_id(&out);
    let branch = parse_new_ticket_branch(&out);
    let ticket_suffix = branch.strip_prefix("ticket/").unwrap().to_string();
    let ticket_path = format!("tickets/{ticket_suffix}.md");

    // Branch exists locally.
    assert!(env.branch_exists(&branch), "ticket branch not created");

    // Frontmatter is correct — read from the branch, not the working tree.
    let ticket = env.branch_content(&branch, &ticket_path);
    // id is now an 8-char hex string
    assert!(ticket.contains(&format!("id = \"{ticket_id}\"")), "wrong id in frontmatter");
    assert!(ticket.contains("state = \"new\""), "wrong state");
    assert!(ticket.contains("author = \"test-agent\""), "author not set");
    assert!(ticket.contains(&format!("branch = \"{branch}\"")), "branch not set");
    assert!(ticket.contains("created_at"), "created_at missing");
    assert!(ticket.contains("updated_at"), "updated_at missing");

    // Body scaffold includes all four required sections and history.
    assert!(ticket.contains("### Problem"), "missing ### Problem");
    assert!(ticket.contains("### Acceptance criteria"), "missing ### Acceptance criteria");
    assert!(ticket.contains("### Out of scope"), "missing ### Out of scope");
    assert!(ticket.contains("### Approach"), "missing ### Approach");
    assert!(ticket.contains("## History"), "missing ## History");
    assert!(ticket.contains("| — | new | test-agent |"), "missing creation history row");

    // ── Step 3: agent writes the spec ───────────────────────────────────────
    // Simulate: git checkout <branch>, edit ticket, commit, checkout main.

    git_ok(env.root(), &["checkout", &branch]);
    assert_eq!(env.current_branch(), branch);

    // Preserve the frontmatter written by apm new; replace only the body.
    let existing = env.read(&ticket_path);
    let fm_end = existing.find("\n+++\n").expect("frontmatter close not found") + 5;
    let frontmatter = &existing[..fm_end];

    let new_body = r#"
## Spec

### Problem

`parse_count` in `src/parser.rs` subtracts 1 from the split count, causing a
panic on empty input and returning 0 for a single-item string. Any caller
expecting a correct count gets wrong results.

### Acceptance criteria

- [ ] `parse_count("")` returns 0 without panicking
- [ ] `parse_count("a")` returns 1
- [ ] `parse_count("a,b,c")` returns 3
- [ ] Existing `parse_items` behaviour is unchanged

### Out of scope

- Changing the delimiter from comma to anything else
- Unicode or whitespace normalisation

### Approach

Remove the `- 1` in `parse_count`. Add a guard for empty input that returns 0
immediately. Update the existing tests to cover the single-item case.

## History

| When | From | To | By |
|------|------|----|-----|
| 2026-03-26T00:00Z | — | new | test-agent |
"#;

    env.write(&ticket_path, &format!("{frontmatter}{new_body}"));
    git_ok(env.root(), &["-c", "commit.gpgsign=false", "add", &ticket_path]);
    git_ok(env.root(), &["-c", "commit.gpgsign=false", "commit", "-m", &format!("ticket({ticket_id}): write spec")]);
    git_ok(env.root(), &["checkout", "main"]);
    assert_eq!(env.current_branch(), "main");

    // Spec commit is on the ticket branch but not on main.
    let branch_commits = env.commits_on_branch(&branch, "main");
    assert!(
        branch_commits.iter().any(|c| c.contains("write spec")),
        "spec commit not found on ticket branch"
    );

    // ── Step 4: apm state new → specd ───────────────────────────────────────
    // apm state reads from git blobs — no working-tree prep needed.

    let out = env.apm_as("test-agent", &["state", &ticket_id, "specd"]);
    assert!(out.status.success(), "apm state specd failed:\n{}", stderr(&out));
    assert!(stdout(&out).contains("new → specd"), "unexpected output: {}", stdout(&out));

    let ticket = env.branch_content(&branch, &ticket_path);
    assert!(ticket.contains("state = \"specd\""), "state not updated to specd");
    assert!(ticket.contains("| new | specd |"), "history row missing");
    assert!(ticket.contains("updated_at"), "updated_at not refreshed");

    // ── Step 5: supervisor approves — apm state specd → ready ───────────────

    let out = env.apm_as("philippe", &["state", &ticket_id, "ready"]);
    assert!(out.status.success(), "apm state ready failed:\n{}", stderr(&out));

    let ticket = env.branch_content(&branch, &ticket_path);
    assert!(ticket.contains("state = \"ready\""), "state not updated to ready");
    assert!(ticket.contains("| specd | ready |"), "history row missing");

    // ── Step 6: agent claims ticket — apm start ──────────────────────────────
    // apm start transitions ready → in_progress, sets agent, provisions worktree.

    let out = env.apm_as("test-agent", &["start", &ticket_id]);
    assert!(out.status.success(), "apm start failed:\n{}", stderr(&out));
    let start_out = stdout(&out);
    assert!(start_out.contains("in_progress"), "unexpected output: {}", start_out);
    assert!(start_out.contains("Worktree:"), "worktree path missing from output: {}", start_out);

    // Main worktree is still on main — agent works in the provisioned worktree.
    assert_eq!(env.current_branch(), "main", "main worktree should stay on main");

    let ticket = env.branch_content(&branch, &ticket_path);
    assert!(ticket.contains("state = \"in_progress\""), "state not in_progress");
    assert!(!ticket.contains("agent ="), "agent field must not be written");
    assert!(ticket.contains("| ready | in_progress |"), "history row missing");

    // Parse the worktree path from the output line "Worktree: /path/to/wt"
    let wt_path = start_out
        .lines()
        .find(|l| l.starts_with("Worktree:"))
        .and_then(|l| l.strip_prefix("Worktree:"))
        .map(|s| std::path::PathBuf::from(s.trim()))
        .expect("could not parse worktree path from apm start output");

    // ── Step 7: agent fixes the bug ──────────────────────────────────────────
    // Work in the worktree (the agent's checkout of the ticket branch).

    let fixed = r#"/// Parse a comma-separated list and return the item count.
pub fn parse_count(input: &str) -> usize {
    if input.is_empty() {
        return 0;
    }
    input.split(',').count()
}

pub fn parse_items(input: &str) -> Vec<&str> {
    input.split(',').map(str::trim).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_empty() {
        assert_eq!(parse_count(""), 0);
    }

    #[test]
    fn count_multiple() {
        assert_eq!(parse_count("a,b,c"), 3);
    }

    #[test]
    fn count_single() {
        assert_eq!(parse_count("a"), 1);
    }
}
"#;
    std::fs::create_dir_all(wt_path.join("src")).unwrap();
    std::fs::write(wt_path.join("src/parser.rs"), fixed).unwrap();
    git_ok(&wt_path, &["-c", "commit.gpgsign=false", "add", "src/parser.rs"]);
    git_ok(&wt_path, &["-c", "commit.gpgsign=false", "commit", "-m", &format!("ticket({ticket_id}): fix parse_count off-by-one")]);

    // Code fix commit is on the ticket branch.
    let branch_commits = env.commits_on_branch(&branch, "main");
    assert!(
        branch_commits.iter().any(|c| c.contains("fix parse_count")),
        "code fix commit not found on ticket branch"
    );

    // Fixed file is in the worktree.
    let src = std::fs::read_to_string(wt_path.join("src/parser.rs")).unwrap();
    assert!(!src.contains("- 1"), "bug still present in fixed file");
    assert!(src.contains("if input.is_empty()"), "empty guard missing");

    // ── Step 8: agent checks acceptance criteria boxes ───────────────────────
    // Work in the worktree.

    let ticket_content = std::fs::read_to_string(wt_path.join(&ticket_path)).unwrap();
    let checked = ticket_content
        .replace("- [ ] `parse_count(\"\")` returns 0 without panicking", "- [x] `parse_count(\"\")` returns 0 without panicking")
        .replace("- [ ] `parse_count(\"a\")` returns 1", "- [x] `parse_count(\"a\")` returns 1")
        .replace("- [ ] `parse_count(\"a,b,c\")` returns 3", "- [x] `parse_count(\"a,b,c\")` returns 3")
        .replace("- [ ] Existing `parse_items` behaviour is unchanged", "- [x] Existing `parse_items` behaviour is unchanged");
    std::fs::write(wt_path.join(&ticket_path), &checked).unwrap();
    git_ok(&wt_path, &["-c", "commit.gpgsign=false", "add", &ticket_path]);
    git_ok(&wt_path, &["-c", "commit.gpgsign=false", "commit", "-m", &format!("ticket({ticket_id}): check acceptance criteria")]);

    // All boxes checked.
    let ticket = env.branch_content(&branch, &ticket_path);
    assert!(!ticket.contains("- [ ]"), "unchecked boxes remain");
    assert_eq!(ticket.matches("- [x]").count(), 4, "expected 4 checked boxes");

    // ── Step 9: apm state in_progress → implemented ─────────────────────────

    let out = env.apm_as("test-agent", &["state", &ticket_id, "implemented"]);
    assert!(out.status.success(), "apm state implemented failed:\n{}", stderr(&out));

    let ticket = env.branch_content(&branch, &ticket_path);
    assert!(ticket.contains("state = \"implemented\""), "state not implemented");
    assert!(ticket.contains("| in_progress | implemented |"), "history row missing");

    // ── Step 10: merge ticket branch into main ───────────────────────────────
    // Simulates a PR merge.

    git_ok(env.root(), &["checkout", "main"]);
    git_ok(env.root(), &[
        "-c", "commit.gpgsign=false",
        "merge", "--no-ff", &branch,
        "-m", &format!("Merge {branch} — Fix parse_count off-by-one"),
    ]);

    // After merge: main has the fixed parser.rs.
    let src = env.read("src/parser.rs");
    assert!(!src.contains("- 1"), "merged main still has the bug");
    assert!(src.contains("if input.is_empty()"), "fix not in main after merge");

    // ── Step 11: apm sync detects merged branch and offers to close ─────────────
    // --offline skips the remote fetch/push.
    // sync reads tickets from git blobs directly — no working-tree prep needed.

    let out = env.apm(&["sync", "--offline"]);
    assert!(out.status.success(), "apm sync failed:\n{}", stderr(&out));
    assert!(
        stdout(&out).contains("branch merged"),
        "merge suggestion not reported:\n{}",
        stdout(&out)
    );

    // Ticket is still in implemented — no auto-close without --auto-close flag.
    let ticket_after = env.branch_content(&branch, &ticket_path);
    assert!(ticket_after.contains("state = \"implemented\""), "state should still be implemented after sync");
}

// ---------------------------------------------------------------------------
// State transition enforcement
// ---------------------------------------------------------------------------

#[test]
fn state_rejects_illegal_transition() {
    // Use a config with explicit transition rules.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git_ok(p, &["init", "-q", "-b", "main"]);
    git_ok(p, &["config", "user.email", "test@test.com"]);
    git_ok(p, &["config", "user.name", "test"]);

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
effort_weight   = -2.0
risk_weight     = -1.0

[[workflow.states]]
id         = "new"
label      = "New"
actionable = ["agent"]

[[workflow.states.transitions]]
to      = "specd"
trigger = "manual"

[[workflow.states]]
id = "specd"
label = "Specd"

[[workflow.states.transitions]]
to      = "ready"
trigger = "manual"

[[workflow.states]]
id         = "ready"
label      = "Ready"
actionable = ["agent"]

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#,
    ).unwrap();

    git_ok(p, &["-c", "commit.gpgsign=false", "add", "apm.toml"]);
    git_ok(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();

    // Create a ticket and write a valid spec body before transitioning to specd.
    let out = apm_env(p, "test-agent", &["new", "Enforcement test"]);
    assert!(out.status.success());
    let id1 = parse_new_ticket_id(&out);
    let branch1 = parse_new_ticket_branch(&out);
    let path1 = format!("tickets/{}.md", branch1.strip_prefix("ticket/").unwrap());
    write_valid_spec_for_test(p, &branch1, &path1);

    // new → specd is allowed.
    let out = apm_env(p, "test-agent", &["state", &id1, "specd"]);
    assert!(out.status.success(), "new → specd should be allowed:\n{}", stderr(&out));

    // specd → new is NOT allowed (no such transition defined, and new is not terminal).
    let out = apm_env(p, "test-agent", &["state", &id1, "new"]);
    assert!(!out.status.success(), "specd → new should be rejected");
    let err = stderr(&out);
    assert!(err.contains("no transition"), "expected transition error, got: {err}");
    assert!(err.contains("specd"), "error should mention current state");
    assert!(err.contains("new"), "error should mention target state");

    // Terminal states are always reachable regardless of transition rules.
    let out = apm_env(p, "test-agent", &["state", &id1, "closed"]);
    assert!(out.status.success(), "specd → closed should be allowed (terminal state)");

    // new → specd → ready via defined transitions (need a fresh ticket since #1 is now closed).
    let out = apm_env(p, "test-agent", &["new", "Second enforcement test"]);
    assert!(out.status.success());
    let id2 = parse_new_ticket_id(&out);
    let branch2 = parse_new_ticket_branch(&out);
    let path2 = format!("tickets/{}.md", branch2.strip_prefix("ticket/").unwrap());
    write_valid_spec_for_test(p, &branch2, &path2);
    let out = apm_env(p, "test-agent", &["state", &id2, "specd"]);
    assert!(out.status.success(), "new → specd should be allowed");
    let out = apm_env(p, "test-agent", &["state", &id2, "ready"]);
    assert!(out.status.success(), "specd → ready should be allowed");
}

// ---------------------------------------------------------------------------
// apm next prioritisation
// ---------------------------------------------------------------------------

#[test]
fn next_respects_priority_and_actionable_states() {
    let env = Env::setup();

    // Create three tickets with different priorities.
    let out = env.apm_as("test-agent", &["new", "Low priority task"]);
    assert!(out.status.success());
    let id1 = parse_new_ticket_id(&out);
    let out = env.apm_as("test-agent", &["new", "High priority task"]);
    assert!(out.status.success());
    let id2 = parse_new_ticket_id(&out);
    let branch2 = parse_new_ticket_branch(&out);
    let path2 = format!("tickets/{}.md", branch2.strip_prefix("ticket/").unwrap());
    let out = env.apm_as("test-agent", &["new", "Medium priority task"]);
    assert!(out.status.success());
    let id3 = parse_new_ticket_id(&out);

    // Set priorities (apm set/next/state read from git blobs — no working-tree prep needed).
    env.apm(&["set", &id1, "priority", "1"]);
    env.apm(&["set", &id2, "priority", "9"]);
    env.apm(&["set", &id3, "priority", "5"]);

    // Promote all tickets to groomed so they are agent-actionable.
    env.apm(&["state", &id1, "groomed"]);
    env.apm(&["state", &id2, "groomed"]);
    env.apm(&["state", &id3, "groomed"]);

    // apm next --json should return the highest-priority actionable ticket.
    let out = env.apm(&["next", "--json"]);
    assert!(out.status.success(), "apm next failed:\n{}", stderr(&out));
    let json = stdout(&out);
    // id is now a hex string
    assert!(
        json.contains(&format!("\"id\":\"{id2}\"")) || json.contains(&format!("\"id\": \"{id2}\"")),
        "expected ticket id2 (highest priority), got: {json}"
    );

    // Move id2 to specd (not actionable) — next should now return id3.
    write_valid_spec_for_test(env.root(), &branch2, &path2);
    env.apm(&["state", &id2, "specd"]);

    let out = env.apm(&["next", "--json"]);
    assert!(out.status.success());
    let json = stdout(&out);
    assert!(
        json.contains(&format!("\"id\":\"{id3}\"")) || json.contains(&format!("\"id\": \"{id3}\"")),
        "expected ticket id3 after id2 moved to specd, got: {json}"
    );
}
