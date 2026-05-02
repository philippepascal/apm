+++
id = "059e2e74"
title = "Replace direct ticket-file writes with apm new"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/059e2e74-replace-direct-ticket-file-writes-with-a"
created_at = "2026-05-01T20:27:29.576253Z"
updated_at = "2026-05-02T04:25:18.763612Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

The integration test file `apm/tests/integration.rs` contains ten helper functions that build ticket TOML frontmatter as raw string literals and write them directly to disk:

- `write_ticket_to_branch` (17 call sites) — generic helper covering states `new`, `ready`, `in_progress`, `implemented`, `ammend`
- `write_closed_ticket` (21 call sites) — always state `closed`
- `write_spec_ticket` (17 call sites) — state `in_progress` with Problem and Approach content
- `write_implemented_ticket` (4 call sites) — state `implemented`, used by squash/merge tests
- `write_in_progress_ticket` (4 call sites) — state `in_progress`, optional `target_branch` field
- `write_ticket_with_amendment_requests` (5 call sites) — state `ammend` with checkbox amendment content
- `write_ticket_with_owner` (7 call sites) — any state, adds `owner` field
- `write_ticket_with_epic` (3 call sites) — any state, optional `epic` field
- `write_ticket_in_epic` (6 call sites) — any state, `epic` + `owner` fields
- `write_ticket_with_agent` (0 call sites, dead code) — writes `agent` field

Beyond the helpers, five inline ticket constructions write frontmatter directly inside specific test bodies (lines ~660, ~999, ~1318, ~1879, ~3141). Three further constructions use `apm_core::git::commit_to_branch` with `concat!`-built frontmatter strings (lines ~393–430).

All of these share the same flaw: the frontmatter is synthesised offline, so tests use legacy integer IDs (`1`, `2`) rather than the production 8-character hex format, and silently stay green when required fields are added, field names change, or branch-naming rules evolve.

The desired state is that every ticket fixture goes through the real `apm` CLI (`apm new`, `apm state`, `apm set`, `apm spec`) so the test fixtures track production behaviour. Where a test deliberately requires a state that is unreachable through normal CLI flows — a ticket whose `branch` field references a non-existent remote branch, a field with no CLI setter — the direct write is retained and annotated `// BYPASS: <specific reason>`.

### Acceptance criteria

- [ ] `cargo test -p apm --test integration` passes with no new failures after all changes
- [ ] Every migrated helper body invokes `apm new` via `env!("CARGO_BIN_EXE_apm")` instead of constructing raw `+++\n` frontmatter strings
- [ ] Ticket IDs in migrated fixtures are dynamically generated 8-char hex strings (as produced by `apm new`), not hardcoded integers or fixed string literals
- [ ] No helper function or test body calls `write_ticket_with_agent` (the function is deleted)
- [ ] Every direct TOML write that cannot be replaced has a `// BYPASS: <specific reason>` comment on the immediately preceding line
- [ ] No migrated call site passes a hardcoded integer ID or pre-computed branch name as ticket identity; callers use the `(id, branch)` tuple returned by the helper

### Out of scope

- Migrating setup helpers (`setup()`, `setup_merge()`, `setup_with_close_workflow()`, `setup_with_local_worktrees()`, `setup_with_worktrees()`, `setup_sync_repo()`, etc.) — each has a dedicated sibling ticket in the epic
- Adding or removing test functions — only helper bodies and their call sites change
- Adding new `apm` CLI commands to support fields that currently have no setter (e.g., `focus_section`, standalone `target_branch`)
- Removing the `apm.toml` legacy fallback from `Config::load` — covered by ticket 40fdde3b, intentionally last in the epic
- Migrating `commit_ticket_to_branch()`, `push_to_origin()`, `remote_ref_sha()`, or other utility/support functions that are not ticket-content helpers
- Writing new test scenarios — scope is migrating existing direct writes, not expanding test coverage

### Approach

All changes are in `apm/tests/integration.rs`.

**Step 1 — Add `run_apm` and `create_ticket` primitives**

Insert these two helpers near the existing `git()` helper (around line 34), before any ticket-writing helper:

