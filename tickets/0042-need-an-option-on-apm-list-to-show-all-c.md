+++
id = 42
title = "need an option on apm list to show all closed tickets"
state = "new"
priority = 0
effort = 2
risk = 1
author = "apm"
branch = "ticket/0042-need-an-option-on-apm-list-to-show-all-c"
created_at = "2026-03-28T08:35:04.245019Z"
updated_at = "2026-03-28T18:25:54.952056Z"
+++

## Spec

### Problem

`apm list --state closed` returns no results because the list command always excludes terminal-state tickets unless `--all` is passed. Users have no intuitive way to inspect closed tickets — they must know to combine `--state closed --all`, which is surprising.

### Acceptance criteria

- [ ] `apm list --state closed` returns all closed tickets without requiring `--all`
- [ ] `apm list --state closed --all` continues to work (no regression)
- [ ] `apm list` (no flags) still excludes closed and other terminal-state tickets
- [ ] `apm list --all` still shows every ticket including terminal-state ones

### Out of scope

- Changing any other filtering behaviour (unassigned, supervisor, actionable)
- Adding a dedicated `--closed` flag (the `--state` mechanism is sufficient)

### Approach

In `apm/src/cmd/list.rs`, compute `terminal_ok` as:
- `true` when `--all` is set (current behaviour)
- `true` when `--state` explicitly names a terminal state (the requested fix)
- `false` otherwise

Concretely, change the `terminal_ok` expression from:

```rust
let terminal_ok = all || !terminal.contains(fm.state.as_str());
```

to:

```rust
let state_is_terminal = state_filter.as_deref().map_or(false, |s| terminal.contains(s));
let terminal_ok = all || state_is_terminal || !terminal.contains(fm.state.as_str());
```

No CLI argument changes needed. The fix is entirely inside `list.rs`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T08:35Z | — | new | apm |