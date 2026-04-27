+++
id = "941e57fa"
title = "Make pr_or_epic_merge the default strategy and document tradeoffs"
state = "in_design"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/941e57fa-make-pr-or-epic-merge-the-default-strate"
created_at = "2026-04-27T20:27:54.114826Z"
updated_at = "2026-04-27T22:03:42.883507Z"
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
- [ ] `apm-core/src/default/workflow.toml` `in_progress → implemented` transition has `completion = "pr_or_epic_merge"` as the active value
- [ ] `README.md` completion strategy list marks `pr_or_epic_merge` as the default, not `pr`
- [ ] `README.md` includes the four-row strategy tradeoff table (strategies: pr_or_epic_merge, merge, pr, none) matching the table in `docs/strategy-and-dependencies.md` section 'Recommended default'
- [ ] `docs/agents.md` is created and contains the completion strategy tradeoff table
- [ ] The tradeoffs documented in `docs/agents.md` and `README.md` correctly describe dependency-composition safety for each strategy

### Out of scope

- Enforcing strategy/target rules at `apm new --depends-on` or `apm start` (ticket a3dc64db)
- Extending `apm validate` with dependency-rule checks (ticket e845127e)
- Hash-trip on config or workflow change (ticket b10d957a)
- The `apm refresh-epic` command (ticket 2973e208)
- Epic quiescence checks in `apm epic close` (ticket 056b1ee1)
- Removing the per-epic `max_workers` override (ticket 6e3f9e91)
- Any changes to `.rs` Rust source files — behaviour is already implemented

### Approach

Three files change; no Rust source changes are required.

**1. `.apm/workflow.toml` lines 151-152**

Uncomment line 151 (`completion = "pr_or_epic_merge"`) and comment out line 152 (`completion = "merge"`), so that `pr_or_epic_merge` is the active value for the `in_progress → implemented` transition.

**2. `README.md` completion strategy list (currently lines 173-181)**

- Remove `(default)` from the `pr` bullet (line 175).
- Add `(default)` to the `pr_or_epic_merge` bullet (line 177).
- After the closing bullet of the strategy list, insert the four-row tradeoff table from `docs/strategy-and-dependencies.md` section 'Recommended default'. Columns: Strategy, Composes dependencies?, Notes. Rows: pr_or_epic_merge (Yes, within an epic; Default — same strategy yields PR-on-main and merge-to-epic depending on target_branch), merge (Yes, when ticket and deps share target_branch; Lands directly on the target, skips supervisor review on main), pr (No; state→implemented fires when the PR is opened, not when it merges, so downstream tickets can start before upstream code lands), none (No; nothing lands automatically, downstream tickets cannot rely on upstream code being present).

**3. Create `docs/agents.md`**

New file with three parts:
- A one-paragraph summary of the two-tier model: standalone tickets (no epic) target the default branch and get a PR on implemented; epic tickets target the epic branch and merge directly on implemented.
- The same four-row tradeoff table as above.
- A closing note that dependencies are only safe with `pr_or_epic_merge` (within an epic) or `merge` (shared target branch); point to `docs/strategy-and-dependencies.md` for the full dependency rules.

### Open questions


### Amendment requests

default configuration in src/ must have completion = "pr_or_epic_merge

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:27Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T20:44Z | groomed | in_design | philippepascal |
| 2026-04-27T20:47Z | in_design | specd | claude-0427-2044-7318 |
| 2026-04-27T22:03Z | specd | ammend | philippepascal |
| 2026-04-27T22:03Z | ammend | in_design | philippepascal |