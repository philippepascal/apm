+++
id = "0082"
title = "apm new: fall back to vi when EDITOR and VISUAL are unset"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/0082-apm-new-fall-back-to-vi-when-editor-and-"
created_at = "2026-03-30T04:49:04.279707Z"
updated_at = "2026-03-30T18:07:38.564978Z"
+++

## Spec

### Problem

`apm new` (without `--no-edit`) tries to open `$EDITOR`. If `$EDITOR` is unset
it prints `warning: $EDITOR is not set, skipping editor open` and silently
skips the edit step. The same function in `review.rs` tries `$VISUAL` before
`$EDITOR` but `new.rs` does not. Neither falls back to `vi`, which is
universally available on POSIX systems. The result: a user without `$EDITOR`
set gets no editor and a confusing warning.

### Acceptance criteria

- [x] `apm new` (without `--no-edit`) checks `$VISUAL` first, then `$EDITOR`,
  then falls back to `vi`
- [x] No warning is printed when falling back to `vi` — the fallback is silent
- [x] The `open_editor` function in `review.rs` also uses the same lookup order
  (`$VISUAL` → `$EDITOR` → `vi`) for consistency
- [x] `cargo test --workspace` passes

### Out of scope

- Making the fallback editor configurable via `apm.toml`
- Detecting whether `vi` is actually present on `$PATH`

### Approach

In `apm/src/cmd/new.rs`, replace the current lookup:

```rust
let editor = match std::env::var("EDITOR") {
    Ok(e) if !e.is_empty() => e,
    _ => {
        eprintln!("warning: $EDITOR is not set, skipping editor open");
        return Ok(());
    }
};
```

with:

```rust
let editor = std::env::var("VISUAL")
    .ok()
    .filter(|e| !e.is_empty())
    .or_else(|| std::env::var("EDITOR").ok().filter(|e| !e.is_empty()))
    .unwrap_or_else(|| "vi".to_string());
```

Apply the same change to `open_editor` in `apm/src/cmd/review.rs`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T04:49Z | — | new | claude-0330-0245-main |
| 2026-03-30T04:52Z | new | in_design | claude-0330-0245-main |
| 2026-03-30T04:53Z | in_design | specd | claude-0330-0245-main |
| 2026-03-30T05:11Z | specd | ready | apm |
| 2026-03-30T05:45Z | ready | in_progress | claude-0330-0245-main |
| 2026-03-30T06:11Z | in_progress | implemented | claude-0329-1200-a1b2 |
| 2026-03-30T14:26Z | implemented | accepted | apm |
| 2026-03-30T18:07Z | accepted | closed | apm-sync |