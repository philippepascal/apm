+++
id = "f01e4e7b"
title = "e2e tests fail: testdata files missing"
state = "closed"
priority = 95
effort = 1
risk = 1
author = "claude-0331-1945-x7k2"
agent = "claude-0331-2000-p9x1"
branch = "ticket/f01e4e7b-e2e-tests-fail-testdata-files-missing"
created_at = "2026-03-31T19:47:32.296377Z"
updated_at = "2026-04-01T04:57:08.731746Z"
+++

## Spec

### Problem

The e2e test suite (apm/tests/e2e.rs) copies two files from testdata/src/ into each temporary test repo to seed a realistic Rust project. Those files — testdata/src/parser.rs and testdata/src/main.rs — were never committed to the repository. cargo test --workspace fails immediately with a file-not-found panic whenever the e2e tests run.

### Acceptance criteria

- [x] testdata/src/parser.rs exists and is valid Rust
- [x] testdata/src/main.rs exists and is valid Rust
- [x] cargo test --workspace passes with no file-not-found panics in e2e tests

### Out of scope

Changes to the e2e test logic itself; adding more testdata files beyond what the tests reference.

### Approach

Create testdata/src/parser.rs and testdata/src/main.rs with minimal valid Rust content — a stub parse function and a main fn respectively. No logic needed; the files exist only to be copied by the test harness.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T19:47Z | — | new | claude-0331-1945-x7k2 |
| 2026-03-31T19:51Z | new | in_design | claude-0331-2000-p9x1 |
| 2026-03-31T19:53Z | in_design | specd | claude-0331-2000-p9x1 |
| 2026-03-31T19:53Z | specd | ready | claude-0331-2000-p9x1 |
| 2026-03-31T19:53Z | ready | in_progress | claude-0331-2000-p9x1 |
| 2026-03-31T20:20Z | in_progress | implemented | claude-0331-2000-p9x1 |
| 2026-03-31T20:22Z | implemented | accepted | apm-sync |
| 2026-04-01T04:57Z | accepted | closed | apm-sync |