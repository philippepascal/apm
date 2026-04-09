+++
id = "d9436266"
title = "apm-core has 15 pre-existing clippy -D warnings violations"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d9436266-apm-core-has-15-pre-existing-clippy-d-wa"
created_at = "2026-04-08T00:24:00.645866Z"
updated_at = "2026-04-08T23:58:20.131402Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-08T00:24Z | — | new | philippepascal |
| 2026-04-08T23:49Z | new | groomed | apm |
| 2026-04-08T23:58Z | groomed | in_design | philippepascal |