# apm integration tests

## Test conventions

### Default rule

Tests must exercise APM through real `apm` CLI invocations. Use the test harness helpers that call
`Command::new("apm")` (or equivalent). Tests must not substitute raw filesystem writes,
hand-built frontmatter strings, or direct git porcelain for commands that `apm` exposes.

### Starting point for new tests

Use the `init_repo()` helper as the canonical starting point for new integration tests. It runs
`apm init` against a fresh temp repo, producing a repo whose shape matches production. Compose
targeted overrides on top of it rather than hand-rolling a setup from scratch.

### Bypass definition

A bypass is any of the following when used in place of an `apm` command:

- `std::fs::write` (or equivalent) that creates or modifies a ticket file on a branch
- A hand-constructed frontmatter string passed to git directly
- A raw git command acting on the ticket namespace in place of an `apm` command

### When bypass is permitted

Bypass is permitted only when no `apm` command path exists — for example:

- Seeding branches into a bare origin repo
- Injecting intentionally corrupt state to test error paths
- Overriding a config field that has no setter command

### Required annotation

Every bypass line must carry an inline comment immediately above or on the same line:

```rust
// BYPASS: <one-line reason why no apm command covers this>
```

### Auditing bypasses

To list all bypasses in the test suite:

```
grep -rn "BYPASS:" apm/tests/
```
