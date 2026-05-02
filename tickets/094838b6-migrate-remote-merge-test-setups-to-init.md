+++
id = "094838b6"
title = "Migrate remote-merge test setups to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/094838b6-migrate-remote-merge-test-setups-to-init"
created_at = "2026-05-01T20:27:20.736073Z"
updated_at = "2026-05-02T04:05:51.594813Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

Three setup helpers in `apm/tests/integration.rs` back the merge-strategy integration tests:
`setup_squash_remote` (line 3914), `setup_pr_or_epic_merge_remote` (line 4710), and
`setup_merge_strategy_remote` (line 5301). All three follow the same bare-remote + local-clone
pattern, but each hand-writes a minimal `apm.toml` at repo root and never calls `apm init`.

As a result each fixture diverges from a real user repo in two ways:

1. Config is written to the legacy `apm.toml` location instead of `.apm/config.toml` /
   `.apm/workflow.toml`.
2. The hand-crafted workflow states are a small, frozen subset of the production default
   (3–4 states vs. the 12-state production default).

Any change to the init template — new required states, renamed fields, config file layout —
is invisible to the 4 tests backed by these helpers. The helpers should instead call the real
`apm init` binary (same pattern established by dependency ticket 795dce11's `init_repo()`
helper) and override only what the test needs via real commands or marked bypass.

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
| 2026-05-02T03:08Z | new | groomed | philippepascal |
| 2026-05-02T04:05Z | groomed | in_design | philippepascal |