```rust
fn run_apm(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_apm");
    let out = std::process::Command::new(bin)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "apm {:?} failed:\nstdout: {}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    out
}

/// Create a ticket via `apm new` and return (id, branch).
fn create_ticket(dir: &std::path::Path, title: &str) -> (String, String) {
    let out = run_apm(dir, &["new", "--no-edit", "--no-aggressive", title]);
    // stdout format: "Created ticket {id}: {filename} (branch: {branch})\n"
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout.lines().find(|l| l.starts_with("Created ticket")).unwrap();
    // e.g. "Created ticket a1b2c3d4: a1b2c3d4-my-title.md (branch: ticket/a1b2c3d4-my-title)"
    let id = line.split_whitespace().nth(2).unwrap().to_string();
    let branch = line
        .split("(branch: ").nth(1).unwrap()
        .trim_end_matches(')')
        .to_string();
    (id, branch)
}
```

**Step 2 — Replace helper bodies**

Rewrite each helper body in-place. Signature changes where noted; update call sites afterwards (Step 3).

`write_ticket_to_branch(dir, state, title) -> (String, String)`
- Drop parameters `branch: &str`, `filename: &str`, `id: u32`; add return type
- Body: `create_ticket(dir, title)` → `(id, branch)`. If state is `implemented`, first call `run_apm(dir, &["spec", &id, "--section", "Acceptance criteria", "--set", "- [x] Done"])`. Then `run_apm(dir, &["state", &id, state, "--force", "--no-aggressive"])`. Return `(id, branch)`.

`write_closed_ticket(dir, slug) -> (String, String)`
- Drop parameter `id: u32`; return remains `(String, String)` = (branch, rel_path)
- Body: `create_ticket(dir, slug)` → `(id, branch)`. `run_apm(dir, &["state", &id, "closed", "--no-aggressive"])`. Compute `rel_path` from `branch` (strip `ticket/` prefix, append `.md`). Return `(branch, rel_path)`.

`write_implemented_ticket(dir)`
- Drop parameters `branch: &str`, `filename: &str`
- Body: `run_apm(dir, &["new", "--no-edit", "--no-aggressive", "--section", "Acceptance criteria", "--set", "- [x] Done", "Squash test"])`. Parse id from stdout. `run_apm(dir, &["state", &id, "implemented", "--force", "--no-aggressive"])`. `git(dir, &["checkout", "main"])`.

`write_in_progress_ticket(dir, title) -> (String, String)` — for `target_branch = None` call sites only
- Drop parameters `id: &str`, `branch: &str`, `filename: &str`, `target_branch: Option<&str>`; add `title: &str`, return `(String, String)`
- Body: `create_ticket(dir, title)` → `(id, branch)`. `run_apm(dir, &["state", &id, "in_progress", "--force", "--no-aggressive"])`. `git(dir, &["checkout", "main"])`. Return `(id, branch)`.
- Call sites where `target_branch = Some(...)`: keep existing direct-write body for those specific calls, mark with `// BYPASS: target_branch field has no CLI setter independent of epic setup; the test needs a specific target_branch value without creating a real epic`.

`write_spec_ticket(dir, problem, approach)` — drop `id: u32`
- Body: `create_ticket(dir, "spec test")` → `(id, branch)`. Set spec sections via `run_apm`: `--section Problem --set <problem>`, `--section "Acceptance criteria" --set "- [ ] criterion one"`, `--section "Out of scope" --set "nothing"`, `--section Approach --set <approach>`. `run_apm(dir, &["state", &id, "in_progress", "--force", "--no-aggressive"])`. `git(dir, &["checkout", "main"])`.

`write_ticket_with_amendment_requests(dir)` — drop `id: u32`
- Body: `create_ticket(dir, "spec test")` → `(id, branch)`. Set Problem, AC, Out of scope, Approach via `run_apm`. `run_apm(dir, &["state", &id, "specd", "--force", "--no-aggressive"])`. `run_apm(dir, &["spec", &id, "--section", "Amendment requests", "--set", "- [ ] Add error handling\n- [ ] Fix the bug"])`. `run_apm(dir, &["state", &id, "ammend", "--no-aggressive"])`. `git(dir, &["checkout", "main"])`.

`write_ticket_with_owner(dir, state, title, owner) -> (String, String)` — drop `branch`, `filename`, `id`; add return
- Body: `create_ticket(dir, title)` → `(id, branch)`. `run_apm(dir, &["set", &id, "owner", owner])`. `run_apm(dir, &["state", &id, state, "--force", "--no-aggressive"])`. `git(dir, &["checkout", "main"])`. Return `(id, branch)`.

