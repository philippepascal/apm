+++
id = "40fdde3b"
title = "Drop apm.toml legacy fallback from Config::load"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/40fdde3b-drop-apm-toml-legacy-fallback-from-confi"
created_at = "2026-05-01T20:27:33.796162Z"
updated_at = "2026-05-01T20:27:33.796162Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
+++

## Spec

### Problem

apm-core/src/config.rs:644-648 falls back to apm.toml at repo root when .apm/config.toml is missing. The fallback exists only for tests that hand-write apm.toml; once the integration-test migration is complete (sibling tickets in this epic), no production code path or test relies on it. Remove the fallback path. Failure mode after removal should be a clear error (config not found, run `apm init`). Verify by running the full test suite — any test that breaks indicates a sibling migration was incomplete. This ticket should be done last in the epic.

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
| 2026-05-01T20:27Z | — | new | philippepascal |
