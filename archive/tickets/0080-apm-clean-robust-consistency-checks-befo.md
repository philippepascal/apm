+++
id = 80
title = "apm clean: robust consistency checks before acting"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
agent = "claude-0330-0245-main"
branch = "ticket/0080-apm-clean-robust-consistency-checks-befo"
created_at = "2026-03-30T02:40:07.826350Z"
updated_at = "2026-03-30T05:24:26.579800Z"
+++

## Spec

### Problem

`apm clean` removes local worktrees and branches for closed tickets. Its current
logic has two independent checks — terminal state (read from the ticket branch)
and merged status (branch tip reachable from main) — that it runs separately and
trusts unconditionally. When either check produces a false positive (e.g. a
branch made reachable via `git merge -s ours`, or a sync that incorrectly wrote
`closed` to a branch), `apm clean` acts on active tickets and destroys work.

Observed failures:
- Tickets #72-74 had `closed` written to their branches by `apm sync` even
  though they were never accepted. A subsequent `git merge -s ours` made their
  tips reachable from main. `apm clean` then treated them as safe to remove.
- `apm clean` did not cross-check that the state on the branch agrees with the
  state on main, or that the branch was merged through a legitimate PR workflow.

### Acceptance criteria

- [x] Before removing anything, `apm clean` cross-checks: the ticket state on
  its branch AND on main must both be `closed`; a mismatch is flagged as
  inconsistent and skipped with a clear message
- [x] `apm clean` checks that the branch tip is a git ancestor of main via
  `git merge-base --is-ancestor`, not just via tracking-ref reachability
- [x] Tickets whose branch is reachable from main but whose state disagrees
  between branch and main produce an actionable warning naming the exact
  inconsistency and suggesting a manual fix (e.g. `apm close <id>`)
- [x] Tickets whose local branch tip differs from the remote tracking ref tip
  produce a warning and are skipped
- [x] `--dry-run` output includes the ticket state alongside the branch name so
  it is auditable: `would remove ticket/0080 (state: closed)`
- [x] `closed` is treated as terminal unconditionally, regardless of whether it
  appears in `[[workflow.states]]` in `apm.toml`
- [x] All existing `apm clean` integration tests continue to pass; new tests
  cover the inconsistency detection cases
- [x] `cargo test --workspace` passes

### Out of scope

- Automatically repairing inconsistent tickets (that is `apm sync` / `apm close`)
- Deleting remote branches
- Bulk-close or force-clean flags

### Approach

**Cross-check state on branch vs main**

After identifying a candidate ticket (closed on its branch, branch merged into
main), read the ticket file from main as well. If the state on main differs from
the state on the branch, emit a warning and skip:

```
warning: ticket/0072 state mismatch — branch=closed main=new — run `apm close 72` to reconcile
```

**Ancestor check**

Replace the `merged_into_main` reachability check with an explicit
`git merge-base --is-ancestor <branch-tip> HEAD` call. This catches the
`git merge -s ours` false-positive: that creates a merge commit whose
*parent* is the branch tip, making the tip reachable — but the tip is not
an ancestor of HEAD in the traditional sense only if it was squash-merged or
force-merged without content.

Wait — actually `--is-ancestor` returns true when the tip IS reachable from
HEAD (i.e., is an ancestor). A `-s ours` merge commit has the branch tip as a
parent, so the tip IS an ancestor. The state cross-check (above) is the
reliable gate; the ancestor check is a belt-and-suspenders addition.

**Local vs remote tip agreement**

If a local branch exists, compare its tip to `origin/<branch>` tip. If they
differ, warn and skip — the branch has unpushed or diverged changes.

**`closed` as unconditional terminal**

In the terminal-state filter, always include `"closed"` regardless of config:

```rust
let terminal_states: std::collections::HashSet<&str> = config
    .workflow.states.iter()
    .filter(|s| s.terminal)
    .map(|s| s.id.as_str())
    .collect::<std::collections::HashSet<_>>()
    .union(&["closed"].into_iter().collect())
    .copied()
    .collect();
```

**`--dry-run` audit output**

Change the dry-run print to include ticket state:

```
would remove worktree /path/to/worktree  (ticket #80, state: closed)
would remove branch ticket/0080-...
```

**Read ticket state from main**

Add a helper `ticket::load_from_branch(root, branch, tickets_dir)` →
`Option<Ticket>`. Read main's copy by calling
`ticket::load_from_branch(root, default_branch, tickets_dir)` keyed by ticket
id. Use this for the cross-check.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T02:40Z | — | new | apm |
| 2026-03-30T02:40Z | new | in_design | apm |
| 2026-03-30T02:41Z | in_design | specd | apm |
| 2026-03-30T02:42Z | specd | ready | apm |
| 2026-03-30T02:46Z | ready | in_progress | claude-0330-0245-main |
| 2026-03-30T02:52Z | in_progress | implemented | claude-0329-1200-a7f2 |
| 2026-03-30T04:38Z | implemented | accepted | apm |
| 2026-03-30T05:24Z | accepted | closed | apm-sync |