+++
id = "d86a0a3c"
title = "Verify apm instructions one-liner no longer mentions shell discipline"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d86a0a3c-verify-apm-instructions-one-liner-no-lon"
created_at = "2026-05-31T02:11:53.054170Z"
updated_at = "2026-05-31T03:04:15.428856Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["7e66181a", "56500644", "68829abb", "d2a947ea"]
+++

## Spec

### Problem

Small verification ticket. After d2a947ea lands, verify the 'apm instructions' one-line summary in the apm Command Reference section is current.

PROBLEM (current state on main): the one-liner reads 'apm instructions  Output APM system knowledge for agents: state machine, ticket format, shell discipline, session identity, and command reference'. The 'shell discipline,' clause is stale — ticket a3c34ddc moved shell discipline out of apm instructions into the role files.

WHAT TO DO:
- Run 'apm instructions' (no args) or 'apm --help' or whichever surface renders this short summary.
- Confirm 'shell discipline' is no longer mentioned in the apm instructions one-liner.
- If d2a947ea missed it, fix here.

OUT OF SCOPE:
- All other help text (d2a947ea covers the full audit).
- The apm instructions content itself (already migrated by a3c34ddc).

REFERENCES:
- apm/src/main.rs or wherever the about = '...' attribute is set on the instructions subcommand
- a3c34ddc for the change that made this stale
- d2a947ea for the broader help-text audit

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
| 2026-05-31T02:11Z | — | new | philippepascal |
| 2026-05-31T03:04Z | new | closed | philippepascal |
