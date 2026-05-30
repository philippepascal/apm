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
| 2026-05-30T02:11Z | — | new | philippepascal |
| 2026-05-30T02:14Z | new | groomed | philippepascal |
| 2026-05-30T02:21Z | groomed | in_design | philippepascal |