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

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

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