+++
id = "5cf54181"
title = "Detect mid-merge state and share guidance strings"
state = "in_progress"
priority = 0
effort = 3
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5cf54181-detect-mid-merge-state-and-share-guidanc"
created_at = "2026-04-17T18:32:40.602264Z"
updated_at = "2026-04-17T18:48:49.765741Z"
epic = "47375a6a"
target_branch = "epic/47375a6a-safer-apm-sync"
+++

## Spec

### Problem

Two supporting concerns shared across the other sync tickets:

1. **Mid-merge state is undetected.** If the user runs `apm sync` while the repo is in a mid-merge, mid-rebase, or mid-cherry-pick state (e.g. `.git/MERGE_HEAD` exists), sync's attempts to fast-forward or merge will compound the mess. Sync should detect this state at the top of the flow and bail with clear guidance ("finish or abort first").

2. **Guidance strings are scattered.** Tickets A and B both need copy-pasteable recovery instructions for scenarios sync cannot auto-handle (dirty-overlap FF, diverged main, diverged ticket/epic branch, mid-merge repo). Having these strings defined once in a small module keeps wording consistent and makes future tweaks single-point.

This ticket provides the mid-merge detection and the shared guidance-strings module that tickets A and B consume. It lands first in sequence but is small in scope.

See `/Users/philippepascal/Documents/apm/apm-sync-scenarios.md` — particularly the "Dirty-tree edge cases" and "Guidance copy" sections — for the full list of messages and their triggers. Implementers must add comments explaining when each guidance string fires.

### Acceptance criteria

- [x] A helper `detect_mid_merge_state(root) -> Option<MidMergeState>` exists in `apm-core/src/git_util.rs` (or a new module) and returns `Some` when the repo is in any of: mid-merge, mid-rebase (merge or apply), mid-cherry-pick
- [x] `apm sync` calls this helper at the top of its flow. When a mid-state is detected, sync prints the "mid-merge" guidance and exits with a success status without performing fetch, ref updates, or close detection
- [x] A single module (e.g. `apm-core/src/sync_guidance.rs`) holds all copy-pasteable guidance strings used by the sync flow, keyed by case:
- [x] - `MAIN_BEHIND_DIRTY_OVERLAP` (for ticket A)
- [x] - `MAIN_DIVERGED_CLEAN` / `MAIN_DIVERGED_DIRTY` (for ticket A)
- [x] - `TICKET_OR_EPIC_DIVERGED` (for ticket B)
- [x] - `MID_MERGE_IN_PROGRESS` (for this ticket)
- [ ] Each guidance string is exposed as a public constant or `const fn`; callers reference by name, not literal
- [ ] The module has comments describing each string's trigger condition
- [ ] Unit tests cover mid-state detection for: clean repo (None), mid-merge (Some), mid-rebase-merge (Some), mid-rebase-apply (Some), mid-cherry-pick (Some)
- [ ] `cargo test --workspace` passes

### Out of scope

- Actually consuming the guidance strings from tickets A and B — each of those tickets wires its own call sites; this ticket only provides the module
- Any form of automatic resolution of a mid-merge / mid-rebase state
- Detecting other "incomplete" states like un-applied stash, bisect in progress, submodule-related mid-states — scope is limited to merge/rebase/cherry-pick
- Runtime translation / i18n of guidance strings — plain ASCII, English only
- Emitting the mid-merge guidance from commands other than `apm sync` (if desired later, separate ticket)

### Approach

**1. New module `apm-core/src/sync_guidance.rs`.** Holds named `&'static str` constants for each guidance case. Keep them as plain multi-line string literals — no templating. Callers `println!` or `eprintln!` them directly.

