+++
id = "ba6da9cb"
title = "apm sync incorrectly mentions main instead of epic name in error message for missing merge"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ba6da9cb-apm-sync-incorrectly-mentions-main-inste"
created_at = "2026-06-09T21:47:31.578694Z"
updated_at = "2026-06-12T07:58:00.596811Z"
+++

## Spec

### Problem

When `apm sync` detects a ticket that appears to be in an `implemented`-equivalent state but whose branch has not been detected as merged, it emits a hint telling the user the ticket was not merged into `main`. This hint hardcodes the word "main" regardless of what the actual target branch is.

A ticket can target a branch other than the project default — most commonly an epic branch set via `target_branch` in the frontmatter. When such a ticket is unmerged, the hint is misleading: it points the user at `main` when the real merge target is, for example, `epic/abc-user-auth`. Even for tickets with no `target_branch`, the project default branch is configurable and may not be called `main`.

The fix is localised to the hint-generation block in `apm-core/src/sync.rs`.

### Acceptance criteria

- [ ] When a ticket has no `target_branch` and its branch is unmerged, the hint names the project's configured default branch (e.g. `develop`, `trunk`) rather than the literal string `main`.
- [ ] When a ticket has a `target_branch` set (e.g. an epic branch like `epic/abc-user-auth`) and its branch is unmerged, the hint names that `target_branch` instead of `main`.
- [ ] The hint message text is otherwise unchanged: it still includes the ticket id and the `apm state <id> closed` recovery command.
- [ ] All existing `cargo test --workspace` tests continue to pass.

### Out of scope

- Changing any other sync hint or error message that may also reference branch names.
- Adding new hint messages for missing-merge scenarios beyond what already exists.
- Changes to `sync_guidance.rs` constants (they use `<default>` placeholders and are not involved in this hint path).

### Approach

**File:** `apm-core/src/sync.rs`, hint-generation loop (~line 149–168).

**Change:** In the loop that builds unmerged-implemented hints, compute the effective target branch before formatting the string, then use it in place of the hardcoded `"main"`:

```rust
// Before
hints.push(format!(
    "ticket #{id} is in `implemented` state but its branch was not detected as merged into \
     main. If it was already merged, close it manually: apm state {id} closed"
));

// After
let target = t.frontmatter.target_branch.as_deref()
    .filter(|s| !s.is_empty())
    .unwrap_or(default_branch);
hints.push(format!(
    "ticket #{id} is in `implemented` state but its branch was not detected as merged into \
     {target}. If it was already merged, close it manually: apm state {id} closed"
));
```

`default_branch` is already bound earlier in `detect()` as `let default_branch = &config.project.default_branch;` and is in scope at the hint-generation loop.

**Tests:** Add a unit test in `apm-core/src/sync.rs` (or `apm/tests/integration.rs`) that:
1. Sets up a repo where `default_branch` is not `main` (e.g. `trunk`).
2. Creates an implemented ticket with no `target_branch`.
3. Calls `detect()` and asserts the resulting hint contains `trunk`, not `main`.
4. Repeats with a ticket whose `target_branch` is `epic/abc-feat` and asserts the hint names `epic/abc-feat`.

No other files change.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-09T21:47Z | — | new | philippepascal |
| 2026-06-12T07:52Z | new | groomed | philippepascal |
| 2026-06-12T07:58Z | groomed | in_design | philippepascal |