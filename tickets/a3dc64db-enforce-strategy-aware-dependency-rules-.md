+++
id = "a3dc64db"
title = "Enforce strategy-aware dependency rules at every write site"
state = "specd"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/a3dc64db-enforce-strategy-aware-dependency-rules-"
created_at = "2026-04-27T20:28:18.110435Z"
updated_at = "2026-04-27T21:07:15.529776Z"
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

Two public functions in `apm-core/src/validate.rs`

No new modules needed; validate.rs is already the home for shared validation logic and accessible as `apm_core::validate::*` from the CLI crate.

**`active_completion_strategy(config: &Config) -> CompletionStrategy`** (pub)
Walk config.workflow.states, find the state with id == "in_progress", find its transition with to == "implemented". Return that transition completion field. If the state or transition is absent, return CompletionStrategy::None (safest default — no deps allowed when the workflow deviates from standard). Ticket e845127e (apm validate dep-rule checks) will call this same function.

**`check_depends_on_rules(strategy, ticket_epic, ticket_target_branch, dep_ids, all_tickets, default_branch) -> Result<()>`** (pub)

Exact signature: strategy: &CompletionStrategy, ticket_epic: Option<&str>, ticket_target_branch: Option<&str>, dep_ids: &[String], all_tickets: &[Ticket], default_branch: &str

Rules per strategy:
- Pr | None | Pull: reject any non-empty dep_ids; error "depends_on is not allowed under the <strategy> completion strategy".
- PrOrEpicMerge: if ticket_epic is None, error "pr_or_epic_merge requires the ticket to belong to an epic before depends_on can be set". For each dep ID: look it up in all_tickets (not found = error "dep <id> not found"); check dep epic == ticket_epic. Collect mismatches. On any violation, single error "pr_or_epic_merge requires all deps to share epic <epic>; offending deps: <id1>, <id2>".
- Merge: normalize both sides using default_branch when target_branch is None. Collect deps whose target differs. On any violation: "merge requires all deps to share target_branch <branch>; offending deps: <id1>, <id2>".

Ticket is importable in validate.rs via `use crate::ticket_fmt::Ticket;`.

**Write-site: apm/src/cmd/new.rs**

In run(), after depends_on_parsed and epic_id/target_branch are resolved (around line 48), and before the call to ticket::create:
- Guard: only run when depends_on_parsed is Some and non-empty.
- Call `apm_core::ticket::load_all_from_git(root, &config.tickets.dir)` to get existing tickets.
- Call `active_completion_strategy(&config)` to get the strategy.
- Call `check_depends_on_rules` with strategy, epic_id.as_deref(), target_branch.as_deref(), dep IDs, loaded tickets, and config.project.default_branch.
- Propagate error with `?`.
When depends_on is empty (the common case) the ticket load is skipped entirely. new.rs calls CmdContext::load_config_only which already suffices since the ticket load is done separately above.

**Write-site: apm/src/cmd/set.rs**

In run(), before the call to ticket::set_field (around line 20), add a pre-check for field == "depends_on" and value != "-":
- Parse dep IDs from value: split on comma, trim, filter empty (same logic as in set_field).
- If the parsed list is non-empty, call active_completion_strategy and check_depends_on_rules.
- Use t.frontmatter.epic.as_deref(), t.frontmatter.target_branch.as_deref(), &tickets (ctx.tickets, already loaded), and ctx.config.project.default_branch.
- Bail on error before any mutation occurs.

**Tests (9 new unit tests in apm-core/src/validate.rs `#[cfg(test)]`)**

Follow the existing pattern: build Config from a TOML string, build Ticket objects with Frontmatter directly.

- strategy_finds_in_progress_to_implemented: config with in_progress -> implemented completion = "pr_or_epic_merge" returns PrOrEpicMerge
- strategy_defaults_to_none_when_absent: config with no in_progress state returns None
- dep_rules_pr_rejects_dep: strategy Pr, one dep ticket -> Err
- dep_rules_none_rejects_dep: strategy None, one dep ticket -> Err
- dep_rules_pr_or_epic_merge_same_epic_ok: ticket and dep both in epic "abc" -> Ok
- dep_rules_pr_or_epic_merge_different_epic_fails: dep in epic "xyz", ticket in epic "abc" -> Err naming the dep ID
- dep_rules_pr_or_epic_merge_ticket_no_epic_fails: ticket has no epic -> Err
- dep_rules_merge_both_default_branch_ok: both target_branch None -> Ok
- dep_rules_merge_different_target_fails: dep has different target_branch -> Err naming the dep ID

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T20:57Z | groomed | in_design | philippepascal |
| 2026-04-27T21:07Z | in_design | specd | claude-0427-2057-0500 |