`write_ticket_with_epic(dir, state, title, epic: Option<&str>) -> (String, String)` — drop `branch`, `filename`, `id`; add return
- Body: If `epic = Some(e)`, call `run_apm(dir, &["new", "--no-edit", "--no-aggressive", "--epic", e, title])` and parse `(id, branch)` from stdout. If `epic = None`, call `create_ticket(dir, title)`. Then `run_apm(dir, &["state", &id, state, "--force", "--no-aggressive"])`. `git(dir, &["checkout", "main"])`. Return `(id, branch)`.

`write_ticket_in_epic(dir, state, title, owner, epic_id) -> (String, String)` — drop `branch`, `filename`, `id`; add return
- Body: `run_apm(dir, &["new", "--no-edit", "--no-aggressive", "--epic", epic_id, title])` → `(id, branch)`. `run_apm(dir, &["set", &id, "owner", owner])`. `run_apm(dir, &["state", &id, state, "--force", "--no-aggressive"])`. `git(dir, &["checkout", "main"])`. Return `(id, branch)`.

**Step 3 — Update all call sites**

For every helper whose signature changed, update each call site to:
- Pass the new parameters (drop `branch`, `filename`, `id` args)
- Capture `(id, branch)` from the return value
- Replace any later reference to the old hardcoded branch string with the captured `branch`

Key locations: `write_ticket_to_branch` has 17 call sites; `write_closed_ticket` has 21 (already captures return); `write_ticket_with_owner` has 7; `write_ticket_with_epic` has 3; `write_ticket_in_epic` has 6.

For `write_in_progress_ticket` call sites where `target_branch = Some(...)` (lines ~4768 and ~5377): keep the direct-write body inline at the call site and annotate with `// BYPASS:`.

**Step 4 — Apply `// BYPASS:` annotations to inline constructions**

Five inline constructions cannot cleanly use `apm new`. Add the comment on the line immediately before the frontmatter string:

- Line ~999 (`sync_closes_implemented_ticket_with_no_branch`):
  `// BYPASS: ticket.branch references a branch that does not exist; apm new always creates the branch, making this scenario impossible without a direct write`
- Line ~660 (`show_displays_epic_target_branch_depends_on_when_set`):
  `// BYPASS: requires specific target_branch and depends_on values; target_branch has no CLI setter independent of epic, and depends_on IDs do not correspond to real tickets`
- Line ~1318 (`spec_fixup_with_amendment_requests_converts_checkbox`):
  `// BYPASS: tests that apm auto-converts plain bullet items to checkboxes; file must contain the pre-conversion format that apm spec would rewrite`
- Line ~1879 (`start_owner_guard_allows_owner`):
  `// BYPASS: focus_section field has no apm set subcommand`
- Line ~3141 (`clean_detects_mismatch_branch_state`):
  `// BYPASS: deliberately overwrites ticket on main with a different state than what is on the ticket branch to test mismatch detection in apm clean`

**Step 5 — Handle `concat!`-based constructions (lines ~393–430)**

These create sibling tickets with specific hardcoded IDs (`"aaaa1111"`, `"bbbb2222"`, `"cccc3333"`) that may be asserted in test output. Inspect each test function:
- If the test asserts on the specific ID string in command output → keep the `apm_core::git::commit_to_branch` call and annotate `// BYPASS: test asserts on specific ticket ID string that apm new cannot produce deterministically`
- If the test only checks count or presence without checking the specific ID → replace with `apm new --no-edit --no-aggressive --epic <id>` + `apm state --force <state>`

**Step 6 — Delete `write_ticket_with_agent`**

Remove the entire function (it has zero callers and is annotated `#[allow(dead_code)]`).

**Step 7 — Verify**

`cargo test -p apm --test integration` must pass. All `--force` transitions must succeed in CI without a remote (the `--no-aggressive` flag prevents any fetch/push).

**Known constraints**

- `apm state <state> --force` bypasses workflow transition rules but still enforces spec-content validation for `implemented` (all acceptance criteria must be `[x]`). Always seed at least one checked criterion before forcing to `implemented`.
- `apm new` title is a positional argument; confirm exact invocation with `apm new --help` during implementation.
- Parsing `apm new` stdout is fragile to format changes; the parse lives in exactly one place (`create_ticket`) so a format change breaks only that helper.
- `--no-aggressive` must be passed to every `apm new` and `apm state` call in tests; test repos have no remote and aggressive mode would hang or error.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:25Z | groomed | in_design | philippepascal |