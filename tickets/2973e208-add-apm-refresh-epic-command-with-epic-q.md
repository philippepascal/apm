+++
id = "2973e208"
title = "Add apm refresh-epic command with epic quiescence check"
state = "in_design"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2973e208-add-apm-refresh-epic-command-with-epic-q"
created_at = "2026-04-27T20:28:30.358011Z"
updated_at = "2026-04-27T21:14:58.773671Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

Long-running epic branches drift from the default branch over time. There is no built-in APM command to pull default-branch updates into an epic branch. The spec at `docs/strategy-and-dependencies.md` (§ 'Refresh and close: epic must be quiescent') defines `apm refresh-epic <id>` as the supervisor-facing tool for this: it opens a PR from the default branch into the epic branch, which the supervisor reviews and merges so subsequent workers in the epic see the updated tip.

The command must refuse to run if any ticket in the epic is currently being worked on (i.e., in a state that is neither terminal nor `worker_end`, such as `in_design` or `in_progress`) or has a live worker process (alive `.apm-worker.pid`). This precondition is shared with `apm epic close` (ticket 056b1ee1), so the check must be extracted into a reusable `epic_is_quiescent()` helper in `apm-core`.

APM does not stop running workers; the supervisor is responsible for pausing the dispatcher and waiting for the active worker to complete before calling this command.

### Acceptance criteria

- [ ] `apm refresh-epic <id>` exits with a non-zero code and prints an error if the id prefix matches no epic branch
- [ ] `apm refresh-epic <id>` exits with a non-zero code and prints an error if the id prefix is ambiguous (matches multiple epic branches)
- [ ] `apm refresh-epic <id>` exits with a non-zero code and lists every blocking ticket when any epic ticket is in a state that is not terminal and not `worker_end`
- [ ] `apm refresh-epic <id>` exits with a non-zero code and lists every blocking ticket when any epic ticket has a live `.apm-worker.pid` (alive process), even if the ticket state appears clean
- [ ] `apm refresh-epic <id>` prints a message and exits 0 when the default branch has no new commits not yet present in the epic branch (nothing to refresh)
- [ ] `apm refresh-epic <id>` creates a PR with `--base <epic_branch> --head <default_branch>` when the epic is quiescent and new commits exist
- [ ] The refresh PR title is formatted as `<epic_id>: refresh from <default_branch>`
- [ ] The refresh PR body lists the commits on the default branch that are not yet in the epic branch (one per line, `--oneline` format)
- [ ] If an open PR from the default branch into the epic branch already exists, `apm refresh-epic` reports the existing PR number and exits 0 without creating a duplicate
- [ ] `apm_core::epic::epic_is_quiescent` is a public function that accepts the repo root, the epic id, the loaded config, and a list of ticket worktrees, and returns the blocking descriptions so `apm epic close` (ticket 056b1ee1) can reuse it without re-implementing the logic

### Out of scope

- Updating `apm epic close` to use `epic_is_quiescent()` — that is ticket 056b1ee1
- Auto-merging the refresh PR; the supervisor merges it manually
- Stopping or killing running workers before the refresh
- Any changes to `apm validate`, dependency rules, or strategy enforcement
- Removing the per-epic `max_workers` override (ticket 6e3f9e91)
- Changing the default completion strategy (ticket 941e57fa)
- Adding the refresh PR to any CI/automation pipeline

### Approach

**Files changed, in implementation order:**

**`apm-core/src/epic.rs`** — add `pub fn epic_is_quiescent(root, epic_id, config, worktrees) -> Result<Vec<String>>`:
- Load tickets via `load_all_from_git`; filter to `t.frontmatter.epic == Some(epic_id)`.
- For each epic ticket, look up its `StateConfig`. If `!terminal && !worker_end`, push `"  {id} — {title} (state: {state})"`.
- Find the ticket's worktree by matching its branch against the `worktrees: &[(PathBuf, String)]` slice. If found and `worker::is_alive(pid)` on its `.apm-worker.pid` returns true, push `"  {id} — {title} (live worker)"`. Skip the live-worker check if the ticket already appears from the state check.
- Return the blocker list; empty = quiescent. Accept `worktrees` as a parameter so callers can reuse an already-loaded list and the function is unit-testable.

