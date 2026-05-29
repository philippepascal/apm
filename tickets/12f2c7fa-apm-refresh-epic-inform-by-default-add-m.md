+++
id = "12f2c7fa"
title = "apm refresh-epic: inform by default, add --merge / --pr / --auto modes"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/12f2c7fa-apm-refresh-epic-inform-by-default-add-m"
created_at = "2026-05-29T01:17:38.982422Z"
updated_at = "2026-05-29T01:26:27.618639Z"
+++

## Spec

### Problem

CURRENT BEHAVIOR: apm refresh-epic <id> (apm/src/cmd/epic.rs run_refresh_epic) always opens a PR from the default branch (main) into the epic branch. It requires the epic to be quiescent (epic_is_quiescent) and bails otherwise. There is no way to just check the situation, and no way to do a direct local merge.

DESIRED BEHAVIOR — make the mode explicit:
- Default (no flags): INFORM ONLY. Detect whether main merges cleanly into the epic branch and report the result — ahead-count of main over the epic plus clean-vs-conflict status. Do NOT modify anything. Use git merge-tree for the clean/conflict detection (no working-tree changes). This mode is read-only, so it should work regardless of whether the epic is quiescent.
- --merge: actually perform the merge of main into the epic branch (local merge).
- --pr: create/update the PR from main into the epic branch (the current behavior).
- --auto: merge if the merge is clean; otherwise fall back to creating a PR.

QUIESCENCE: the acting modes (--merge, --pr, --auto) keep the existing quiescence requirement — refreshing an epic while its tickets are in flight is unsafe. The default inform mode must NOT require quiescence (it only reads).

FLAG VALIDATION: --merge, --pr, --auto are mutually exclusive; reject combinations with a clear error.

SHARED PRIMITIVE: the clean/conflict detection (main -> epic via git merge-tree) is the same primitive needed by the epic-freshness surfacing work (separate ticket, 7a76dd16). Implement it once as a reusable apm-core helper and use it in both places rather than duplicating.

OUT OF SCOPE: surfacing freshness in apm list/UI (separate ticket 7a76dd16); auto-detecting staleness at dispatch time or gating dispatch; an 'accept divergence' mechanism. This ticket is only about the refresh-epic command's modes.

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
| 2026-05-29T01:17Z | — | new | philippepascal |
| 2026-05-29T01:18Z | new | groomed | philippepascal |
| 2026-05-29T01:26Z | groomed | in_design | philippepascal |
