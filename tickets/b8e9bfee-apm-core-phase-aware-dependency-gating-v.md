+++
id = "b8e9bfee"
title = "apm-core: phase-aware dependency gating via satisfies_deps tags and dep_requires"
state = "groomed"
priority = 8
effort = 0
risk = 0
author = "apm"
branch = "ticket/b8e9bfee-apm-core-phase-aware-dependency-gating-v"
created_at = "2026-04-02T21:24:08.067343Z"
updated_at = "2026-04-02T21:25:06.883342Z"
+++

## Spec

### Problem

Dependency satisfaction is currently binary: a dep either has `satisfies_deps = true` (implemented/closed) or it doesn't. This means a ticket in `groomed` (waiting to have its spec written) cannot be dispatched until all its deps are fully implemented — even though the only thing needed to write its spec is that the dep's spec exists (`specd`).

In practice this makes the supervisor a bottleneck: every ticket in a dependency chain must be fully implemented before downstream spec work can begin in parallel, eliminating most of the benefit of having a spec-writing phase at all.

The fix is to allow states to declare a named gate tag via `satisfies_deps`, and allow actionable states to declare which gate they require via `dep_requires`. A dep is satisfied for ticket A if the dep's state carries a tag that matches A's required gate (or has `satisfies_deps = true`, or is terminal).

Example for this project's `apm.toml`:
- `specd` gets `satisfies_deps = "spec"` — a dep at this state unblocks downstream spec-writing
- `groomed` gets `dep_requires = "spec"` — only needs deps to reach spec level
- `ready` needs no change — defaults to requiring `satisfies_deps = true` (current behaviour)

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
| 2026-04-02T21:24Z | — | new | apm |
| 2026-04-02T21:25Z | new | groomed | apm |
