+++
id = "a3dc64db"
title = "Enforce strategy-aware dependency rules at every write site"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a3dc64db-enforce-strategy-aware-dependency-rules-"
created_at = "2026-04-27T20:28:18.110435Z"
updated_at = "2026-04-27T20:57:29.580622Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

`apm new --depends-on` and `apm set <id> depends_on …` currently accept dependencies regardless of the configured completion strategy. The spec at `docs/strategy-and-dependencies.md` (section 'Dependency rules per strategy') defines when dependencies compose safely:

- pr_or_epic_merge: ticket and all deps must share an epic
- merge: ticket and all deps must share target_branch (same epic, or all on default)
- pr / none: --depends-on is rejected outright

Implement the rule check at every site where `depends_on` is written:

- `apm new` — `apm-core/src/ticket/ticket_util.rs::create`, around the `depends_on` parameter
- `apm set <id> depends_on <ids>` — wherever the `set` subcommand handles the `depends_on` field

Reject violations with a clear message naming the offending dep IDs and the rule that was broken.

Re-validating at `apm start` is **not** required: the hash-trip / `apm validate` mechanism (separate ticket) catches the post-hoc case where a previously-valid setup becomes invalid after a config change. Validating at every write site plus the hash-trip is sufficient and avoids redundant checks on a hot path.

The strategy-determination logic (read from `workflow.toml` to find the `in_progress → implemented` transition's `completion` field) and the per-strategy rule check should live in a single helper that the write paths and `apm validate` (separate ticket) all share.

See docs/strategy-and-dependencies.md, section 'Dependency rules per strategy'.

### Acceptance criteria

- [ ] `apm new --depends-on <dep> --epic <epic>` succeeds when `<dep>` belongs to the same epic as the new ticket under `pr_or_epic_merge` strategy
- [ ] `apm new --depends-on <dep> --epic <epic>` fails with a message naming `<dep>` and the violated rule when `<dep>` belongs to a different (or no) epic under `pr_or_epic_merge` strategy
- [ ] `apm new --depends-on <dep>` (no `--epic`) fails under `pr_or_epic_merge` strategy because the new ticket is not in any epic
- [ ] `apm new --depends-on <dep>` fails unconditionally under `pr` strategy, naming `<dep>` and the rule
- [ ] `apm new --depends-on <dep>` fails unconditionally under `none` strategy, naming `<dep>` and the rule
- [ ] `apm new --depends-on <dep>` succeeds under `merge` strategy when ticket and `<dep>` share the same `target_branch` (including both having no `target_branch`, i.e. both targeting the default branch)
- [ ] `apm new --depends-on <dep>` fails under `merge` strategy when ticket and `<dep>` have different `target_branch` values, naming `<dep>` and the rule
- [ ] `apm set <id> depends_on <dep>` succeeds when strategy rules are satisfied for existing ticket `<id>`
- [ ] `apm set <id> depends_on <dep>` fails with a message naming `<dep>` and the violated rule when the dep violates strategy rules for `<id>`
- [ ] `apm set <id> depends_on -` always succeeds regardless of strategy (clearing deps is always allowed)
- [ ] Error messages name the offending dep IDs and state the violated rule (e.g. "dep abc123 not in epic xyz789; pr_or_epic_merge requires all deps to share the ticket epic")
- [ ] `active_completion_strategy` and `check_depends_on_rules` are public exports from `apm_core::validate`, not inlined in the CLI handlers

### Out of scope

- Enforcing dependency rules at `apm start` time (the hash-trip / `apm validate` mechanism, tickets b10d957a and e845127e, handles post-hoc drift after config changes)
- Extending `apm validate` to check existing tickets against strategy rules across the whole ticket set (ticket e845127e)
- Hash-trip on config or workflow change triggering automatic re-validation (ticket b10d957a)
- Changing the default completion strategy to `pr_or_epic_merge` (ticket 941e57fa)
- Removing the per-epic `max_workers` override (ticket 6e3f9e91)
- Epic quiescence checks in `apm epic close` or `apm refresh-epic` (tickets 056b1ee1, 2973e208)
- Supporting user-defined state names other than `in_progress`/`implemented` for strategy lookup (the helper reads exactly the `in_progress -> implemented` transition)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T20:57Z | groomed | in_design | philippepascal |