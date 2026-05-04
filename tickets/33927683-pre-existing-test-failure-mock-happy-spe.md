+++
id = "33927683"
title = "Pre-existing test failure: mock_happy_spec_mode_transitions_to_specd"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/33927683-pre-existing-test-failure-mock-happy-spe"
created_at = "2026-05-04T03:33:27.432606Z"
updated_at = "2026-05-04T04:35:40.291779Z"
+++

## Spec

### Problem

The integration test `start::tests::mock_happy_spec_mode_transitions_to_specd` (and two sibling tests that share the same helper) fails because `find_apm_bin()` — a test-only helper in `apm-core/src/start.rs` — resolves to the wrong binary.\n\n`find_apm_bin()` first checks `APM_BIN`, then falls back to `which apm`. On any machine with Atom Package Manager installed via Homebrew, `which apm` returns `/opt/homebrew/bin/apm`, which does not recognise the `spec`, `state`, or `set` subcommands. The mock-happy shell script calls `"$APM" spec "$ID" ...`, which fails immediately, so the ticket never reaches `specd` state and the assertion fires.\n\nThe correct binary is the one this workspace builds: `target/debug/apm` (or `target/release/apm`). The fix is to replace the `which apm` fallback with a lookup that derives the target directory from `std::env::current_exe()`, which always points at the actual Cargo test binary regardless of PATH.\n\nThe failure pre-dates ticket f8cbd68c and is not a regression introduced by that branch.

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
| 2026-05-04T03:33Z | — | new | claude-0503-1430-f8cb|philippepascal |
| 2026-05-04T04:35Z | new | groomed | philippepascal |
| 2026-05-04T04:35Z | groomed | in_design | philippepascal |