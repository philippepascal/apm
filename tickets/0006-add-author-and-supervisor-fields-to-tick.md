+++
id = 6
title = "Add author and supervisor fields to ticket frontmatter"
state = "ready"
priority = 5
effort = 3
risk = 2
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

The spec defines `author` (set at creation, never changes) and `supervisor`
(the engineer responsible for approving specs and reviewing PRs). Both are missing
from `Frontmatter`. Without `supervisor`, the board cannot show each engineer their
personal slice, and there is no way to route questions or spec approvals.

### Acceptance criteria

- [ ] `Frontmatter` has `author: Option<String>` and `supervisor: Option<String>`
- [ ] `apm new` sets `author` from `APM_AGENT_NAME` env var (null if unset)
- [ ] `apm set <id> supervisor <name>` updates the supervisor field
- [ ] `apm set <id> author <name>` is rejected with a clear error ("author is immutable")
- [ ] `apm list --supervisor <name>` filters to tickets supervised by that name
- [ ] Existing ticket files without these fields parse correctly (fields default to `None`)

### Out of scope

- Board "personal slice" filtering beyond `--supervisor` flag
- `apm supervise` and `apm take` commands (tracked in #9)
- Enforcing that supervisor is a known/registered user

### Approach

1. Add `author` and `supervisor` to `Frontmatter` with `#[serde(skip_serializing_if = "Option::is_none")]`
2. In `cmd/new.rs`: read `APM_AGENT_NAME` env var, set `author`
3. In `cmd/set.rs`: handle `supervisor` field; reject `author` with an error
4. In `cmd/list.rs`: add `--supervisor` arg, filter on `fm.supervisor`

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
| 2026-03-25 | manual | specd → ready | |
