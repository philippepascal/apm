+++
id = "5c494a5d"
title = "Migrate setup_merge() to init_repo()"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5c494a5d-migrate-setup-merge-to-init-repo"
created_at = "2026-05-01T20:26:46.198163Z"
updated_at = "2026-05-02T03:22:43.659646Z"
epic = "0b1c71db"
target_branch = "epic/0b1c71db-integration-tests-use-real-apm-commands"
depends_on = ["795dce11"]
+++

## Spec

### Problem

`setup_merge()` at `apm/tests/integration.rs:134` hand-writes an `apm.toml` at the repo root containing a 7-state workflow and `completion = "merge"` on the `in_progress → implemented` transition. It never calls `apm init`, so the fixture diverges from the production repo shape: the file is written to the legacy `apm.toml` location (not `.apm/workflow.toml`), the state list is smaller than the production default, and changes to the init template — new states, field renames, config file layout — are invisible to the 6 tests that depend on this helper.

The `merge` completion strategy is intentional and must be preserved. The 6 `depends_on` tests create tickets with no epic, relying on identical `target_branch` values (both defaulting to `main`) to satisfy validation. The production default `completion = "pr_or_epic_merge"` would reject those tickets because they lack an epic. There is no `apm` command to change a workflow completion strategy post-init, so a `// BYPASS:` filesystem edit is required after `init_repo()` runs.

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
| 2026-05-01T20:26Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:22Z | groomed | in_design | philippepascal |