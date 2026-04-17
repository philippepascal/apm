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

- [ ] `apm move <ticket_id> <epic_id>` moves a standalone ticket into the named epic: the ticket's `epic` frontmatter field is set to the target epic's ID and `target_branch` is set to the epic's branch name
- [ ] After `apm move <ticket_id> <epic_id>`, the ticket's git branch is forked from the target epic's branch tip (i.e. `git merge-base <ticket-branch> <epic-branch>` equals the epic branch tip at the moment of the move)
- [ ] Commits that existed on the original ticket branch and are not part of the old base are replayed on the new branch in the same order
- [ ] After `apm move <ticket_id> <epic_id>`, `apm epic show <epic_id>` lists the ticket
- [ ] After `apm move <ticket_id> <epic_id>`, the ticket's `## History` section contains a new row recording the move (from-epic or from-main, to-epic)
- [ ] `apm move <ticket_id> -` clears the `epic` and `target_branch` fields in the ticket's frontmatter and rebases the branch onto `main`
- [ ] After `apm move <ticket_id> -`, `apm epic show <old_epic_id>` no longer lists the ticket
- [ ] `apm move <ticket_id> <epic_id_2>` when the ticket is already in `<epic_id_1>` moves it to `<epic_id_2>` (both frontmatter and branch topology)
- [ ] `apm move <ticket_id> <epic_id>` when the ticket is already in `<epic_id>` exits with an informative message and makes no changes
- [ ] `apm move <ticket_id> -` when the ticket has no epic exits with an informative message and makes no changes
- [ ] `apm move <closed_ticket_id> <epic_id>` exits with a clear error (cannot move a terminal ticket)
- [ ] `apm move <ticket_id> <nonexistent_epic>` exits with a clear error
- [ ] When rebase conflicts occur, the command fails with a clear error message, runs `git rebase --abort`, and leaves the repository in a clean state with no partial branches or uncommitted changes

### Out of scope

- Automatic conflict resolution when replaying commits onto the new base (command fails cleanly; user must resolve by hand or create a fresh ticket with `apm new --epic`)\n- Updating remote branches or open pull requests to reflect the rebased branch\n- Bulk-moving multiple tickets into an epic in one invocation\n- Moving an epic itself (changing an epic's parent or merge target)\n- A frontmatter-only `apm set epic <id>` shortcut that skips the rebase — this would silently break branch topology\n- Moving a ticket that is already `in_progress` with code commits that conflict on the new base (same failure path as above)

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