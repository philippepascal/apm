+++
id = "c8041bff"
title = "apm-server tests fail in CI due to missing apm-ui/dist"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/c8041bff-apm-server-tests-fail-in-ci-due-to-missi"
created_at = "2026-04-07T00:22:33.027201Z"
updated_at = "2026-04-07T00:22:33.027201Z"
+++

## Spec

### Problem

cargo test --workspace fails for apm-server because include_dir!("apm-ui/dist") panics at compile time when the UI has not been built. This is pre-existing and unrelated to any recent changes — confirmed by testing on main branch without worktree modifications. The UI dist directory must be built (npm/vite build) before apm-server tests can run.

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
| 2026-04-07T00:22Z | — | new | philippepascal |