**`apm-core/src/github.rs`** — add `pub fn gh_pr_create_or_update_between(root, head, base, title, body, messages)`:
- Check for existing open PR: `gh pr list --head <head> --base <base> --state open --json number --jq .[0].number`.
- If found, push `"PR #{n} already open ({head} → {base})"` and return `Ok(())`.
- Otherwise: `gh pr create --base <base> --head <head> --title <title> --body <body>`. On success push the URL; on failure bail.
- Leave the existing `gh_pr_create_or_update` unchanged.

**`apm/src/cmd/epic.rs`** — add `pub fn run_refresh_epic(root, id_arg)`:
1. Load config via `CmdContext::load_config_only`.
2. Resolve epic branch with `find_epic_branches`; bail on 0 or >1 matches (same messages as `run_close`).
3. Parse `epic_id` with `epic_id_from_branch`.
4. Load worktrees via `apm_core::worktree::list_ticket_worktrees(root)?`.
5. Call `epic_is_quiescent(root, epic_id, &config, &worktrees)?`; if blockers are non-empty, bail: `"cannot refresh epic: the following tickets are not quiescent:\n{blockers}"`.
6. Run `git log --oneline --no-decorate <epic_branch>..<default_branch>`. If output is empty, print `"epic branch is up to date with {default_branch}"` and return `Ok(())`.
7. Build `pr_title = "{epic_id}: refresh from {default_branch}"` and `pr_body` = the raw `--oneline` output.
8. Push the epic branch to remote with `git::push_branch_tracking` (ensures the base ref exists on GitHub).
9. Call `gh_pr_create_or_update_between(root, default_branch, &epic_branch, &pr_title, &pr_body, &mut messages)?` and print each message.

**`apm/src/main.rs`**:
- Add `RefreshEpic { id: String }` with doc comment `/// Pull default-branch updates into an epic branch` to the top-level `Command` enum.
- Dispatch: `Command::RefreshEpic { id } => cmd::epic::run_refresh_epic(&root, &id)?`.
- Add `"  refresh-epic   Pull default-branch updates into an epic branch"` to the `Epics:` block in the `help_template` string.

**Tests to add** (in `apm-core/src/epic.rs`):
- `epic_is_quiescent` returns empty vec when all epic tickets are in `worker_end` or `terminal` states.
- `epic_is_quiescent` returns a blocker for a ticket whose state is `!terminal && !worker_end`.
- `epic_is_quiescent` detects a live-worker blocker when a `.apm-worker.pid` containing the current process's PID exists in the matched worktree path.

### 1. `apm-core/src/epic.rs` — add `epic_is_quiescent()`

Add a public function:

```rust
pub fn epic_is_quiescent(
    root: &Path,
    epic_id: &str,
    config: &Config,
    worktrees: &[(std::path::PathBuf, String)],  // (wt_path, branch)
) -> Result<Vec<String>>  // returns list of human-readable blocker lines
```

Logic:
1. Load all tickets via `apm_core::ticket::load_all_from_git(root, &config.tickets.dir)?`; filter to those where `t.frontmatter.epic.as_deref() == Some(epic_id)`.
2. For each epic ticket:
   - Look up the `StateConfig` by `t.frontmatter.state`. If the state is **not** `terminal` and **not** `worker_end`, push a blocker: `"  {id} — {title} (state: {state})"`.
   - Find the ticket's worktree by matching `t.frontmatter.branch` (or `ticket_fmt::branch_name_from_path`) against the `worktrees` slice. If found, check `wt_path.join(".apm-worker.pid")`: if it exists and `worker::is_alive(pid)` returns true, push a blocker: `"  {id} — {title} (live worker)"`. Skip the live-worker check if the ticket is already listed from the state check (avoid double-listing the same ticket).
