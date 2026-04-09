+++
id = "d9436266"
title = "apm-core has 15 pre-existing clippy -D warnings violations"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d9436266-apm-core-has-15-pre-existing-clippy-d-wa"
created_at = "2026-04-08T00:24:00.645866Z"
updated_at = "2026-04-09T00:24:25.698181Z"
+++

## Spec

### Problem

Running `cargo clippy --package apm-core -- -D warnings` fails with 15 pre-existing violations across three files in the apm-core crate:\n\n- **archive.rs**: 1 × `double_ended_iterator_last` — `.last()` called on a `DoubleEndedIterator` instead of the more efficient `.next_back()`.\n- **start.rs**: 1 × `too_many_arguments` — `spawn_container_worker` has 10 parameters (clippy default limit: 7).\n- **ticket.rs**: 2 × `too_many_arguments` (`pick_next` at 9 params, `create` at 13 params); 7 × `unnecessary_map_or` (`.map_or(true, …)` / `.map_or(false, …)` should use `is_none_or` / `is_some_and`); 3 × `manual_strip` (`starts_with` + index slice should use `strip_prefix`).\n\nThese warnings are not regressions — they predate ticket 24069bd8 — but they prevent enabling `-D warnings` in per-crate clippy CI for apm. Fixing them unblocks that CI gate without touching any public API signatures.

### Acceptance criteria

- [ ] `cargo clippy --package apm-core -- -D warnings` exits with code 0
- [ ] `archive.rs` `double_ended_iterator_last` warning is gone (`.last()` replaced with `.next_back()`)
- [ ] `start.rs` `too_many_arguments` warning on `spawn_container_worker` is suppressed with `#[allow(clippy::too_many_arguments)]`
- [ ] `ticket.rs` `too_many_arguments` warning on `pick_next` is suppressed with `#[allow(clippy::too_many_arguments)]`
- [ ] `ticket.rs` `too_many_arguments` warning on `create` is suppressed with `#[allow(clippy::too_many_arguments)]`
- [ ] `ticket.rs` `unnecessary_map_or` warnings are gone — all instances of `.map_or(true, |x| …)` replaced with `.is_none_or(|x| …)` and `.map_or(false, |x| …)` replaced with `.is_some_and(|x| …)`
- [ ] `ticket.rs` `manual_strip` warnings are gone — `starts_with(prefix)` + index slice replaced with `strip_prefix(prefix)`
- [ ] All existing apm-core tests continue to pass (`cargo test --package apm-core`)

### Out of scope

- Refactoring `too_many_arguments` functions into builder/config structs (a future ticket may do this; here we just suppress the lint)\n- Fixing clippy warnings in any crate other than apm-core\n- Adding `-D warnings` to CI configuration (that is a follow-on ticket once the crate is clean)\n- Any behaviour changes to `pick_next`, `create`, or `spawn_container_worker`

### Approach

All changes are in `apm-core/src/`. No public API signatures change. Run `cargo clippy --package apm-core -- -D warnings` after each file to verify incrementally.

**`apm-core/src/archive.rs` — 1 fix**

Line ~83: replace `.last()` with `.next_back()` on the `rel_path.split('/')` chain.

```rust
// before
let filename = rel_path.split('/').last().unwrap_or(rel_path.as_str());
// after
let filename = rel_path.split('/').next_back().unwrap_or(rel_path.as_str());
```

**`apm-core/src/start.rs` — 1 suppression**

Add `#[allow(clippy::too_many_arguments)]` on the line immediately before `fn spawn_container_worker(`.

**`apm-core/src/ticket.rs` — multiple fixes**

1. **`too_many_arguments` × 2** — add `#[allow(clippy::too_many_arguments)]` before `pub fn pick_next(` (~line 203) and before `pub fn create(` (~line 437).

2. **`unnecessary_map_or`** — replace every matching pattern. Exact locations from exploration:

   - Line ~166 (inside `pick_next`):
     ```rust
     // before
     .filter(|t| owner_filter.map_or(true, |f| t.frontmatter.owner.as_deref() == Some(f)))
     // after
     .filter(|t| owner_filter.is_none_or(|f| t.frontmatter.owner.as_deref() == Some(f)))
     ```
   - Line ~737:
     ```rust
     // before
     let state_ok = state_filter.map_or(true, |s| fm.state == s);
     // after
     let state_ok = state_filter.is_none_or(|s| fm.state == s);
     ```
   - Line ~739:
     ```rust
     // before
     let state_is_terminal = state_filter.map_or(false, |s| terminal.contains(s));
     // after
     let state_is_terminal = state_filter.is_some_and(|s| terminal.contains(s));
     ```
   - Lines ~741–743:
     ```rust
     // before
     let actionable_ok = actionable_filter.map_or(true, |actor| {
         actionable_map.get(fm.state.as_str())
             .map_or(false, |actors| actors.iter().any(|a| a == actor || a == "any"))
     });
     // after
     let actionable_ok = actionable_filter.is_none_or(|actor| {
         actionable_map.get(fm.state.as_str())
             .is_some_and(|actors| actors.iter().any(|a| a == actor || a == "any"))
     });
     ```
   - Lines ~745–747:
     ```rust
     // before
     let author_ok = author_filter.map_or(true, |a| fm.author.as_deref() == Some(a));
     let owner_ok  = owner_filter.map_or(true, |o| fm.owner.as_deref() == Some(o));
     let mine_ok   = mine_user.map_or(true, |me| { … });
     // after
     let author_ok = author_filter.is_none_or(|a| fm.author.as_deref() == Some(a));
     let owner_ok  = owner_filter.is_none_or(|o| fm.owner.as_deref() == Some(o));
     let mine_ok   = mine_user.is_none_or(|me| { … });
     ```

3. **`manual_strip`** — around line 598, replace the `starts_with` + index-slice pattern with `strip_prefix`:
   ```rust
   // before
   if l.starts_with("- [ ] ") {
       Some(ChecklistItem { checked: false, text: l[6..].to_string() })
   } else if l.starts_with("- [x] ") {
       Some(ChecklistItem { checked: true, text: l[6..].to_string() })
   } else if l.starts_with("- [X] ") {
       Some(ChecklistItem { checked: true, text: l[6..].to_string() })
   }
   // after
   if let Some(s) = l.strip_prefix("- [ ] ") {
       Some(ChecklistItem { checked: false, text: s.to_string() })
   } else if let Some(s) = l.strip_prefix("- [x] ") {
       Some(ChecklistItem { checked: true, text: s.to_string() })
   } else if let Some(s) = l.strip_prefix("- [X] ") {
       Some(ChecklistItem { checked: true, text: s.to_string() })
   }
   ```

**Verification order**

1. Edit `archive.rs`, run `cargo clippy --package apm-core -- -D warnings` — confirm 1 fewer warning.
2. Edit `start.rs`, re-run.
3. Edit `ticket.rs` (all changes in one pass), re-run — expect zero warnings.
4. Run `cargo test --package apm-core` — all tests must pass.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T00:24Z | — | new | philippepascal |
| 2026-04-08T23:49Z | new | groomed | apm |
| 2026-04-08T23:58Z | groomed | in_design | philippepascal |
| 2026-04-09T00:01Z | in_design | specd | claude-0408-2358-dfd0 |
| 2026-04-09T00:24Z | specd | ready | apm |
| 2026-04-09T00:24Z | ready | in_progress | philippepascal |
