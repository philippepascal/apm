+++
id = "0c2ef539"
title = "Add pr_or_epic_merge completion strategy"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/0c2ef539-add-pr-or-epic-merge-completion-strategy"
created_at = "2026-04-02T00:38:36.244478Z"
updated_at = "2026-04-02T00:38:43.969736Z"
+++

## Spec

### Problem

The `completion` field on workflow transitions currently supports `"pr"`, `"merge"`, and `"none"`. These are global — every ticket using that transition behaves the same way. This is wrong for the epics model: tickets inside an epic should merge directly into the epic branch (no PR, since the epic branch itself will be PRed to main later), while free tickets should open a PR to main as usual.

Spec reference: `docs/epics.md` (§ Workflow integration). The epic branch is the integration point — individual ticket merges to it are internal; the final epic-to-main merge is the one that gets reviewed.

A new strategy value `"pr_or_epic_merge"` handles both cases in one config entry:
- If the ticket has `target_branch` set in frontmatter → merge into `target_branch` directly (no PR)
- If `target_branch` is absent → open a PR to the default branch (existing `"pr"` behaviour)

This lets the workflow config express the intended policy once, without per-ticket overrides or separate transitions for epic vs. free tickets.

### Acceptance criteria

- [ ] `"pr_or_epic_merge"` is a valid value for `completion` on a transition in `workflow.toml` and is parsed without error
- [ ] When `completion = "pr_or_epic_merge"` and `target_branch` is absent: behaviour is identical to `completion = "pr"` — branch is pushed and a PR is opened targeting the default branch
- [ ] When `completion = "pr_or_epic_merge"` and `target_branch` is set in frontmatter: branch is pushed and merged into `target_branch`; no PR is opened
- [ ] Merge conflict on the epic branch: `apm state` aborts cleanly and reports the conflict; the ticket stays in its current state
- [ ] `apm verify` lists `completion = pr_or_epic_merge` for transitions that use it
- [ ] Existing `"pr"`, `"merge"`, and `"none"` strategies are unaffected

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
| 2026-04-02T00:38Z | — | new | philippepascal |
| 2026-04-02T00:38Z | new | groomed | philippepascal |