+++
id = "b8e9bfee"
title = "apm-core: phase-aware dependency gating via satisfies_deps tags and dep_requires"
state = "in_design"
priority = 8
effort = 0
risk = 0
author = "apm"
agent = "50734"
branch = "ticket/b8e9bfee-apm-core-phase-aware-dependency-gating-v"
created_at = "2026-04-02T21:24:08.067343Z"
updated_at = "2026-04-02T21:44:34.423045Z"
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

- [ ] `StateConfig` deserialises `satisfies_deps` as either a boolean (`true`/`false`) or a string tag (e.g. `"spec"`) without error
- [ ] `StateConfig` accepts a new optional `dep_requires` string field (e.g. `dep_requires = "spec"`)
- [ ] A ticket whose state has `dep_requires = "X"` is considered unblocked when every dep is in a state that has `satisfies_deps = "X"`, `satisfies_deps = true`, or `terminal = true`
- [ ] A ticket whose state has no `dep_requires` still requires every dep to have `satisfies_deps = true` or `terminal = true` (unchanged behaviour)
- [ ] `apm next` returns a `groomed` ticket (which has `dep_requires = "spec"`) when its only dependency is in state `specd` (which has `satisfies_deps = "spec"`)
- [ ] `apm next` does NOT return a `ready` ticket (no `dep_requires`) when its only dependency is in state `specd` only — it still requires `implemented` or `closed`
- [ ] The project `workflow.toml` has `satisfies_deps = "spec"` on the `specd` state and `dep_requires = "spec"` on the `groomed` state

### Out of scope

- Multiple gate tags per state (e.g. a state satisfying both `"spec"` and `"impl"` gates simultaneously)
- Per-dependency-edge gate overrides (gate is declared on the dependent's state, not on individual dep links)
- Any display or UI changes to how blocked/unblocked status is shown
- Changing how `in_design`, `ammend`, or `question` states interact with dependency gating

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
| 2026-04-02T21:44Z | groomed | in_design | philippepascal |