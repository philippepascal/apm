+++
id = "40fdde3b"
title = "Drop apm.toml legacy fallback from Config::load"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/40fdde3b-drop-apm-toml-legacy-fallback-from-confi"
created_at = "2026-05-01T20:27:33.796162Z"
updated_at = "2026-05-02T04:38:13.572145Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["dac20967", "5c494a5d", "296c1061", "c148f904", "f701ef81", "4abc535a", "cc154ee4", "a0171e83", "464d67d5", "094838b6", "443a1840", "059e2e74"]
+++

## Spec

### Problem

apm-core/src/config.rs:644-648 falls back to apm.toml at repo root when .apm/config.toml is missing. The fallback exists only for tests that hand-write apm.toml; once the integration-test migration is complete (sibling tickets in this epic), no production code path or test relies on it. Remove the fallback path. Failure mode after removal should be a clear error (config not found, run `apm init`). Verify by running the full test suite — any test that breaks indicates a sibling migration was incomplete. This ticket should be done last in the epic.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

- Removing the apm init --migrate path (init.rs lines 156-169); that still moves apm.toml to .apm/config.toml for real users migrating old repos\n- Changing any test behaviour — only fixture setup code changes\n- Adding new apm commands\n- Migrating the integration.rs helpers already covered by sibling tickets (dac20967, 5c494a5d, 296c1061, c148f904, f701ef81, 4abc535a, cc154ee4, a0171e83, 464d67d5, 094838b6, 443a1840, 059e2e74)\n- The e2e.rs first setup (lines 43-115) that writes apm.toml then calls apm init; that is testing migration and remains valid

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-01T20:27Z | — | new | philippepascal |
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:38Z | groomed | in_design | philippepascal |