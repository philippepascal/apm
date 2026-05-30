+++
id = "ac87718b"
title = "Replace apm list epics footer with inline ↓ marker on tickets whose epic is behind main"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ac87718b-replace-apm-list-epics-footer-with-inlin"
created_at = "2026-05-30T02:17:52.780155Z"
updated_at = "2026-05-30T02:32:24.166390Z"
+++

## Spec

### Problem

GOAL: simplify apm list output by removing the dedicated 'epics:' footer added by ticket 7a76dd16 and instead showing a minimal visual indicator (the down arrow ↓) next to the epic identifier in each ticket row whose epic is behind main. Users wanting full details (commit count, clean vs conflicts) run apm epic list — that command already surfaces the full freshness label per ticket 7a76dd16.

PROBLEM: the current 'epics:' footer in apm list is functional but adds vertical noise to the most-used triage command and forces the user to mentally cross-reference the footer entries against the tickets in the rows above. The supervisor scanning apm list cares about WHICH tickets are on stale epics; the count and conflict detail is secondary and lives more naturally in apm epic list.

APPROACH (direction; spec-writer to refine):
1. Remove the existing epics-freshness footer block from apm/src/cmd/list.rs (the section appended after the stale-tickets footer that prints 'epics:' followed by one line per stale epic). Keep the merge_tree_status calls — they are needed for the new inline indicator — but stop emitting the footer.
2. For each filtered ticket row that has a non-empty epic (or target_branch starting with epic/), compute the epic's freshness once per distinct epic (deduplicate via a BTreeMap keyed by epic ID, same as the current footer logic). If merge_tree_status returns ahead > 0 for that epic, append a small marker beside wherever the epic identifier appears in the row. The marker is a bare ↓ — no count, no clean/conflict label. Tickets whose epic is up to date show no marker. Tickets without an epic show no marker.
3. The spec-writer chooses the rendering position. The natural candidates are (a) immediately after the target_branch column when target_branch starts with epic/, or (b) next to the 8-char epic ID if one is rendered. Pick whichever fits the existing column structure with the least disruption; do not invent a new column.
4. apm epic list is unaffected (it already shows the full freshness label).

CONSTRAINTS:
- The marker must be unambiguous (no false positives on tickets whose target_branch is not an epic — i.e. main-scoped tickets never show it).
- Re-use the existing helper: apm_core::epic::merge_tree_status (introduced by ticket 12f2c7fa). Do not add a new freshness helper.
- Compute freshness at most once per distinct epic in the filtered set; the same epic appearing on many tickets must not re-run git per ticket.
- The marker rendering must work in non-TTY (no color codes that mangle piped output), and color is optional (if used, gate behind isatty).

OUT OF SCOPE:
- Changes to apm epic list (already shows full freshness label).
- Changes to apm-server or apm-ui — the SupervisorView chip bar from 7a76dd16 is unaffected.
- Changes to apm next (its current behavior of printing a freshness note when the top ticket has an epic stays as-is).
- A new CLI flag to toggle the marker or restore the footer.
- Changes to merge_tree_status itself.

TESTS:
- An integration test that runs apm list on a repo with two epics — one up-to-date and one behind main — and tickets scoped to each, plus a main-scoped ticket. Assert that ticket rows on the stale epic include ↓ where the epic id is rendered; ticket rows on the fresh epic do not; the main-scoped ticket row never includes ↓; the old 'epics:' footer no longer appears in the output.
- An integration test that verifies merge_tree_status is called exactly once per distinct epic id, not once per ticket row, when multiple tickets share the same stale epic.
- Existing apm list integration tests that asserted the footer wording are updated or replaced; tests that asserted absence of the footer (the empty case) continue to pass.

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
| 2026-05-30T02:17Z | — | new | philippepascal |
| 2026-05-30T02:18Z | new | groomed | philippepascal |
| 2026-05-30T02:32Z | groomed | in_design | philippepascal |
