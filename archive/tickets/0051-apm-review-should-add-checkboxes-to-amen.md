+++
id = 51
title = "apm review should add checkboxes to amendment requests"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "claude-0328-1430-a4f2"
agent = "claude-0329-1430-main"
branch = "ticket/0051-apm-review-should-add-checkboxes-to-amen"
created_at = "2026-03-28T22:35:26.680433Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

When a supervisor runs `apm review <id>` and transitions a ticket to `ammend`, they write amendment requests as free-form text in `### Amendment requests`. These are often plain bullet points (`- item`) rather than checkboxes (`- [ ] item`).

The problem is that `apm state <id> specd` (the agent's resubmit command) calls `doc.unchecked_amendments()`, which only counts items with `- [ ]` or `- [x]` syntax. If the supervisor wrote plain bullets, the check passes immediately without the agent doing any work — the guard has no teeth.

`apm review` is the natural enforcement point: after the supervisor saves and chooses `ammend`, the tool should normalise any plain `- ` bullets in the amendment section into `- [ ] ` checkboxes before committing, so the format is always consistent and the guard in `state.rs` is always effective.

### Acceptance criteria

- [x] When `apm review` transitions a ticket to `ammend`, plain list items (`^- ` lines that are not already `- [ ]` or `- [x]`) inside `### Amendment requests` are converted to `- [ ] `
- [x] Items already formatted as `- [ ]` or `- [x]` are left unchanged
- [x] Lines that are not list items (blank lines, prose, HTML comments) are left unchanged
- [x] If `### Amendment requests` is absent or contains only the placeholder comment, no conversion is attempted
- [x] Transitions to any state other than `ammend` are not affected
- [x] Integration test: after a `review` call targeting `ammend`, a plain bullet in the amendment section appears as `- [ ]` in the committed ticket

### Out of scope

- Changing how `apm state <id> ammend` handles the amendment section
- Changing `unchecked_amendments()` or any other validation logic
- Converting items in any section other than `### Amendment requests`
- Retroactively fixing tickets already in `ammend` state with plain bullets

### Approach

In `apm/src/cmd/review.rs`, after `extract_spec` and before the spec commit, when the chosen target state is `ammend`:

1. Locate `### Amendment requests` in `new_spec`. If absent, skip.
2. Walk lines of that section (stop at next `##` heading or end of string).
3. For each line matching `^- ` that does not start with `- [ ]` or `- [x]`, insert `[ ] ` after the `- `.
4. Reassemble the spec with converted lines.

The normalised spec is used for the changed-body check and the commit. The existing `ensure_amendment_section` in `state.rs` still runs afterwards via `super::state::run` and inserts the placeholder if absent — no change needed there.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T22:35Z | — | new | claude-0328-1430-a4f2 |
| 2026-03-29T22:56Z | new | in_design | claude-spec-51 |
| 2026-03-29T22:58Z | in_design | specd | claude-spec-51 |
| 2026-03-30T00:53Z | specd | ready | apm |
| 2026-03-30T00:55Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-30T00:59Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-30T01:02Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |