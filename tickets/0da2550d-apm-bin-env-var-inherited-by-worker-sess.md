+++
id = "0da2550d"
title = "APM_BIN env var inherited by worker sessions may point to stale Homebrew binary"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0da2550d-apm-bin-env-var-inherited-by-worker-sess"
created_at = "2026-05-04T05:00:29.668572Z"
updated_at = "2026-05-04T05:07:10.085185Z"
+++

## Spec

### Problem

During worker sessions started via apm start, APM_BIN is set to the installed binary (e.g. /opt/homebrew/Cellar/apm/0.1.18/bin/apm-server). Worker processes and their cargo test runs inherit this. If the installed binary predates the spec subcommand, mock-wrapper tests that respect APM_BIN priority will fail in the worker environment unless APM_BIN is cleared before test invocation. Consider either: (1) not setting APM_BIN in the worker env, (2) setting it to the CLI binary not the server binary, or (3) documenting that cargo test --workspace should be run with env -u APM_BIN in worker sessions.

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
| 2026-05-04T05:00Z | — | new | claude-0504-0441-a918|philippepascal |
| 2026-05-04T05:07Z | new | groomed | philippepascal |
| 2026-05-04T05:07Z | groomed | in_design | philippepascal |
