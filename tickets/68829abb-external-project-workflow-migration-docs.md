+++
id = "68829abb"
title = "External-project workflow migration: docs and optional tooling"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/68829abb-external-project-workflow-migration-docs"
created_at = "2026-05-31T02:00:37.800326Z"
updated_at = "2026-05-31T03:04:04.126064Z"
epic = "a42eceea"
target_branch = "epic/a42eceea-workflow-schema-state-level-worker-profi"
depends_on = ["c3f5aa4d"]
+++

## Spec

### Problem

After the schema redesign lands (1e758cd5 + c3f5aa4d), any project that already has a .apm/workflow.toml in the old shape will fail to load. This ticket provides migration support for external projects (notably the user's syn project, plus any other apm-using projects in the wild).

DECIDE BEFORE WRITING THE SPEC:
- Should migration be documented (manual edit), or should there be an apm subcommand that performs it (apm migrate-workflow or similar)?
- If a tool: should it be one-shot (run once, exit) or part of apm validate (offer to migrate on validation failure)?

DEFAULT RECOMMENDATION (spec-writer can adjust): documentation first, no tool. Most apm-using projects right now are the supervisor's own, and a one-paragraph migration guide is enough. Add a tool later if more projects emerge.

SCOPE IF DOC-ONLY:

1. Add a section to the top-level README or to docs/workflow-migration.md (create if needed) titled 'Migrating workflow.toml to v2 schema'. Cover:
   - What changed: state-level worker_profile replaces transition.worker_profile and transition.role; state.actionable_by is dropped; some lifecycle transitions are removed (in_design to ammend; merge_failed to in_progress).
   - Step-by-step: for each agent-owned state, add worker_profile = 'agent/role'. Delete the transition.worker_profile and transition.role keys. Delete the state.actionable_by key. Remove the dropped transitions.
   - A worked example: a small workflow.toml before and after.
   - Validation: run apm validate after; clear errors will name what is wrong.

2. If you have access to the syn project tree, prepare a separate migration PR for its .apm/workflow.toml that mirrors the apm repo's migration (done in 1e758cd5). Otherwise, document the migration so the supervisor can apply it.

SCOPE IF TOOL:
Specify a subcommand like 'apm migrate-workflow' (or fold into 'apm validate --fix') that:
- Reads the existing .apm/workflow.toml
- Applies the transformation in memory
- Writes the new file (with a backup of the old)
- Reports any ambiguous cases for manual resolution (e.g., a transition that was tagged role: coder where the destination state is supervisor-owned — should it stay or be removed?)

TESTS (whichever path):
- The migration docs apply cleanly to a sample old-schema workflow.toml (test it manually on a copy of the syn workflow.toml).
- If a tool is added: cargo test --workspace passes; the tool produces a workflow that passes apm validate.

OUT OF SCOPE:
- Schema struct changes (1e758cd5).
- apm validate rules (c3f5aa4d).
- This project's own .apm/workflow.toml migration (already done in 1e758cd5).
- Any code paths beyond the migration tool itself if a tool is chosen.

REFERENCES:
- The before/after schema can be derived from the workflow.toml in 1e758cd5 (before this epic) vs 1e758cd5 (after this epic).
- README.md
- docs/ directory if present

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
| 2026-05-31T02:00Z | — | new | philippepascal |
| 2026-05-31T03:04Z | new | closed | philippepascal |
