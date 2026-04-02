+++
id = "0c2ef539"
title = "Add pr_or_epic_merge completion strategy"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
branch = "ticket/0c2ef539-add-pr-or-epic-merge-completion-strategy"
created_at = "2026-04-02T00:38:36.244478Z"
updated_at = "2026-04-02T00:40:10.014134Z"
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

- Per-ticket `completion` override in frontmatter
- Any change to how `"pr"` or `"merge"` strategies behave
- Squash or rebase merge options
- Auto-transitioning after merge

### Approach

**`apm-core/src/config.rs`** — add `PrOrEpicMerge` variant to `CompletionStrategy` enum. Ensure it deserializes from `"pr_or_epic_merge"` and serializes back to the same string.

**`apm-core/src/state.rs`** — in the `match completion` block, add:

```rust
CompletionStrategy::PrOrEpicMerge => {
    git::push_branch(root, &branch)?;
    if let Some(ref target) = t.frontmatter.target_branch {
        merge_into_default(root, &branch, target)?;
    } else {
        gh_pr_create_or_update(root, &branch, &config.project.default_branch, &id, &t.frontmatter.title)?;
    }
}
```

`merge_into_default` and `gh_pr_create_or_update` already exist and accept the target branch as a parameter — no new git logic needed.

**`apm/src/cmd/verify.rs`** — the existing loop that prints `completion` values already handles any `CompletionStrategy` variant via `Display`; add `PrOrEpicMerge` to the `Display` impl and it appears automatically.

**`workflow.toml`** — change `completion = "pr"` on `in_progress → implemented` to `completion = "pr_or_epic_merge"`.

**Tests** — in `apm-core` unit tests or integration tests, add:
- `pr_or_epic_merge` with no `target_branch` → confirm PR path taken
- `pr_or_epic_merge` with `target_branch` set → confirm merge path taken

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T00:38Z | — | new | philippepascal |
| 2026-04-02T00:38Z | new | groomed | philippepascal |
| 2026-04-02T00:40Z | groomed | in_design | philippepascal |
| 2026-04-02T00:40Z | in_design | specd | philippepascal |
