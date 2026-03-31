+++
id = "f01e4e7b"
title = "e2e tests fail: testdata files missing"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0331-1945-x7k2"
branch = "ticket/f01e4e7b-e2e-tests-fail-testdata-files-missing"
created_at = "2026-03-31T19:47:32.296377Z"
updated_at = "2026-03-31T19:51:57.809926Z"
+++

## Spec

### Problem

The e2e test suite (apm/tests/e2e.rs) copies two files from testdata/src/ into each temporary test repo to seed a realistic Rust project. Those files — testdata/src/parser.rs and testdata/src/main.rs — were never committed to the repository. cargo test --workspace fails immediately with a file-not-found panic whenever the e2e tests run.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T19:47Z | — | new | claude-0331-1945-x7k2 |
| 2026-03-31T19:51Z | new | in_design | claude-0331-2000-p9x1 |