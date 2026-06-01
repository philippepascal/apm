+++
id = "1469175e"
title = "apm refresh-epic --merge: push after local merge so downstream sees the refresh"
state = "in_progress"
priority = 7
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1469175e-apm-refresh-epic-merge-push-after-local-"
created_at = "2026-05-31T03:26:11.802159Z"
updated_at = "2026-06-01T07:36:42.980163Z"
+++

## Spec

### Problem

`apm refresh-epic --merge` merges the default branch into the epic worktree locally but does not push to origin. The dispatch path in `apm start` calls `remote_branch_tip`, which prefers `origin/<epic-branch>` when that ref exists. Any ticket dispatched after a local-only merge therefore receives the pre-merge epic content. The refresh is silently ineffective for all downstream workers until the supervisor pushes manually.

This asymmetry was confirmed in practice on the syn project: `apm refresh-epic <id> --merge` completed successfully, but a subsequent `apm start` on a ticket in that epic dispatched from the stale `origin/<epic-branch>` tip. The `--pr` path (the `else` branch of `run_refresh_epic` in `apm/src/cmd/epic.rs`) already calls `push_branch_tracking` before opening the PR; the `--merge` path has no equivalent step.

### Acceptance criteria

- [x] `apm refresh-epic <id> --merge --push` pushes the epic branch to origin after a successful local merge; `git rev-parse origin/<epic-branch>` equals the post-merge local tip.
- [ ] `apm refresh-epic <id> --merge --no-push` completes the local merge without pushing; `origin/<epic-branch>` is unchanged; a warning is printed to stderr stating that downstream `apm start` will read stale content until the branch is pushed manually.
- [ ] `apm refresh-epic <id> --merge` with stdout connected to a terminal prompts `Push refreshed epic to origin? [Y/n]`; pressing Enter or typing `y`/`Y` pushes; typing `n`/`N` skips with the stale-origin warning.
- [ ] `apm refresh-epic <id> --merge` with stdout not connected to a terminal skips the push without prompting and prints the stale-origin warning to stderr.
- [ ] When the local merge fails with a conflict, no push is attempted regardless of the `--push`/`--no-push` flags.
- [ ] Passing both `--push` and `--no-push` together is rejected as a CLI error.
- [ ] The `--pr` path behaviour is unchanged: `push_branch_tracking` still runs before PR creation.
- [ ] The default path (no `--merge`, `--pr`, or `--auto` flag) behaviour is unchanged.

### Out of scope

- Cascading default-branch updates into individual in-flight ticket worktrees after an epic refresh — separate, larger concern; file as its own ticket
- Changes to the dispatch-time merge logic in `apm-core/src/start.rs` (`remote_branch_tip`'s origin-preference) — separate design decision
- `--push`/`--no-push` flags on the `--pr` path (`--pr` always pushes before creating a PR; no change needed)
- Explicit push-flag support for the `--auto` path — the default prompt/warn logic applies when `--auto` resolves to a local merge

### Approach

#### `apm/src/main.rs` — add flags to `RefreshEpic`

Add two new fields to the `RefreshEpic` variant (after `auto_mode`):

```rust
/// Push the merged epic branch to origin without prompting
#[arg(long, conflicts_with = "no_push")]
push: bool,
/// Skip pushing after merge and print a warning, without prompting
#[arg(long = "no-push", conflicts_with = "push")]
no_push: bool,
```

Update the dispatch match arm:

```rust
Command::RefreshEpic { id, merge, pr, auto_mode, push, no_push } =>
    cmd::epic::run_refresh_epic(&root, &id, merge, pr, auto_mode, push, no_push),
```

#### `apm/src/util.rs` — add `prompt_yes_no_default_yes`

Add alongside the existing `prompt_yes_no`. Returns `true` unless the user types `n`/`N`; empty input (Enter) returns `true`.

```rust
/// Print `prompt`, flush stdout, read one line; returns true unless the user types "n" (default yes).
pub fn prompt_yes_no_default_yes(prompt: &str) -> io::Result<bool> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(!line.trim().eq_ignore_ascii_case("n"))
}
```

#### `apm/src/cmd/epic.rs` — update `run_refresh_epic`

Change the signature to accept two new booleans:

```rust
pub fn run_refresh_epic(
    root: &Path, id_arg: &str, merge: bool, pr: bool,
    auto_mode: bool, push: bool, no_push: bool,
) -> Result<()>
```

After the successful `merge_ref` branch (following the `None =>` conflict-bail arm in `run_refresh_epic`'s `do_merge` block), insert the push decision:

```rust
let should_push = if push {
    true
} else if no_push {
    false
} else if std::io::stdout().is_terminal() {
    crate::util::prompt_yes_no_default_yes("Push refreshed epic to origin? [Y/n] ")?
} else {
    false
};

if should_push {
    apm_core::git::push_branch_tracking(root, &epic_branch)?;
    println!("pushed {epic_branch} to origin");
} else {
    eprintln!(
        "warning: {epic_branch} was not pushed; \
         downstream `apm start` will read stale origin content until pushed manually"
    );
}
```

The warning fires whenever the push is skipped, including when the user explicitly passes `--no-push`, so the consequence is always visible.

#### Tests — `apm/tests/integration.rs`

Add three test cases. Each sets up a temp git repo with a bare remote (so actual pushes work), creates an epic branch, and puts a commit on the default branch that the epic branch is behind.

1. **`--push`**: call `run_refresh_epic` with `push=true`. Assert `git rev-parse origin/<epic-branch>` equals the post-merge local tip.
2. **`--no-push`**: call with `no_push=true`. Assert `origin/<epic-branch>` still points at the pre-merge tip, and that stderr contains `"was not pushed"`.
3. **Non-interactive default**: call with `push=false, no_push=false` (stdout is not a terminal in test context). Assert same as `--no-push`: origin unchanged, warning on stderr.

### Open questions


### Amendment requests

- [x] Line number drift: the spec cites apm-core/src/start.rs lines 454-458 for the dispatch-preference logic that reads origin/<branch>, but the actual location after recent edits is around lines 481-485 (the remote_branch_tip preference branch in start.rs::run). Either update the line references, or describe the location by symbol (the remote_branch_tip preference branch in start.rs::run) so future drift does not invalidate the spec.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T03:26Z | — | new | philippepascal |
| 2026-06-01T02:52Z | new | groomed | philippepascal |
| 2026-06-01T02:53Z | groomed | in_design | philippepascal |
| 2026-06-01T02:57Z | in_design | specd | claude |
| 2026-06-01T03:06Z | specd | ammend | philippepascal |
| 2026-06-01T07:02Z | ammend | in_design | philippepascal |
| 2026-06-01T07:04Z | in_design | specd | claude |
| 2026-06-01T07:36Z | specd | ready | philippepascal |
| 2026-06-01T07:36Z | ready | in_progress | philippepascal |