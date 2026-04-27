+++
id = "941e57fa"
title = "Make pr_or_epic_merge the default strategy and document tradeoffs"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/941e57fa-make-pr-or-epic-merge-the-default-strate"
created_at = "2026-04-27T20:27:54.114826Z"
updated_at = "2026-04-27T20:44:02.372454Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

The `in_progress → implemented` transition in `.apm/workflow.toml` (line 152) uses `completion = "merge"`, with `pr_or_epic_merge` commented out on line 151. This is inconsistent with the design intent in `docs/strategy-and-dependencies.md`, which designates `pr_or_epic_merge` as the recommended default.

The `merge` strategy always merges directly to the target branch. For standalone tickets (no epic, target = default branch), this bypasses supervisor PR review. For epic tickets it behaves identically to `pr_or_epic_merge`, so `merge` provides no advantage in that context either. `pr_or_epic_merge` implements the two-tier model with a single setting: standalone tickets get a PR against the default branch (supervisor-reviewed, safe for parallel work), and epic tickets get a direct merge to the epic branch (autonomous serial, deps-safe within the epic).

Additionally, `README.md` line 175 marks `pr` as the default strategy and does not explain the tradeoffs between strategies. After this change it should reflect `pr_or_epic_merge` as the default and include the four-row tradeoff table from `docs/strategy-and-dependencies.md`. `docs/agents.md` does not currently exist; this ticket creates it as the canonical agent-facing reference for completion strategy behaviour.

### Acceptance criteria

- [ ] `.apm/workflow.toml` `in_progress → implemented` transition has `completion = "pr_or_epic_merge"` as the active (uncommented) value
- [ ] The old `completion = "merge"` line is removed or commented out in `.apm/workflow.toml`
- [ ] `README.md` completion strategy list marks `pr_or_epic_merge` as the default, not `pr`
- [ ] `README.md` includes the four-row strategy tradeoff table (strategies: pr_or_epic_merge, merge, pr, none) matching the table in `docs/strategy-and-dependencies.md` section 'Recommended default'
- [ ] `docs/agents.md` is created and contains the completion strategy tradeoff table
- [ ] The tradeoffs documented in `docs/agents.md` and `README.md` correctly describe dependency-composition safety for each strategy

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
| 2026-04-27T20:27Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T20:44Z | groomed | in_design | philippepascal |