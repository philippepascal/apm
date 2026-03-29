+++
id = 48
title = "Extend aggressive mode to apm new, review, and take"
state = "ammend"
priority = 2
effort = 3
risk = 1
author = "claude-0328-c72b"
branch = "ticket/0048-extend-aggressive-mode-to-apm-new-review"
created_at = "2026-03-28T19:50:06.625320Z"
updated_at = "2026-03-29T19:29:02.129433Z"
+++

## Spec

### Problem

Aggressive mode (fetch-before-read, push-after-write) is implemented in `show`,
`state`, `start`, and `sync`. Three write commands that touch ticket branches are
not covered:

- **`apm new`** — creates a ticket and commits it to a new branch; never pushes,
  so the branch only exists locally until manually synced
- **`apm review`** — reads a ticket (should fetch first), edits it, then calls
  `apm state` internally (which does push in aggressive mode); the fetch before
  display is missing
- **`apm take`** — claims a ticket by writing to its branch; no push afterwards

None of these commands have a `--no-aggressive` escape hatch either.

### Acceptance criteria

- [ ] `apm new` pushes the new ticket branch after creating it when
  `sync.aggressive = true`
- [ ] `apm review` fetches the ticket branch before opening the editor when
  `sync.aggressive = true`; the post-edit push already happens via the internal
  `apm state` call (no change needed there)
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

**`apm/src/cmd/review.rs`**: add `no_aggressive: bool` parameter; before opening
the editor, fetch the ticket branch when aggressive:
```rust
if aggressive {
    if let Err(e) = git::fetch_branch(root, &branch) {
        eprintln!("warning: fetch failed: {e:#}");
    }
}
```

**`apm/src/cmd/take.rs`**: add `no_aggressive: bool` parameter; after writing
the updated ticket to the branch, push when aggressive.

**`apm/src/main.rs`**: add `#[arg(long)] no_aggressive: bool` to the `New`,
`Review`, and `Take` subcommand variants; pass through to each `run` function.

### Amendment requests

- [ ] `apm review` is being redesigned (see TICKET-LIFECYCLE): it will open
  `$EDITOR`, commit on save, and auto-resolve the state transition (no `--to`
  flag). The aggressive fetch is still needed, but the current approach section
  references the old `apm review` implementation. Update the approach to
  describe where the fetch fits in the redesigned command.
- [ ] `apm new --context` is a new variant of `apm new` (separate ticket).
  If aggressive push for `apm new` is addressed here, ensure the `--context`
  path is also covered (it creates the same branch, the same push applies).

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T19:50Z | — | new | claude-0328-c72b |
| 2026-03-28T19:54Z | new | specd | claude-0328-c72b |
| 2026-03-29T19:11Z | specd | ammend | claude-0329-1200-a1b2 |