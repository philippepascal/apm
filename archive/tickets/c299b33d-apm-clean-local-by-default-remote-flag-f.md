+++
id = "c299b33d"
title = "apm clean: local by default, --remote flag for old branch cleanup"
state = "closed"
priority = 0
effort = 4
risk = 3
author = "apm"
agent = "3805"
branch = "ticket/c299b33d-apm-clean-local-by-default-remote-flag-f"
created_at = "2026-04-02T20:44:35.825711Z"
updated_at = "2026-04-02T22:27:01.700953Z"
+++

## Spec

### Problem

`apm clean` currently removes both worktrees and local branches in a single operation, conflating two concerns with very different frequency and risk profiles. Worktree cleanup is a routine local housekeeping task done regularly after tickets close; branch deletion (local or remote) is rarer and carries more consequence.

The current default is too aggressive: deleting local branches removes the offline reference to merged work and requires a network round-trip to recover. More critically, there is no supported path to delete **remote** branches at all — accumulated `ticket/*` branches on origin grow indefinitely.

The fix is to split `apm clean` into three explicitly opt-in levels:

1. **Worktree removal** (default, no flags): remove the worktree directory under `apm--worktrees/` for each terminal-state ticket. No branch is touched. This is the safe, high-frequency operation.

2. **Local branch removal** (`--branches`): also delete the local `ticket/*` branch. Safe because the content is already on origin, but kept opt-in since losing local refs is annoying when offline.

3. **Remote branch removal** (`--remote --older-than <threshold>`): delete `ticket/*` branches from origin that are in a terminal state and whose last commit predates the given threshold. Requires an explicit age guard to prevent accidental mass deletion.

A fourth flag, `--untracked`, extends worktree removal to cover worktrees that contain untracked non-temp files (build artifacts, etc.) that currently cause a skip-with-warning. Without `--untracked`, only the known-temp files (`.apm-worker.pid`, `.apm-worker.log`, etc.) are auto-removed; all other untracked files block removal with a warning.

### Acceptance criteria

- [x] **Default behavior (worktrees only):**
- [x] `apm clean` removes the worktree for each terminal-state ticket that has one
- [x] `apm clean` does not delete any local branch
- [x] `apm clean --dry-run` lists worktrees that would be removed and exits without modifying anything
- [x] `apm clean --dry-run` does not list any local branch for deletion

- [x] **With `--branches`:**
- [x] `apm clean --branches` removes worktrees and deletes local `ticket/*` branches for terminal-state tickets
- [x] `apm clean --branches` prunes the corresponding `origin/<branch>` remote-tracking ref after deleting the local branch (to prevent re-creation on next `apm sync`)
- [x] `apm clean --branches --dry-run` lists both worktrees and local branches that would be removed

- [x] **With `--remote --older-than`:**
- [x] `apm clean --remote --older-than 30d` deletes remote `ticket/*` branches in terminal states whose last commit is older than 30 days
- [x] `apm clean --remote --older-than 2026-01-01` accepts ISO date (`YYYY-MM-DD`) as the threshold
- [x] `apm clean --remote` (without `--older-than`) exits with a non-zero status and an error message stating `--older-than` is required
- [x] `--older-than` without `--remote` exits with a non-zero status and an error message stating it requires `--remote`
- [x] `apm clean --remote --older-than 30d` only removes branches whose ticket is in a terminal state; non-terminal or non-ticket branches are never touched
- [x] `apm clean --remote --older-than 30d --yes` skips per-branch confirmation prompts
- [x] `apm clean --remote --older-than 30d --dry-run` lists remote branches that would be deleted without modifying anything

- [x] **With `--untracked`:**
- [x] `apm clean --untracked` removes a worktree that has only untracked non-temp files by deleting those files first, then removing the worktree
- [x] `apm clean` (without `--untracked`) prints a warning for any worktree with untracked non-temp files and leaves it in place
- [x] `apm clean --untracked` still skips a worktree that has modified tracked files, printing a warning

