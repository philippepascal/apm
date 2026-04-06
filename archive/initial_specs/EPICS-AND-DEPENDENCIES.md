# APM — Epics and Ticket Dependencies

> **Status:** Draft · **Date:** 2026-03-31
> Extends the V3 spec with two interlocking features: **epics** (shared development
> branches for groups of related tickets) and **depends_on** (explicit dispatch-
> time ordering between tickets).

---

## Motivation

APM dispatches tickets in priority order. But priority is a hint, not a
guarantee: a worker assigned to ticket B can be dispatched before ticket A
merges to `main`, even when B's code cannot compile without A's changes.

Two problems need to be solved together:

1. **Ordering**: worker on B must not start until A is merged.
2. **Pollution**: merging a dozen incremental UI tickets one-by-one to `main`
   produces noise and makes bisecting harder. An "epic" of related tickets
   should accumulate on a shared development branch and merge to `main` in one
   logical unit.

---

## Concepts

### Epic

An epic is a named development branch (`epic/<slug>`) that groups a set of
tickets. Tickets in an epic:

- Branch off the epic branch (not `main`) when their worktree is provisioned.
- Open their PR targeting the epic branch (not `main`).
- Are merged to the epic branch as they complete, making their work immediately
  available to subsequent tickets in the same epic.

When all tickets in the epic are closed, the epic branch itself is PR'd to
`main` in a single merge.

### depends_on

A ticket can declare that it depends on one or more other tickets:

```toml
depends_on = ["36ea9bdb", "54eb5bfc"]
```

`apm work` (and `apm start --next`) will not dispatch a ticket until all of its
dependencies are in a state that makes their output available on the target
branch. Specifically:

- If the dependency is **in the same epic**: its state must be `closed` (PR
  merged to the epic branch).
- If the dependency is **outside the epic** (or there is no epic): its state
  must be `closed` with its PR merged to `main`.

Dependencies only block dispatch; they do not affect the state machine
transitions a supervisor or agent can trigger manually.

---

## Data model changes

### Epic record

Epics are stored as files in the `epics/` directory (configurable via
`[epics] dir` in `apm.toml`), on their own `epic/<slug>` branches. Format
mirrors the ticket format:

```toml
+++
id       = "e-ui-foundation"
slug     = "ui-foundation"
title    = "UI foundation"
branch   = "epic/ui-foundation"
state    = "open"          # open | closed
created_at = "2026-03-31T00:00Z"
tickets  = [
  "36ea9bdb",
  "54eb5bfc",
  "ed5c2b3b",
  "e1748434",
]
+++

## Description

Establishes the axum backend skeleton and the React/Vite frontend skeleton,
wired together, with the 3-column layout shell. No business logic — the goal
is a compilable, deployable stack that all subsequent UI tickets can build on.
```

The `tickets` array is append-only and ordered by intended dispatch sequence.
The epic branch is created by `apm epic new`.

### Ticket frontmatter additions

```toml
epic       = "e-ui-foundation"   # optional; epic id this ticket belongs to
depends_on = ["36ea9bdb"]        # optional; list of ticket ids
```

Both fields are optional. A ticket with `depends_on` but no `epic` depends on
tickets that target `main`.

---

## CLI additions

### `apm epic new <title>`

Creates the epic record and the `epic/<slug>` branch.

```
apm epic new "UI foundation"
→ epic/e-ui-foundation created on branch epic/ui-foundation
```

Options:
- `--tickets <id,...>` — pre-populate the ticket list
- `--no-edit` — skip opening `$EDITOR` (agent-safe)

### `apm epic show <epic-id>`

Prints the epic record and each ticket's current state, in order.

```
e-ui-foundation — UI foundation   [open]
  36ea9bdb  closed      apm-server: axum/tokio skeleton
  54eb5bfc  in_progress apm-server: ticket list and detail API
  ed5c2b3b  ready       apm-ui: Vite + React skeleton
  e1748434  ready       apm-ui: 3-column layout
```

### `apm epic list`

Lists all open epics and their progress (N closed / M total).

### `apm epic add <epic-id> <ticket-id>`

Appends a ticket to an existing epic. Sets the ticket's `epic` field and
`depends_on` if the previous ticket in the list is a natural dependency.

### `apm epic close <epic-id>`

