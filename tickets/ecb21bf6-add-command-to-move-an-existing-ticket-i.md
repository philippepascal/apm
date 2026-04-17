+++
id = "ecb21bf6"
title = "Add command to move an existing ticket into an epic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ecb21bf6-add-command-to-move-an-existing-ticket-i"
created_at = "2026-04-17T18:48:52.510757Z"
updated_at = "2026-04-17T18:54:32.174962Z"
+++

## Spec

### Problem

APM has no first-class command to associate an already-created ticket with an epic. Epic membership can only be set at ticket creation via `apm new --epic <epic_id>` — there is no post-creation move command.\n\nThis matters because epic association is not just a metadata hint: when a ticket is created with `--epic`, its branch is forked from the epic's branch tip, so the ticket's code lands inside the epic's merge scope. A ticket created without `--epic` has its branch forked from `main`. Retroactively patching only the frontmatter would leave `apm epic show` and branch topology out of sync.\n\nThe workaround today is manual: close the standalone ticket, create a replacement with `apm new --epic <epic_id>`, and copy the spec content. This is tedious, risks content drift, and loses the original ticket's branch and any commits on it.\n\nA proper move command should: (a) fork a new branch base from the target epic (or `main` when removing from an epic), (b) replay any commits from the original ticket branch onto the new base via `git rebase --onto`, (c) update the ticket file's frontmatter in place (`epic`, `target_branch`, history row), and (d) leave the same ticket ID — keeping any `depends_on` references intact. This is consistent with how the rest of APM works: epic membership is read from the `epic` frontmatter field, so updating both the frontmatter and the branch topology in one atomic command fully re-seats the ticket.

### Acceptance criteria

Checkboxes; each one independently testable.

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
| 2026-04-17T18:48Z | — | new | philippepascal |
| 2026-04-17T18:50Z | new | groomed | apm |
| 2026-04-17T18:54Z | groomed | in_design | philippepascal |