- [x] **Invariants:**
- [x] Remote branches are never deleted unless `--remote` is explicitly passed
- [x] Known-temp files (`.apm-worker.pid`, `.apm-worker.log`, `pr-body.md`, `body.md`, `ac.txt`) are auto-removed in all modes without requiring `--untracked`

### Out of scope

- Deleting the ticket file from the `main` branch (that is `apm close`)
- Pruning non-`ticket/*` remote branches
- Any changes to `apm close` behavior
- Configuring default flags via `apm.toml` (e.g. making `--branches` the default per-project)
- Recovering or archiving branches before deletion
- `--remote` without a ticket-state lookup (e.g. deleting any stale remote branch regardless of whether it has a ticket)

### Approach

**Decision: local branch deletion becomes opt-in.** The default `apm clean` no longer deletes local branches. `--branches` is the new opt-in flag. This is a deliberate behavior change; no backward-compat shim is needed.

#### 1. `apm/src/main.rs` — CLI flags

Add to the `Clean` variant:

```rust
/// Also delete local ticket/* branches (default: worktrees only)
#[arg(long)]
branches: bool,

/// Delete remote ticket/* branches in terminal states older than --older-than
#[arg(long)]
remote: bool,

/// Age threshold for --remote: e.g. "30d" or "2026-01-01" (YYYY-MM-DD)
#[arg(long, value_name = "THRESHOLD", requires = "remote")]
older_than: Option<String>,

/// Skip per-branch confirmation prompts for --remote
#[arg(long)]
yes: bool,

/// Remove untracked non-temp files from worktrees before removal
#[arg(long)]
untracked: bool,
```

Use clap's `requires = "remote"` on `older_than` and validate in the handler that `--remote` requires `--older-than` (exit with error if missing). Update the `long_about` docstring to describe all modes.

#### 2. `apm-core/src/clean.rs` — core logic

**`remove()` signature change:**

```rust
pub fn remove(root: &Path, candidate: &CleanCandidate, force: bool, remove_branches: bool) -> Result<()>
```

The local-branch and remote-tracking-ref deletion blocks gate on `remove_branches`. All callers in `apm/src/cmd/clean.rs` pass the value of the `--branches` flag.

**`--untracked` handling in `candidates()`:**

The function gains an `untracked: bool` parameter. In the dirty-worktree branch (where `wt_clean` is false):

- Current: if `force && modified_tracked.is_empty()` → add to candidates
- New: additionally, if `untracked && modified_tracked.is_empty()` → run `remove_untracked(path, &diagnosis.other_untracked)` then add to candidates with `force: false` (worktree is now clean)

Known-temp files continue to be removed unconditionally (no change to existing `known_temp` logic).

**New type + function: `RemoteCandidate` and `remote_candidates()`**

```rust
pub struct RemoteCandidate {
    pub branch: String,
    pub last_commit: chrono::DateTime<chrono::Utc>,
}

pub fn remote_candidates(
    root: &Path,
    config: &Config,
    older_than: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<RemoteCandidate>>
```

Steps:
1. Build terminal-state set from config.
2. Call `git::remote_ticket_branches_with_dates(root)` → `Vec<(branch_name, commit_date)>`.
3. For each branch older than `older_than`, load ticket state via `ticket::state_from_branch(root, default_branch, path)`.
4. Keep only those in terminal states; return as `Vec<RemoteCandidate>`.

**`--older-than` parsing** (in `apm/src/cmd/clean.rs`):

- Ends with `d`: parse integer N, subtract N days from `Utc::now()`
- Otherwise: parse as `NaiveDate` with format `%Y-%m-%d`, convert to `DateTime<Utc>`
- Anything else: exit with a clear error message

#### 3. `apm-core/src/git.rs` — new functions

