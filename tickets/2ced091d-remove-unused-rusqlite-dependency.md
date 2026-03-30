+++
id = "2ced091d"
title = "remove unused rusqlite dependency"
state = "in_design"
priority = 0
effort = 1
risk = 0
author = "philippepascal"
agent = "79891"
branch = "ticket/2ced091d-remove-unused-rusqlite-dependency"
created_at = "2026-03-30T17:11:30.733908Z"
updated_at = "2026-03-30T17:21:51.731632Z"
+++

## Spec

### Problem

The workspace Cargo.toml declares rusqlite (version 0.31, bundled feature) as a workspace dependency, and apm-core/Cargo.toml pulls it in. The dependency is never imported or used in any .rs source file.

The bundled feature compiles SQLite from source as part of every build. This adds significant C compilation time and binary weight for zero benefit. Removing it speeds up clean builds and removes a transitive C dependency from the project.

### Acceptance criteria

- [ ] [ ] cargo build succeeds\n[ ] cargo test passes\n[ ] rusqlite removed from Cargo.toml files\n[ ] rusqlite absent from Cargo.lock

### Out of scope

Replacing rusqlite with a different storage backend. This ticket is strictly a removal; no alternative caching layer is introduced.

### Approach

1. In Cargo.toml (workspace root): delete the rusqlite line from [workspace.dependencies].
2. In apm-core/Cargo.toml: delete the rusqlite line from [dependencies].
3. Run cargo build --workspace to confirm compilation succeeds and Cargo.lock is regenerated without rusqlite.
4. Run cargo test --workspace to confirm all tests still pass.

No source files need to change because rusqlite was never imported or used in any .rs file.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T17:11Z | — | new | philippepascal |
| 2026-03-30T17:20Z | new | in_design | philippepascal |