Marks the epic closed. Opens (or drafts) the PR from `epic/<slug>` → `main`.
Fails if any ticket in the epic is not `closed`.

### `apm set <ticket-id> depends_on <id,...>`

Sets the `depends_on` field on a ticket. Accepts a comma-separated list of
ticket IDs. Pass an empty string to clear.

---

## Dispatch changes

### `apm next` — dependency resolution

`apm next` already filters by `actionable` state and agent availability. It
must additionally check `depends_on`:

For each candidate ticket in `ready` state:
1. Resolve each id in `depends_on`.
2. Determine the target branch: the ticket's epic branch if `epic` is set,
   otherwise `main`.
3. Check that the dependency's ticket branch is merged into that target branch
   (`git merge-base --is-ancestor`).
4. If any dependency is not yet merged, skip the ticket. It is not dispatchable.

`apm next --json` includes a `blocked_by` array when a ticket is skipped due to
unmet dependencies, so the delegator can report clearly to the supervisor.

### `apm work` daemon

The daemon respects the same resolution. When the queue appears empty but
there are tickets with unresolved dependencies, the daemon stays alive and
re-evaluates on each poll cycle. It logs:

```
[tick] 3 tickets ready but blocked by unmerged dependencies; waiting...
```

---

## Worktree provisioning changes

When `apm start <id>` provisions a worktree for a ticket that belongs to an
epic:

1. The worktree branches off `epic/<slug>` (not `main`).
2. Before provisioning, APM fetches the latest `epic/<slug>` from origin.
3. The PR opened by the worker targets `epic/<slug>`.

When a PR merges into the epic branch, subsequent workers automatically pick up
the merged work because their worktrees are branched from the latest epic
branch at provision time.

---

## Auto-cascade (optional, phase 2)

When a ticket within an epic closes (PR merged to epic branch), APM can
optionally rebase open sibling worktrees onto the updated epic branch:

```toml
[epics]
auto_rebase = true   # default: false
```

This is opt-in because rebasing can cause conflicts that require human
resolution. When `auto_rebase = false`, workers receive the updated epic branch
at the next `apm start`, not retroactively.

---

## `apm.toml` additions

```toml
[epics]
dir            = "epics"       # where epic records are stored
auto_rebase    = false         # rebase open worktrees when epic branch advances
close_strategy = "pr"          # "pr" (open PR to main) | "squash" | "merge"
```

---

## State machine interaction

Epics do not introduce new ticket states. The existing state machine applies
to every ticket, regardless of whether it belongs to an epic.

The epic itself has two states: `open` and `closed`. These are not part of the
ticket state machine and are not configurable — they reflect whether all
tickets are closed and whether the epic branch has been merged to `main`.

---

## Example: UI foundation epic

```
epic/ui-foundation  ──────────────────────────────────────────────────► main
       │                                                               (1 PR)
       ├── ticket/36ea9bdb (axum skeleton)      ──── merged ──┐
       │                                                       │
       ├── ticket/54eb5bfc (ticket API)          ─── merged ──┤  (depends_on: 36ea9bdb)
       │                                                       │
       ├── ticket/ed5c2b3b (React skeleton)      ─── merged ──┤  (depends_on: 54eb5bfc)
       │                                                       │
       └── ticket/e1748434 (3-column layout)     ─── merged ──┘  (depends_on: ed5c2b3b)
```

Workers are dispatched in order. Each one finds the epic branch already
contains the previous ticket's changes. The epic branch accumulates all
changes and is PR'd to `main` once `apm epic close e-ui-foundation` is run.

---

## Open questions

- **Epic vs. milestone**: should an epic carry a target date / milestone label,
  or stay purely structural (a branch + a ticket list)?

- **Nested epics**: should a ticket be allowed to belong to more than one epic,
  or should epics be flat? The current design assumes flat (one epic per
  ticket).

- **Cross-epic depends_on**: ticket in epic A depends on ticket in epic B (not
  yet closed to `main`). APM could resolve this by checking the epic-B branch
  instead of `main`. Deferring to phase 2 to keep the initial implementation
  simple.

- **Epic branch divergence**: if `main` advances while the epic is open, should
  APM prompt the supervisor to merge `main → epic/<slug>` periodically? Or only
  at epic-close time?
