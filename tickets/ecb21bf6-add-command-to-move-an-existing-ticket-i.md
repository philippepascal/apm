+++
id = "ecb21bf6"
title = "Add command to move an existing ticket into an epic"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ecb21bf6-add-command-to-move-an-existing-ticket-i"
created_at = "2026-04-17T18:48:52.510757Z"
updated_at = "2026-04-17T18:50:16.107010Z"
+++

## Spec

### Problem

APM has no first-class command to associate an already-created ticket with an epic. Epic membership can only be set at ticket creation via `apm new --epic <epic_id>` — there is no `apm set <id> epic <epic_id>` and no `apm ticket move --epic <epic_id>` equivalent.

This matters because epic association is not just a metadata hint: when a ticket is created with `--epic`, its branch is forked from the epic's branch tip, so the ticket's code lands inside the epic's merge scope. A ticket created without `--epic` has its branch forked from `main`. Retroactively patching only the frontmatter would leave `apm epic show` and `apm list --epic <id>` claiming membership that the branch topology contradicts.

The workaround today is manual: close the standalone ticket, create a replacement with `apm new --epic <epic_id>`, and copy the Problem / AC / OOS / Approach across. This is tedious, risks content drift on the copy, and loses the original ticket's branch (and any commits already on it).

A proper move command should: (a) create a new branch forked from the epic tip, (b) cherry-pick or graft any commits from the original ticket branch onto the new branch, (c) move the spec content into a new ticket file with a fresh ID stamped with `epic = "<id>"`, (d) close/archive the old ticket cleanly, and (e) leave history pointers so the operation is auditable.

Trigger: user hit this on 2026-04-17 after creating ticket `3d73a43b` in parallel with epic `47375a6a` and wanting to bring it in.

This command should apply the same method to move a ticket to another epic, or out of an epic.

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