`remote_ticket_branches_with_dates(root: &Path) -> Result<Vec<(String, DateTime<Utc>)>>`: runs `git for-each-ref refs/remotes/origin/ticket/ --format='%(refname:short) %(creatordate:unix)'`, strips the `origin/` prefix from each branch name, parses the Unix timestamp.

`delete_remote_branch(root: &Path, branch: &str) -> Result<()>`: runs `git push origin --delete <branch>`, returns error on non-zero exit.

#### 4. `apm/src/cmd/clean.rs` — handler updates

- Thread `--branches` into `remove()`.
- Thread `--untracked` into `candidates()`.
- Add `--remote` code path after worktree/branch cleanup:
  1. Parse `--older-than` (required when `--remote`) into `DateTime<Utc>`.
  2. Call `clean::remote_candidates(root, &config, threshold)`.
  3. For each candidate: prompt (unless `--yes`) then call `git::delete_remote_branch()`.
  4. `--dry-run` prints without acting.

#### 5. Tests

- Unit tests in `apm-core/src/clean.rs`: `--older-than` parsing (`30d`, `2026-01-01`, invalid), remote candidate filtering by age and state.
- Integration tests in `apm/tests/integration.rs`: verify `apm clean` leaves local branches intact; verify `apm clean --branches` removes them.

#### Order of changes

1. Add `delete_remote_branch` and `remote_ticket_branches_with_dates` to `git.rs`
2. Update `clean.rs` (`remove`, `candidates`, add `remote_candidates`, `RemoteCandidate`)
3. Update `main.rs` CLI flags
4. Update `cmd/clean.rs` handler
5. Update tests

### Decision: local branch deletion becomes opt-in

The default `apm clean` no longer deletes local branches. `--branches` is the new opt-in flag for that. This is a behavior change; no backward-compat shim is needed.

---

### 1. `apm/src/main.rs` — CLI flags

Add to the `Clean` variant:

```rust
/// Also delete local ticket/* branches (default: worktrees only)
#[arg(long)]
branches: bool,

/// Delete remote ticket/* branches in terminal states older than --older-than
#[arg(long)]
remote: bool,

/// Age threshold for --remote: e.g. "30d" or "2026-01-01" (YYYY-MM-DD)
#[arg(long, value_name = "THRESHOLD", requires = "remote")]
older_than: Option<String>,

/// Remove untracked non-temp files from worktrees before removal
#[arg(long)]
untracked: bool,
```

Use clap's `requires = "remote"` on `older_than` and validate in the handler that `--remote` requires `--older-than` (exit with error if missing). Update the `long_about` docstring to describe all modes.

---

### 2. `apm-core/src/clean.rs` — core logic

**`remove()` signature change:**

```rust
pub fn remove(root: &Path, candidate: &CleanCandidate, force: bool, remove_branches: bool) -> Result<()>
```

The local-branch and remote-tracking-ref deletion blocks are now gated on `remove_branches`. All callers in `apm/src/cmd/clean.rs` pass the value of the `--branches` flag.

**`--untracked` handling in `candidates()`:**

The function gains an `untracked: bool` parameter. In the dirty-worktree branch (where `wt_clean` is false):

- Current: if `force && modified_tracked.is_empty()` → add to candidates (git worktree remove --force handles files)
- New: additionally, if `untracked && modified_tracked.is_empty()` → run `remove_untracked(path, &diagnosis.other_untracked)` then add to candidates with `worktree` set and `force: false` (worktree is now clean)

Known-temp files continue to be removed unconditionally (existing `remove_untracked` call for `known_temp`).

**New function: `remote_candidates()`**

```rust
pub fn remote_candidates(
    root: &Path,
    config: &Config,
    older_than: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<RemoteCandidate>>
```

Where `RemoteCandidate` holds `branch: String` and `last_commit: DateTime<Utc>`.

