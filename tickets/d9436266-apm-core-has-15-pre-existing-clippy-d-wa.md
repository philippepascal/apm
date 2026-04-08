+++
id = "d9436266"
title = "apm-core has 15 pre-existing clippy -D warnings violations"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/d9436266-apm-core-has-15-pre-existing-clippy-d-wa"
created_at = "2026-04-08T00:24:00.645866Z"
updated_at = "2026-04-08T23:49:45.268479Z"
+++

## Spec

### Problem

Running cargo clippy --package apm -- -D warnings fails because apm-core (a dependency) has 15 violations: double_ended_iterator_last in archive.rs, too_many_arguments in start.rs/ticket.rs, unnecessary_map_or in ticket.rs, manual_strip in ticket.rs. These are not new; they existed before ticket 24069bd8 and block any per-crate clippy -D warnings CI for apm.

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
