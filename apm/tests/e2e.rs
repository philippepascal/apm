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

        // Commit source files to main before apm init.
        git_ok(p, &["add", "src/"]);
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

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
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
    assert!(env.root().join("apm.agents.md").exists(), "apm.agents.md missing");
    assert!(env.root().join("apm.toml").exists(), "apm.toml missing");
    assert!(env.root().join(".git/hooks/pre-push").exists(), "pre-push hook missing");
    assert!(env.root().join(".git/hooks/post-merge").exists(), "post-merge hook missing");

    let claude = env.read("CLAUDE.md");
    assert!(claude.contains("@apm.agents.md"), "CLAUDE.md missing @apm.agents.md import");

    // ── Step 2: create a ticket ─────────────────────────────────────────────
    // Agent creates a ticket for the parse_count bug.

    let out = env.apm_as("test-agent", &["new", "Fix parse_count off-by-one"]);
    assert!(out.status.success(), "apm new failed:\n{}", stderr(&out));
    assert!(stdout(&out).contains("Created ticket #1"), "unexpected output: {}", stdout(&out));

    let branch = "ticket/0001-fix-parse-count-off-by-one";
    let ticket_path = "tickets/0001-fix-parse-count-off-by-one.md";

    // Branch exists locally.
    assert!(env.branch_exists(branch), "ticket branch not created");

    // Ticket file lives on the branch, NOT in the working tree on main.
    assert!(
        !env.root().join(ticket_path).exists(),
        "ticket file should not be in working tree on main before sync"
    );

    // Frontmatter is correct.
    let ticket = env.branch_content(branch, ticket_path);
    assert!(ticket.contains("id = 1"), "wrong id");
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

    git_ok(env.root(), &["checkout", branch]);
    assert_eq!(env.current_branch(), branch);

    // Preserve the frontmatter written by apm new; replace only the body.
    let existing = env.read(ticket_path);
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

    env.write(ticket_path, &format!("{frontmatter}{new_body}"));
    git_ok(env.root(), &["-c", "commit.gpgsign=false", "add", ticket_path]);
    git_ok(env.root(), &["-c", "commit.gpgsign=false", "commit", "-m", "ticket(1): write spec"]);
    git_ok(env.root(), &["checkout", "main"]);
    assert_eq!(env.current_branch(), "main");

    // Spec commit is on the ticket branch but not on main.
    let branch_commits = env.commits_on_branch(branch, "main");
    assert!(
        branch_commits.iter().any(|c| c.contains("write spec")),
        "spec commit not found on ticket branch"
    );

    // ── Step 4: apm state new → specd ───────────────────────────────────────
    // Before apm state can read the ticket, sync it to the working tree.

    git_ok(env.root(), &["checkout", branch, "--", ticket_path]);

    let out = env.apm_as("test-agent", &["state", "1", "specd"]);
    assert!(out.status.success(), "apm state specd failed:\n{}", stderr(&out));
    assert!(stdout(&out).contains("new → specd"), "unexpected output: {}", stdout(&out));

    let ticket = env.branch_content(branch, ticket_path);
    assert!(ticket.contains("state = \"specd\""), "state not updated to specd");
    assert!(ticket.contains("| new | specd |"), "history row missing");
    assert!(ticket.contains("updated_at"), "updated_at not refreshed");

    // ── Step 5: supervisor approves — apm state specd → ready ───────────────

    git_ok(env.root(), &["checkout", branch, "--", ticket_path]);

    let out = env.apm_as("philippe", &["state", "1", "ready"]);
    assert!(out.status.success(), "apm state ready failed:\n{}", stderr(&out));

    let ticket = env.branch_content(branch, ticket_path);
    assert!(ticket.contains("state = \"ready\""), "state not updated to ready");
    assert!(ticket.contains("| specd | ready |"), "history row missing");

    // ── Step 6: agent claims ticket — apm start ──────────────────────────────
    // apm start transitions ready → in_progress, sets agent, checks out branch.

    git_ok(env.root(), &["checkout", branch, "--", ticket_path]);

    let out = env.apm_as("test-agent", &["start", "1"]);
    assert!(out.status.success(), "apm start failed:\n{}", stderr(&out));
    assert!(stdout(&out).contains("in_progress"), "unexpected output: {}", stdout(&out));

    // We're now on the ticket branch.
    assert_eq!(env.current_branch(), branch, "apm start did not check out ticket branch");

    let ticket = env.branch_content(branch, ticket_path);
    assert!(ticket.contains("state = \"in_progress\""), "state not in_progress");
    assert!(ticket.contains("agent = \"test-agent\""), "agent not set");
    assert!(ticket.contains("| ready | in_progress |"), "history row missing");

    // ── Step 7: agent fixes the bug ──────────────────────────────────────────
    // Edit src/parser.rs on the ticket branch and commit.

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
    env.write("src/parser.rs", fixed);
    git_ok(env.root(), &["-c", "commit.gpgsign=false", "add", "src/parser.rs"]);
    git_ok(env.root(), &["-c", "commit.gpgsign=false", "commit", "-m", "ticket(1): fix parse_count off-by-one"]);

    // Code fix commit is on the ticket branch.
    let branch_commits = env.commits_on_branch(branch, "main");
    assert!(
        branch_commits.iter().any(|c| c.contains("fix parse_count")),
        "code fix commit not found on ticket branch"
    );

    // Fixed file is in the working tree.
    let src = env.read("src/parser.rs");
    assert!(!src.contains("- 1"), "bug still present in fixed file");
    assert!(src.contains("if input.is_empty()"), "empty guard missing");

    // ── Step 8: agent checks acceptance criteria boxes ───────────────────────

    let ticket_content = env.read(ticket_path);
    let checked = ticket_content
        .replace("- [ ] `parse_count(\"\")` returns 0 without panicking", "- [x] `parse_count(\"\")` returns 0 without panicking")
        .replace("- [ ] `parse_count(\"a\")` returns 1", "- [x] `parse_count(\"a\")` returns 1")
        .replace("- [ ] `parse_count(\"a,b,c\")` returns 3", "- [x] `parse_count(\"a,b,c\")` returns 3")
        .replace("- [ ] Existing `parse_items` behaviour is unchanged", "- [x] Existing `parse_items` behaviour is unchanged");
    env.write(ticket_path, &checked);
    git_ok(env.root(), &["-c", "commit.gpgsign=false", "add", ticket_path]);
    git_ok(env.root(), &["-c", "commit.gpgsign=false", "commit", "-m", "ticket(1): check acceptance criteria"]);

    // All boxes checked.
    let ticket = env.branch_content(branch, ticket_path);
    assert!(!ticket.contains("- [ ]"), "unchecked boxes remain");
    assert_eq!(ticket.matches("- [x]").count(), 4, "expected 4 checked boxes");

    // ── Step 9: apm state in_progress → implemented ─────────────────────────

    let out = env.apm_as("test-agent", &["state", "1", "implemented"]);
    assert!(out.status.success(), "apm state implemented failed:\n{}", stderr(&out));

    let ticket = env.branch_content(branch, ticket_path);
    assert!(ticket.contains("state = \"implemented\""), "state not implemented");
    assert!(ticket.contains("| in_progress | implemented |"), "history row missing");

    // ── Step 10: merge ticket branch into main ───────────────────────────────
    // Simulates a PR merge.

    git_ok(env.root(), &["checkout", "main"]);
    git_ok(env.root(), &[
        "-c", "commit.gpgsign=false",
        "merge", "--no-ff", branch,
        "-m", "Merge ticket/0001 — Fix parse_count off-by-one",
    ]);

    // After merge: main has the fixed parser.rs.
    let src = env.read("src/parser.rs");
    assert!(!src.contains("- 1"), "merged main still has the bug");
    assert!(src.contains("if input.is_empty()"), "fix not in main after merge");

    // ── Step 11: apm sync detects merged branch → accepted ──────────────────
    // --offline skips the remote fetch/push; --quiet suppresses progress output.

    // First sync the working-tree cache so sync can read the ticket.
    git_ok(env.root(), &["checkout", branch, "--", ticket_path]);

    let out = env.apm(&["sync", "--offline"]);
    assert!(out.status.success(), "apm sync failed:\n{}", stderr(&out));
    assert!(
        stdout(&out).contains("implemented → accepted"),
        "auto-transition not reported:\n{}",
        stdout(&out)
    );

    // Local cache now has state = accepted.
    let cached = env.read(ticket_path);
    assert!(cached.contains("state = \"accepted\""), "local cache not updated to accepted");
    assert!(cached.contains("| implemented | accepted |"), "history row missing in cache");
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
actionable_states = ["new", "ready"]

