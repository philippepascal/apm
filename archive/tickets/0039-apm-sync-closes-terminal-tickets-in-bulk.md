+++
id = 39
title = "apm sync closes terminal tickets in bulk"
state = "closed"
priority = 0
effort = 3
risk = 1
author = "claude-0328-1000-a1b2"
agent = "claude-0328-impl-a1b2"
branch = "ticket/0039-apm-sync-closes-terminal-tickets-in-bulk"
created_at = "2026-03-28T08:07:08.341760Z"
updated_at = "2026-03-28T08:44:25.581740Z"
+++

## Spec

### Problem

Ticket state transitions after a PR is merged never land on `main`. Specifically:

1. `implemented → accepted`: supervisor approves the PR and transitions the
   ticket, but that commit goes to the ticket branch. After merge, `main` still
   shows `implemented`.
2. `accepted → closed`: no mechanism exists to write the final state to `main`.

When the ticket branch is eventually deleted, `main` is left with a stale
ticket file — typically frozen at `implemented` forever. There is also a
subtler case: if a ticket's branch is already gone (merged and deleted) but
`main` shows `implemented`, `apm sync` has no way to know that no more work
is coming and cannot close it automatically.

The solution is to give `apm sync` the ability to detect these conditions and
batch-commit the necessary state updates to `main` in a single commit — keeping
git noise low and keeping `main`'s ticket files accurate.

### Acceptance criteria

- [x] `apm sync` detects tickets in `accepted` state (on any branch) and
  proposes to transition them to `closed`, writing the updated ticket file
  to `main`
- [x] `apm sync` detects tickets where `main` shows `implemented` but no
  ticket branch exists (branch was deleted after merge) and proposes to
  transition them to `closed` on `main`
- [x] By default (interactive), `apm sync` prints a summary of proposed
  closures and prompts for confirmation before committing
- [x] `apm sync --auto-close` skips the prompt and applies all closures
  automatically
- [x] All closure updates in a single sync run are batched into **one commit**
  to `main` (not one commit per ticket)
- [x] The batch commit message lists the closed ticket IDs:
  `"apm sync: close tickets #32, #34, #36"`
- [x] `apm sync --quiet --auto-close` applies closures silently with no output
- [x] If there is nothing to close, no commit is made and no prompt is shown

### Out of scope

- Closing tickets in states other than `accepted` or `implemented`-with-no-branch
- Archiving or deleting ticket branches (branch cleanup is a separate concern)
- Retroactively fixing stale ticket files already on `main` from before this
  feature (one-time migration, not ongoing sync behaviour)
- Any changes to how `apm state` works — this is sync-only

### Approach

**Detection logic** in `apm/src/cmd/sync.rs`:

After the existing fetch + cache-refresh loop, add a pass over all known tickets:

```
for each ticket in cache:
    if ticket.state == "accepted":
        → mark for closure (read ticket file from ticket branch)
    else if ticket.state == "implemented"
         && no branch named ticket.branch exists locally or remotely:
        → mark for closure (read ticket file from main)
```

**Closure write**:

For each ticket marked for closure:
1. Read the current ticket file from its source (branch or main)
2. Set `state = "closed"`, update `updated_at`
3. Append history row: `| <now> | <prev_state> | closed | apm-sync |`
4. Write the file to the working tree at the main branch path

Commit all changes in one shot:
```
git add tickets/...
git commit -m "apm sync: close tickets #N, #M, ..."
```

**Interactive prompt** (no `--auto-close`):

```
Tickets ready to close:
  #32  apm-ticket-format-in-core   (accepted)
  #34  new-command-take-free-text  (implemented, branch gone)

Close all? [y/N]
```

`--auto-close` skips the prompt entirely.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T08:07Z | — | new | claude-0328-1000-a1b2 |
| 2026-03-28T08:08Z | new | specd | claude-0328-1000-a1b2 |
| 2026-03-28T08:19Z | specd | ready | apm |
| 2026-03-28T08:23Z | ready | in_progress | claude-0328-impl-a1b2 |
| 2026-03-28T08:36Z | in_progress | implemented | claude-0328-impl-a1b2 |
| 2026-03-28T08:44Z | implemented | closed | claude-0328-impl-a1b2 |