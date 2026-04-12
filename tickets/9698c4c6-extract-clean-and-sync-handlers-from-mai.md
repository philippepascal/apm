+++
id = "9698c4c6"
title = "Extract clean and sync handlers from main.rs"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9698c4c6-extract-clean-and-sync-handlers-from-mai"
created_at = "2026-04-12T09:03:22.310905Z"
updated_at = "2026-04-12T09:09:58.896135Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
depends_on = ["1ace7d42"]
+++

## Spec

### Problem

`apm-server/src/main.rs` contains a `clean_handler()` function spanning lines ~542-757 (216 lines) that is the single largest handler in the file. It mixes:

- Parameter parsing (lines 551-557)
- Blocking worktree candidate detection via apm_core (lines 568-592)
- Dry-run response building (lines 594-646)
- Remote branch cleanup (lines 648-667)
- Epic branch cleanup with TOML file manipulation (lines 669-750)

There is also a `sync_handler()` with similar complexity.

Both should be extracted into `handlers/maintenance.rs` (or `handlers/clean.rs` + `handlers/sync.rs`). The epic cleanup portion of `clean_handler` duplicates logic from `apm/src/cmd/clean.rs::run_epic_clean()` — if the apm CLI epic moves that to `apm_core`, the server handler should reuse it.

This ticket depends on epic handlers being extracted first to avoid main.rs merge conflicts.

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
| 2026-04-12T09:03Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