Steps:
1. Build terminal-state set from config (same as `candidates()`).
2. Call `git::remote_ticket_branches_with_dates(root)` → `Vec<(branch_name, commit_date)>`.
3. For each branch older than `older_than`, load ticket state via `ticket::state_from_branch(root, default_branch, path)`.
4. Keep only those in terminal states.
5. Return as `Vec<RemoteCandidate>`.

**`--older-than` parsing** (inline in `apm/src/cmd/clean.rs`):

- If value ends with `d`: parse integer, subtract N days from `Utc::now()`
- Otherwise: parse as `NaiveDate` with `chrono::NaiveDate::parse_from_str(..., "%Y-%m-%d")`, convert to `DateTime<Utc>`
- Reject anything else with a clear error message

---

### 3. `apm-core/src/git.rs` — new functions

**`remote_ticket_branches_with_dates(root: &Path) -> Result<Vec<(String, DateTime<Utc>)>>`**

```
git for-each-ref refs/remotes/origin/ticket/ \
    --format=%(refname:short) %(creatordate:unix)
```

Parse each line: strip `origin/` prefix for the branch name, parse Unix timestamp as `DateTime<Utc>`.

**`delete_remote_branch(root: &Path, branch: &str) -> Result<()>`**

```
git push origin --delete <branch>
```

Return error if the command fails (non-zero exit).

---

### 4. `apm/src/cmd/clean.rs` — handler updates

- Thread `--branches` into `remove()`.
- Thread `--untracked` into `candidates()`.
- Add a new `--remote` code path after the existing worktree/branch cleanup:
  1. Parse `--older-than` into `DateTime<Utc>`.
  2. Call `clean::remote_candidates(root, &config, threshold)`.
  3. For each candidate: prompt (unless `--yes`) then call `git::delete_remote_branch()`.
  4. `--dry-run` prints without acting.

---

### 5. Tests

- Unit tests in `apm-core/src/clean.rs`: add tests for `--older-than` parsing, remote candidate filtering by age and state.
- Integration tests in `apm/tests/integration.rs`: add a test that `apm clean` leaves local branches intact, and a separate test that `apm clean --branches` removes them.

---

### Order of changes

1. Add `delete_remote_branch` and `remote_ticket_branches_with_dates` to `git.rs`
2. Update `clean.rs` (modify `remove`, `candidates`; add `remote_candidates`, `RemoteCandidate`)
3. Update `main.rs` CLI flags
4. Update `cmd/clean.rs` handler
5. Update tests

### Open questions


### Amendment requests

- [x] Add `--yes` flag to the CLI flags snippet in the Approach (`main.rs` section): `#[arg(long)] yes: bool` with a docstring like "Skip per-branch confirmation prompts for --remote". The AC requires it but the approach omits it.
- [x] Remove the duplicate approach content. The Approach section already contains the full plan under `####` subsections; the identical content is repeated as top-level `###` sections ("### Decision", "### 1.", "### 2.", etc.) after the Approach section. Delete the duplicates, keeping only the `### Approach` version.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:44Z | — | new | apm |
| 2026-04-02T20:50Z | new | groomed | apm |
| 2026-04-02T20:50Z | groomed | in_design | philippepascal |
| 2026-04-02T20:55Z | in_design | specd | claude-0402-2100-b7f3 |
| 2026-04-02T21:11Z | specd | ammend | apm |
| 2026-04-02T21:11Z | ammend | in_design | apm |
| 2026-04-02T21:12Z | in_design | specd | apm |
| 2026-04-02T21:12Z | specd | ammend | apm |
| 2026-04-02T21:13Z | ammend | in_design | philippepascal |
| 2026-04-02T21:15Z | in_design | specd | claude-0402-2115-f3a2 |
| 2026-04-02T21:17Z | specd | ready | apm |
| 2026-04-02T21:19Z | ready | in_progress | philippepascal |
| 2026-04-02T21:44Z | in_progress | implemented | claude-0402-2130-x7k2 |
| 2026-04-02T22:27Z | implemented | closed | apm-sync |