+++
id = "328b3c71"
title = "apm work --dry-run: show all candidates up to max-workers"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
agent = "philippepascal"
branch = "ticket/328b3c71-apm-work-dry-run-show-all-candidates-up-"
created_at = "2026-03-30T16:31:01.147894Z"
updated_at = "2026-03-30T17:02:35.295368Z"
+++

## Spec

### Problem

Currently, `apm work --dry-run` shows only the single highest-priority ticket that the work loop would dispatch next. When a project has `max_concurrent > 1`, the work loop actually starts multiple workers in parallel — but the dry-run output gives no indication of which tickets those would be.

This makes dry-run nearly useless as a preview tool. Users who want to sanity-check what `apm work` would do before running it for real cannot see the full picture: they only see the first dispatch, not the second or third.

The fix is to have `--dry-run` show up to `max_concurrent` candidates in priority order — matching what the actual work loop would start.

### Acceptance criteria

- [ ] `apm work --dry-run` prints one line per candidate, up to `max_concurrent` candidates
- [ ] Each output line identifies the ticket by id, state, and title (matching the format `dry-run: would start next: #<id> [<state>] <title>`)
- [ ] When fewer actionable tickets exist than `max_concurrent`, only the available tickets are printed (no padding or error)
- [ ] When there are no actionable tickets, the output is `dry-run: no actionable tickets` (existing behaviour preserved)
- [ ] Candidates are printed in priority order (highest score first), matching the order the work loop would start them
- [ ] The command exits 0 in all cases above

### Out of scope

- Changes to `apm next` — it continues to return a single ticket
- Changes to the non-dry-run `apm work` behaviour
- Filtering by currently running workers (dry-run does not check for live processes)
- New output formats (JSON, table, etc.)

### Approach

Modify `run_dry` in `apm/src/cmd/work.rs`:

1. Read `max_concurrent` from `config.agents.max_concurrent` (already available via the `config` argument).
2. Collect all candidates matching `actionable` + `startable` filters, sort by score descending, then take the first `max_concurrent` entries. This avoids calling `pick_next` in a loop.
3. If the resulting list is empty, print `dry-run: no actionable tickets`.
4. Otherwise, print one line per candidate using the existing format string.

The sorting + filtering logic mirrors what `ticket::pick_next` already does internally; inline it rather than calling `pick_next` repeatedly and removing tickets between calls.

Files changed:
- `apm/src/cmd/work.rs` — `run_dry` function only; no other files need to change

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T16:31Z | — | new | philippepascal |
| 2026-03-30T16:39Z | new | in_design | philippepascal |
| 2026-03-30T16:43Z | in_design | specd | claude-0330-1640-b3f2 |
| 2026-03-30T17:01Z | specd | ready | philippepascal |
| 2026-03-30T17:02Z | ready | in_progress | philippepascal |
