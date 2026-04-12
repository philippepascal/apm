+++
id = "2973f8d1"
title = "Move compute_blocking_deps and compute_valid_transitions to apm_core"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2973f8d1-move-compute-blocking-deps-and-compute-v"
created_at = "2026-04-12T09:02:59.113894Z"
updated_at = "2026-04-12T09:02:59.113894Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
+++

## Spec

### Problem

`apm-server/src/main.rs` contains two business-logic functions that belong in `apm-core`, not in the HTTP server:

1. **`compute_blocking_deps()`** (lines ~416-443) — given a ticket and all tickets, computes which dependencies are blocking it. This is pure domain logic with no HTTP or server concerns. It duplicates reasoning that `apm_core::ticket` already partially implements (e.g., `dep_satisfied`, `build_reverse_index`).

2. **`compute_valid_transitions()`** (lines ~445-469) — given a ticket's current state and the workflow config, returns the list of valid next states. This duplicates `apm_core::state::available_transitions()`.

Both functions are called from ticket/epic handlers. Moving them to `apm_core` makes them testable independently and available to the CLI if needed. This should be done before extracting handlers, since the extracted handler modules will need to import these from `apm_core` rather than from a sibling module.

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
| 2026-04-12T09:02Z | — | new | philippepascal |