[workflow.prioritization]
priority_weight = 10.0
effort_weight   = -2.0
risk_weight     = -1.0

[[workflow.states]]
id = "new"
label = "New"

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
id = "ready"
label = "Ready"

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true
"#,
    ).unwrap();

    git_ok(p, &["-c", "commit.gpgsign=false", "add", "apm.toml"]);
    git_ok(p, &["-c", "commit.gpgsign=false", "commit", "-m", "init", "--allow-empty"]);
    std::fs::create_dir_all(p.join("tickets")).unwrap();

    // Create a ticket and sync it to the working tree.
    let out = apm_env(p, "test-agent", &["new", "Enforcement test"]);
    assert!(out.status.success());

    let branch = "ticket/0001-enforcement-test";
    let ticket_path = "tickets/0001-enforcement-test.md";
    git_ok(p, &["checkout", branch, "--", ticket_path]);

    // new → specd is allowed.
    let out = apm_env(p, "test-agent", &["state", "1", "specd"]);
    assert!(out.status.success(), "new → specd should be allowed:\n{}", stderr(&out));

    git_ok(p, &["checkout", branch, "--", ticket_path]);

    // specd → closed is NOT allowed (no such transition defined).
    let out = apm_env(p, "test-agent", &["state", "1", "closed"]);
    assert!(!out.status.success(), "specd → closed should be rejected");
    let err = stderr(&out);
    assert!(err.contains("no transition"), "expected transition error, got: {err}");
    assert!(err.contains("specd"), "error should mention current state");
    assert!(err.contains("closed"), "error should mention target state");

    // Ticket state unchanged after rejected transition.
    git_ok(p, &["checkout", branch, "--", ticket_path]);
    let out = apm_env(p, "test-agent", &["state", "1", "ready"]);
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
    let out = env.apm_as("test-agent", &["new", "High priority task"]);
    assert!(out.status.success());
    let out = env.apm_as("test-agent", &["new", "Medium priority task"]);
    assert!(out.status.success());

    // Sync all three to working tree.
    for (id, slug) in &[
        ("0001", "low-priority-task"),
        ("0002", "high-priority-task"),
        ("0003", "medium-priority-task"),
    ] {
        let branch = format!("ticket/{id}-{slug}");
        let path = format!("tickets/{id}-{slug}.md");
        git_ok(env.root(), &["checkout", &branch, "--", &path]);
    }

    // Set priorities.
    env.apm(&["set", "1", "priority", "1"]);
    env.apm(&["set", "2", "priority", "9"]);
    env.apm(&["set", "3", "priority", "5"]);

    // Sync updated frontmatter.
    for (id, slug) in &[
        ("0001", "low-priority-task"),
        ("0002", "high-priority-task"),
        ("0003", "medium-priority-task"),
    ] {
        let branch = format!("ticket/{id}-{slug}");
        let path = format!("tickets/{id}-{slug}.md");
        git_ok(env.root(), &["checkout", &branch, "--", &path]);
    }

    // apm next --json should return the highest-priority actionable ticket.
    let out = env.apm(&["next", "--json"]);
    assert!(out.status.success(), "apm next failed:\n{}", stderr(&out));
    let json = stdout(&out);
    assert!(json.contains("\"id\":2") || json.contains("\"id\": 2"), "expected ticket #2 (highest priority), got: {json}");

    // Move #2 to specd (not actionable) — next should now return #3.
    git_ok(env.root(), &["checkout", "ticket/0002-high-priority-task", "--", "tickets/0002-high-priority-task.md"]);
    env.apm(&["state", "2", "specd"]);
    git_ok(env.root(), &["checkout", "ticket/0002-high-priority-task", "--", "tickets/0002-high-priority-task.md"]);

    let out = env.apm(&["next", "--json"]);
    assert!(out.status.success());
    let json = stdout(&out);
    assert!(json.contains("\"id\":3") || json.contains("\"id\": 3"), "expected ticket #3 after #2 moved to specd, got: {json}");
}
