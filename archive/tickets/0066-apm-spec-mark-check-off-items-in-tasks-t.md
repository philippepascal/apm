+++
id = 66
title = "apm spec --mark: check off items in tasks-type sections"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0066-apm-spec-mark-check-off-items-in-tasks-t"
created_at = "2026-03-29T23:26:12.975776Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

The ticket lifecycle requires agents to check off individual items in `tasks`-type sections — specifically `### Amendment requests` and `### Code review` — as they address each one. This capability does not exist.

The `--check` flag in `apm spec` currently means "validate the spec" (check that all required sections are present), not "mark a checklist item as done". An agent addressing amendments has no way to tick off items without editing the ticket file directly, which requires knowing the full file path and doing a manual git commit.

### Acceptance criteria

- [x] `apm spec <id> --section <name> --mark <item-text>` finds the first unchecked item (`- [ ]`) in `### <name>` whose text matches `<item-text>` (case-insensitive substring) and marks it `- [x]`
- [x] If no unchecked matching item is found, the command exits non-zero with a clear error
- [x] If multiple unchecked items match, the command exits non-zero and lists the ambiguous matches
- [x] The commit message is `ticket(<id>): mark "<item-text>" in <section>`
- [x] Works on any section containing checkbox items, including "Amendment requests", "Code review", and "Acceptance criteria"
- [x] `--mark` without `--section` is an error
- [x] The existing `--check` flag (validate spec) is unaffected
- [x] Integration test: after `apm spec --section "Amendment requests" --mark "Add error handling"`, the matching item is `- [x]` in the committed ticket

### Out of scope

- Unmarking items (`- [x]` → `- [ ]`)
- Fuzzy matching beyond case-insensitive substring

### Approach

1. Add `--mark <text>` argument to the `Spec` subcommand in `apm/src/main.rs`.

2. In `apm/src/cmd/spec.rs`, add `mark_item(body: &str, section: &str, item_text: &str) -> Result<String>`:
   - Locate `### <section>` in the body
   - Scan lines in that section for `- [ ] ` lines where the remainder contains `item_text` (case-insensitive)
   - Return error on zero or multiple matches
   - Replace the matched `- [ ]` with `- [x]`; return updated body

3. In `run`, when `mark` is `Some(text)`: call `mark_item` on the raw ticket body, write updated content back via `commit_to_branch`.

Operate on the raw ticket body string — "Amendment requests" and "Code review" are not modelled as named fields in `TicketDocument`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:26Z | — | new | claude-0329-1430-main |
| 2026-03-29T23:26Z | new | in_design | claude-0329-1430-main |
| 2026-03-29T23:31Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:46Z | specd | ready | apm |
| 2026-03-29T23:56Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-30T00:01Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-30T00:50Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |