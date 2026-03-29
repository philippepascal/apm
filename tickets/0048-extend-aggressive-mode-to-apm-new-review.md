+++
id = 48
title = "Extend aggressive mode to apm new, review, and take"
state = "in_design"
priority = 2
effort = 3
risk = 1
author = "claude-0328-c72b"
branch = "ticket/0048-extend-aggressive-mode-to-apm-new-review"
created_at = "2026-03-28T19:50:06.625320Z"
updated_at = "2026-03-29T20:36:41.219896Z"
+++

## Spec

### Problem

Aggressive mode (fetch-before-read, push-after-write) is implemented in `show`,
`state`, `start`, and `sync`. Three write commands that touch ticket branches are
not covered:

- **`apm new`** — creates a ticket and commits it to a new branch; never pushes,
  so the branch only exists locally until manually synced
- **`apm review`** — reads a ticket (should fetch first), opens `$EDITOR`, commits
  on save, and auto-resolves the state transition; the fetch before opening the
  editor is missing
- **`apm take`** — claims a ticket by writing to its branch; no push afterwards

None of these commands have a `--no-aggressive` escape hatch either.

### Acceptance criteria

- [ ] `apm new` pushes the new ticket branch after creating it when
  `sync.aggressive = true`
- [ ] `apm new --context` (ticket #58) follows the same path — it creates the
  same branch, so the same push applies
- [ ] `apm review` fetches the ticket branch before opening `$EDITOR` when
  `sync.aggressive = true`; the post-edit push happens via the internal
  `apm state` call triggered on save (no change needed for the push side)
- [ ] `apm take` pushes the ticket branch after claiming it when
  `sync.aggressive = true`
- [ ] All three commands accept a `--no-aggressive` flag to opt out
- [ ] If fetch or push fails, a warning is printed and the command continues
  (same fail-soft pattern as existing commands)

### Out of scope

- `apm next`, `apm list`, `apm verify`, `apm agents` — read-only, aggressive
  mode not applicable
- `apm worktrees` — manages worktrees, not ticket branches
- Changing `apm sync`'s aggressive behaviour

### Approach

**`apm/src/cmd/new.rs`**: add `no_aggressive: bool` parameter; after committing
the ticket to its branch, push when aggressive:
```rust
if aggressive {
    if let Err(e) = git::push_branch(root, &branch) {
        eprintln!("warning: push failed: {e:#}");
    }
}
```
This covers both the plain `apm new` and `apm new --context` paths since both
go through the same branch-creation and commit step.

**`apm/src/cmd/review.rs`** (redesigned per ticket #57 — opens `$EDITOR`,
commits on save, auto-resolves state transition): add `no_aggressive: bool`
parameter; at the top of `run()`, before reading the ticket for display in the
editor, fetch when aggressive:
```rust
if aggressive {
    if let Err(e) = git::fetch_branch(root, &branch) {
        eprintln!("warning: fetch failed: {e:#}");
    }
}
```
The post-save push is handled by the internal `apm state` call, which already
pushes in aggressive mode — no additional change needed there.

**`apm/src/cmd/take.rs`**: add `no_aggressive: bool` parameter; after writing
the updated ticket to the branch, push when aggressive.

**`apm/src/main.rs`**: add `#[arg(long)] no_aggressive: bool` to the `New`,
`Review`, and `Take` subcommand variants; pass through to each `run` function.

### Amendment requests

- [x] `apm review` is being redesigned (see TICKET-LIFECYCLE): it will open
  `$EDITOR`, commit on save, and auto-resolve the state transition (no `--to`
  flag). The aggressive fetch is still needed, but the current approach section
  references the old `apm review` implementation. Update the approach to
  describe where the fetch fits in the redesigned command.
- [x] `apm new --context` is a new variant of `apm new` (separate ticket).
  If aggressive push for `apm new` is addressed here, ensure the `--context`
  path is also covered (it creates the same branch, the same push applies).

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T19:50Z | — | new | claude-0328-c72b |
| 2026-03-28T19:54Z | new | specd | claude-0328-c72b |
| 2026-03-29T19:11Z | specd | ammend | claude-0329-1200-a1b2 |
| 2026-03-29T20:36Z | ammend | in_design | claude-0329-main |