3. Return the blocker `Vec<String>`; empty means quiescent.

The function takes `worktrees` as a parameter so callers can reuse an already-loaded list and it is unit-testable without filesystem access.

### 2. `apm-core/src/github.rs` — add `gh_pr_create_or_update_between()`

The existing `gh_pr_create_or_update` hardcodes `--base <default_branch> --head <branch>`. Add:

```rust
pub fn gh_pr_create_or_update_between(
    root: &Path,
    head: &str,
    base: &str,
    title: &str,
    body: &str,
    messages: &mut Vec<String>,
) -> Result<()>
```

Implementation (mirrors the existing function but with explicit head/base):
1. Check for existing open PR: `gh pr list --head <head> --base <base> --state open --json number --jq .[0].number`.
2. If a PR number is returned, push `"PR #{n} already open ({head} → {base})"` and return `Ok(())`.
3. Otherwise: `gh pr create --base <base> --head <head> --title <title> --body <body>`.
4. On success push the URL; on failure bail with stderr.

Leave the existing `gh_pr_create_or_update` unchanged.

### 3. `apm/src/cmd/epic.rs` — add `run_refresh_epic()`

```rust
pub fn run_refresh_epic(root: &Path, id_arg: &str) -> Result<()>
```

Steps:
1. `CmdContext::load_config_only(root)?`
2. `find_epic_branches(root, id_arg)` → bail on 0 or >1 results (same error messages as `run_close`).
3. `epic_id = epic_id_from_branch(&epic_branch)`.
4. `worktrees = apm_core::worktree::list_ticket_worktrees(root)?`.
5. `blockers = apm_core::epic::epic_is_quiescent(root, epic_id, &config, &worktrees)?`; if non-empty, bail:
   ```
   cannot refresh epic: the following tickets are not quiescent:
   <blocker lines joined with \n>
   ```
6. `default_branch = &config.project.default_branch`.
7. Run `git log --oneline --no-decorate <epic_branch>..<default_branch>` via `apm_core::git_util::run` or a direct `Command`. If the output is empty (trimmed), print `"epic branch is up to date with {default_branch}"` and return `Ok(())`.
8. `pr_title = format!("{epic_id}: refresh from {default_branch}")`.
9. `pr_body` = the raw `--oneline` output.
10. `apm_core::git::push_branch_tracking(root, &epic_branch)?` (ensures `base` exists on remote).
11. `apm_core::github::gh_pr_create_or_update_between(root, default_branch, &epic_branch, &pr_title, &pr_body, &mut messages)?`.
12. Print each message.

### 4. `apm/src/main.rs` — wire up the command

- Add variant to the top-level `Command` enum:
  ```rust
  /// Pull default-branch updates into an epic branch
  RefreshEpic {
      /// Epic ID (4–8 char hex prefix)
      id: String,
  },
  ```
- In the `match command` dispatch block, add:
  ```rust
  Command::RefreshEpic { id } => cmd::epic::run_refresh_epic(&root, &id)?,
  ```
- In the `help_template` string, add under `Epics:`:
  ```
    refresh-epic   Pull default-branch updates into an epic branch
  ```

### Order of changes

1. `epic_is_quiescent` in `apm-core/src/epic.rs` (isolated, testable first).
2. `gh_pr_create_or_update_between` in `apm-core/src/github.rs`.
3. `run_refresh_epic` in `apm/src/cmd/epic.rs`.
4. CLI wiring in `apm/src/main.rs`.

### Tests to add

- `epic_is_quiescent` returns empty vec when all tickets are terminal or `worker_end`.
- `epic_is_quiescent` returns a blocker entry for a ticket in `in_progress` (not terminal, not worker_end).
- `epic_is_quiescent` returns a live-worker blocker when a `.apm-worker.pid` with the current process's PID exists in the worktree path.
- `gh_pr_create_or_update_between` uses correct `--head` and `--base` ordering (verify via a test that inspects the constructed args, or at minimum a doc test confirming the function exists).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T21:07Z | groomed | in_design | philippepascal |