```rust
// Guidance strings printed by `apm sync` when automatic handling is unsafe.
// Each constant's doc comment describes the precise trigger condition so
// callers can reference by name rather than guessing from wording.

/// Printed when local <default> is behind origin/<default> (FF possible in principle)
/// but `git merge --ff-only` refused because uncommitted local changes would be overwritten.
pub const MAIN_BEHIND_DIRTY_OVERLAP: &str = "...";

/// Printed when local <default> and origin/<default> have diverged and the working tree is clean.
pub const MAIN_DIVERGED_CLEAN: &str = "...";

/// Printed when local <default> and origin/<default> have diverged and the working tree is dirty.
pub const MAIN_DIVERGED_DIRTY: &str = "...";

/// Printed for a non-checked-out ticket/* or epic/* ref whose local tip and origin tip
/// have diverged (local has unpushed commits AND origin has commits not on local).
pub const TICKET_OR_EPIC_DIVERGED: &str = "...";

/// Printed when `apm sync` detects the repo is mid-merge, mid-rebase, or mid-cherry-pick.
pub const MID_MERGE_IN_PROGRESS: &str = "...";
```

Use the exact string bodies from the design doc's "Guidance copy" section (`/Users/philippepascal/Documents/apm/apm-sync-scenarios.md`), with `<default>`, `<id>`, `<slug>` left as literal placeholders inside the strings — each caller substitutes via `replace(...)` at print time. Keep the substitution logic at the call site, not inside this module (the module stays purely declarative).

Export the module from `apm-core/src/lib.rs` as `pub mod sync_guidance;`.

**2. Mid-merge detection helper.** In `apm-core/src/git_util.rs`:

```rust
// Detect whether the repo is in a mid-merge, mid-rebase, or mid-cherry-pick state.
// Presence of any of the marker files/dirs is definitive — git creates them
// for the duration of the operation and removes them on commit/abort.
pub enum MidMergeState {
    Merge,
    RebaseMerge,
    RebaseApply,
    CherryPick,
}

pub fn detect_mid_merge_state(root: &Path) -> Option<MidMergeState> {
    let git_dir = root.join(".git");
    if git_dir.join("MERGE_HEAD").exists()        { return Some(MidMergeState::Merge); }
    if git_dir.join("rebase-merge").is_dir()      { return Some(MidMergeState::RebaseMerge); }
    if git_dir.join("rebase-apply").is_dir()      { return Some(MidMergeState::RebaseApply); }
    if git_dir.join("CHERRY_PICK_HEAD").exists()  { return Some(MidMergeState::CherryPick); }
    None
}
```

Keep it path-based — no subprocess calls. Comment notes that worktrees use a different `.git` location (a file, not a directory), but `apm sync` runs at the repo root where `.git` is always a directory, so this is safe.

**3. Wire into `apm/src/cmd/sync.rs`.** At the top of `run()`, before the fetch:

```rust
if let Some(state) = git::detect_mid_merge_state(root) {
    // Any sync work done in this state would compound the mess.
    // Bail cleanly; let the user resolve the pending operation first.
    eprintln!("{}", apm_core::sync_guidance::MID_MERGE_IN_PROGRESS);
    return Ok(());
}
```

**4. Tests.** Unit tests in `apm-core/src/git_util.rs` (or `tests/` if the helper ends up there):
- `detect_mid_merge_none_on_clean_repo`
- `detect_mid_merge_on_merge_head` — touch `.git/MERGE_HEAD`, assert `Some(Merge)`
- `detect_mid_merge_on_rebase_merge` — mkdir `.git/rebase-merge`
- `detect_mid_merge_on_rebase_apply` — mkdir `.git/rebase-apply`
- `detect_mid_merge_on_cherry_pick` — touch `.git/CHERRY_PICK_HEAD`

Integration test in `apm/tests/integration.rs`:
- `sync_bails_on_mid_merge_state` — put a temp repo into mid-merge, run sync, assert guidance printed and no fetch attempted (use a bare-repo origin and verify its refs untouched)

**5. Comments.** This module is the "single source of guidance wording" — a short header comment at the top of `sync_guidance.rs` explaining that rule keeps future contributors from sprinkling literals back through the sync flow.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T18:32Z | — | new | philippepascal |
| 2026-04-17T18:33Z | new | groomed | claude-0417-1645-sync1 |
| 2026-04-17T18:34Z | groomed | in_design | claude-0417-1645-sync1 |
| 2026-04-17T18:42Z | in_design | specd | claude-0417-1645-sync1 |
| 2026-04-17T18:48Z | specd | ready | apm |
| 2026-04-17T18:48Z | ready | in_progress | philippepascal |