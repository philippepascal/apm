+++
id = "778b63c6"
title = "Surface merge-failure state and recovery hints in apm-server and apm-ui (read-only)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/778b63c6-surface-merge-failure-state-and-recovery"
created_at = "2026-05-30T02:11:35.270399Z"
updated_at = "2026-05-30T02:21:49.219720Z"
depends_on = ["ae4104f2"]
+++

## Spec

### Problem

The apm-ui supervisor board renders tickets in `merge_failed` (and equivalently-configured) states identically to tickets in normal states such as `in_progress` or `implemented`. When a merge operation fails, the git error is captured in the ticket body under `### Merge notes` and the ticket is moved to the failure state automatically, but the UI shows no visual cue that the ticket is stuck. The supervisor must leave the UI, run `apm show <id>` in the terminal, read the captured error, and work out which `apm state` command to run — information that should be immediately visible in the triage view.

This ticket extends `apm-server` and `apm-ui` to surface two pieces of recovery context: (a) a visual badge on the ticket card indicating merge failure, and (b) a detail panel showing the raw git error and the exact CLI commands to recover. It depends on ae4104f2, which adds `classify_recovery_options(state_id, config)` to `apm-core`. That function inspects the workflow config and classifies each available transition from a given state as `RetryMerge`, `ReturnToWorker`, `Abandon`, or `Other`, without hardcoding any state name. The server consumes this output to compute which state IDs are merge-failure states and to generate per-ticket recovery command strings; the UI renders them read-only. No state-transition API surface is added.

### Acceptance criteria

- [ ] `GET /api/tickets` response envelope includes a `merge_failure_state_ids` field — a JSON array of state ID strings whose available transitions include at least one `RetryMerge` recovery option, as determined by `classify_recovery_options`; the array is empty when no git root is present or config fails to load.
- [ ] `GET /api/tickets/:id` response includes `merge_notes` — the trimmed string content of the `### Merge notes` section of the ticket body, or `null` when that section is absent or empty.
- [ ] `GET /api/tickets/:id` response includes `recovery_options` — a JSON array of objects each with `to` (target state ID), `label` (human-readable name), `kind` (one of `"retry_merge"`, `"return_to_worker"`, `"abandon"`, `"other"`), and `command` (the literal string `"apm state <ticket-id> <to>"`); the array is empty when the ticket's current state has no outgoing transitions or no git root is present.
- [ ] A `TicketCard` whose `ticket.state` is present in the `merge_failure_state_ids` array received from the list endpoint renders a distinct red visual marker.
- [ ] A `TicketCard` whose `ticket.state` is absent from `merge_failure_state_ids` renders no merge-failure marker, even if the state name happens to be `"merge_failed"`.
- [ ] The `TicketDetail` panel for a ticket with a non-null `merge_notes` value displays a "Merge failure" section with the notes rendered verbatim inside a monospace pre block.
- [ ] The `TicketDetail` panel for a ticket with a non-empty `recovery_options` array displays a "Recovery" section listing each option's label, a human-readable kind description (e.g. "Retry merge", "Return to worker", "Abandon"), and the `command` string in a monospace code block styled for easy copying; the section ends with a reference link to `docs/merge-failed-recovery.md`.
- [ ] A ticket whose `recovery_options` is an empty array and `merge_notes` is `null` renders neither the "Merge failure" section nor the "Recovery" section in the detail panel.
- [ ] Server integration test (git-based, default workflow): `GET /api/tickets` returns `merge_failure_state_ids` containing `"merge_failed"`.
- [ ] Server integration test (InMemory): `GET /api/tickets/:id` for a ticket whose body contains `### Merge notes\n\ngit error text` returns `merge_notes: "git error text"`.
- [ ] Server integration test (git-based, default workflow): `GET /api/tickets/:id` for a ticket in `merge_failed` state returns `recovery_options` with at least one entry where `kind` is `"retry_merge"` and `command` matches `"apm state <id> implemented"`.

### Out of scope

- Action buttons or any new API endpoint that triggers a state transition; recovery happens exclusively via the `apm` CLI.
- Inline editing of `### Merge notes` or any other ticket body section in the UI.
- Dispatcher or `apm work` behavior changes around merge failure.
- Hardcoding `"merge_failed"` or any other state name in the server or frontend; all merge-failure classification flows through `classify_recovery_options`.
- Changes to the recovery classification logic itself (delivered by ae4104f2).
- CLI changes to `apm show`, `apm list`, or `apm next` (covered by ae4104f2).
- The `TicketResponse` (list endpoint per-ticket object) gaining a per-ticket `recovery_options` field; the badge is driven by the envelope-level `merge_failure_state_ids` to avoid computing classification for every ticket on every list call.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T02:11Z | — | new | philippepascal |
| 2026-05-30T02:14Z | new | groomed | philippepascal |
| 2026-05-30T02:21Z | groomed | in_design | philippepascal |