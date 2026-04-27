+++
id = "e845127e"
title = "Extend apm validate to enforce dependency rules across tickets"
state = "specd"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e845127e-extend-apm-validate-to-enforce-dependenc"
created_at = "2026-04-27T20:28:41.454959Z"
updated_at = "2026-04-27T21:24:30.907626Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
depends_on = ["a3dc64db"]
+++

## Spec

### Problem

`apm validate` currently checks config correctness, ticket state validity, and branch-field consistency. It does not check whether existing tickets' `depends_on` fields satisfy the rules for the currently configured completion strategy.

The spec at `docs/strategy-and-dependencies.md` (section Dependency rules per strategy) defines when dependencies compose safely: under `pr_or_epic_merge`, all deps must share the ticket's epic; under `merge`, all deps must share the ticket's `target_branch`; under `pr` or `none`, no deps are allowed at all. These rules are enforced at write time by ticket a3dc64db, but tickets created before that enforcement existed -- or tickets whose config changed after creation -- can violate the rules silently.

This ticket extends `apm validate` to walk every non-closed ticket and report each one whose `depends_on` violates the active strategy rule. Ticket a3dc64db provides `active_completion_strategy()` and `check_depends_on_rules()` in `apm-core/src/validate.rs`; this ticket adds a sweep function that calls them over all loaded tickets, keeping the rule logic in a single place shared by both the write-time guards and the full-scan validator.

### Acceptance criteria

- [ ] `apm validate` reports an error for each non-closed ticket whose `depends_on` is non-empty and violates the active completion strategy rule
- [ ] `apm validate` reports no `depends_on` error for tickets with an empty or absent `depends_on`
- [ ] `apm validate` skips tickets in the `closed` state when checking `depends_on`
- [ ] `apm validate` reports a `depends_on` error when a dep ID in `depends_on` is not found in the loaded ticket set
- [ ] `apm validate --json` includes each dependency violation in the errors array with kind = depends_on
- [ ] Human-readable output for dependency violations follows the existing format: error [depends_on] #id: message
- [ ] A ticket with a `depends_on` that satisfies the active strategy (correct epic for `pr_or_epic_merge`, correct `target_branch` for `merge`) produces no `depends_on` error
- [ ] When the strategy is `pr` or `none`, any ticket with a non-empty `depends_on` is flagged
- [ ] `apm validate --config-only` does not run dependency checks (tickets are not loaded in config-only mode)
- [ ] `apm validate` exits with a non-zero exit code when any `depends_on` violation is found
- [ ] A pub fn validate_depends_on(config: &Config, tickets: &[Ticket]) -> Vec<(String, String)> exists in apm-core/src/validate.rs with at least 7 unit tests covering: no deps, closed ticket skipped, pr_or_epic_merge same-epic passes, pr_or_epic_merge cross-epic fails, merge same-target passes, merge different-target fails, pr strategy rejects any dep

### Out of scope

- Implementing active_completion_strategy() and check_depends_on_rules() -- those are ticket a3dc64db
- Hash-trip re-validation triggered by config or workflow changes -- ticket b10d957a
- Auto-fix (--fix) for dependency violations -- no safe automatic correction exists
- Enforcing dependency rules at write time (apm new, apm set) -- ticket a3dc64db
- Changing the default completion strategy to pr_or_epic_merge -- ticket 941e57fa
- Epic quiescence checks in apm epic close or apm refresh-epic -- tickets 056b1ee1, 2973e208
- Removing the per-epic max_workers override -- ticket 6e3f9e91

### Approach

**1. Add validate_depends_on to apm-core/src/validate.rs**

Add use crate::ticket::Ticket; at the top of the file outside #[cfg(test)]
(a3dc64db may already add this import; add it if not already present after that ticket lands).

New public function:

    pub fn validate_depends_on(
        config: &Config,
        tickets: &[Ticket],
    ) -> Vec<(String, String)>  // (subject "#<id>", error message)

Body:
1. Call active_completion_strategy(config) once.
2. For each ticket: skip if fm.state == "closed"; skip if fm.depends_on is None or empty.
3. Collect dep IDs as &[String] from the depends_on vec.
4. Call check_depends_on_rules(&strategy, fm.epic.as_deref(), fm.target_branch.as_deref(), &dep_ids, tickets, &config.project.default_branch).
5. On Err(e), push (format!("#{}", fm.id), e.to_string()) into the result vec.
6. Return the vec.

**2. Wire into apm/src/cmd/validate.rs**

In run(), in the else branch (full ticket load), after the existing loop over tickets (after line 76):

    for (subject, message) in apm_core::validate::validate_depends_on(&config, &tickets) {
        ticket_issues.push(Issue {
            kind: "depends_on".into(),
            subject,
            message,
        });
    }

Add validate_depends_on to the existing pub use apm_core::validate:: imports at the top
of validate.rs, or call it with the full path as shown. No other changes to the CLI crate.

**3. Unit tests in apm-core/src/validate.rs #[cfg(test)]**

Add a make_ticket helper inside the existing mod tests block. It takes (id, state, epic, target_branch, depends_on)
and builds a Ticket via Ticket::parse from an inline TOML string assembled from those fields.

Add a config_with_strategy(strategy: &str) -> Config helper that builds a minimal Config with
the named strategy on the in_progress -> implemented transition; include provider = "github" under
[git_host] for strategies that require a git provider (pr, merge, pr_or_epic_merge).

Seven tests:

- validate_depends_on_no_deps_clean
  Two tickets with no depends_on, pr_or_epic_merge strategy -> empty result.

- validate_depends_on_closed_ticket_skipped
  Closed ticket with a dep that would violate pr strategy -> empty result.

- validate_depends_on_pr_or_epic_merge_same_epic_ok
  Ticket and dep both in epic = "abc", strategy pr_or_epic_merge -> empty result.

- validate_depends_on_pr_or_epic_merge_cross_epic_fails
  Dep has epic = "xyz", ticket has epic = "abc", strategy pr_or_epic_merge -> one violation.
  The violation message must contain the dep ticket ID.

- validate_depends_on_merge_same_target_ok
  Both target_branch = "feat", strategy merge -> empty result.

- validate_depends_on_merge_different_target_fails
  Dep has target_branch = "other", ticket has target_branch = "feat", strategy merge -> one violation.

- validate_depends_on_pr_strategy_rejects_any_dep
  Strategy pr, ticket has one dep -> one violation.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T21:16Z | groomed | in_design | philippepascal |
| 2026-04-27T21:24Z | in_design | specd | claude-0427-2116-b858 |
