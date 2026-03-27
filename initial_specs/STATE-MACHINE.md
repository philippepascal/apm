# APM — State Machine Schema and Reference

> Defines the schema for configuring the APM state machine, and provides the
> reference implementation for the ticker workflow.

---

## 1. Core concepts

The state machine in APM is defined entirely in `apm.toml`. It is not hardcoded.
Two teams can use APM with completely different workflows.

A state machine consists of:
- **States** — the discrete positions a ticket can be in
- **Transitions** — the edges between states, with rules for who can cross them and when

APM's built-in commands (`apm ask`, `apm reply`, `apm start`, `apm state`) are named triggers.
Whether they change state — and what state they change to — is determined by the state machine,
not by APM itself. If a team doesn't define a `question` state, `apm ask` still appends to the
ticket's Open questions section but does not change state. The command is decoupled from the
state machine.

---

## 2. State schema

Each state has the following properties.

### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `id` | string | yes | Machine identifier. Used in frontmatter, transitions, and CLI. No spaces. |
| `label` | string | yes | Human-readable display name. Shown on the board. |
| `description` | string | no | One-line explanation of what this state means and when a ticket is in it. |
| `color` | string | no | Hex color for the board column and card. |
| `terminal` | bool | no | If true, no transitions out. Tickets in terminal states are excluded from board by default. Defaults to false. |
| `actionable` | []string | no | Which actors can act on tickets in this state. Values: `"agent"`, `"supervisor"`, `"engineer"`, `"any"`. Used by `apm next` (filters for `"agent"`) and `apm list --actionable <actor>`. Defaults to empty (no actor filter). |

### TOML format

```toml
[[workflow.states]]
id          = "question"
label       = "Question"
description = "The agent has an open question for the supervisor. Work is paused until answered."
color       = "#f59e0b"
terminal    = false
actionable  = ["supervisor"]
```

---

## 3. Transition schema

Transitions are directed edges: from one state to another. They come in two varieties:

- **Manual transitions** — triggered explicitly by a person or agent running a command
- **Auto-transitions** — triggered by an external event (git push, PR opened, PR merged, review submitted)

### 3.1 Manual transitions

Manual transitions are declared as allowed outgoing transitions on a state, with an optional actor restriction.

#### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `to` | string | yes | Target state id |
| `trigger` | string | yes | What causes this transition. See §4 for values. |
| `actor` | string | yes | Who may enact this transition. See §5 for values. |
| `label` | string | no | Optional description of when to use this transition. |
| `preconditions` | []string | no | Conditions that must be true before the transition is allowed. See §6 for values. |
| `side_effects` | []string | no | Additional operations APM runs when this transition fires. See §7 for values. |

#### TOML format — transitions declared inside a state

```toml
[[workflow.states]]
id    = "specd"
label = "Specd"

  [[workflow.states.transitions]]
  to            = "ready"
  trigger       = "manual"
  actor         = "supervisor"
  label         = "Supervisor approves the spec"
  preconditions = ["spec_not_empty"]

  [[workflow.states.transitions]]
  to      = "ammend"
  trigger = "manual"
  actor   = "supervisor"
  label   = "Supervisor requests spec changes"
```

### 3.2 Auto-transitions

Auto-transitions fire when APM receives a specific event. Declared separately from states.

#### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `on` | string | yes | Event trigger. See §4 for values. |
| `from` | string | yes | Source state id |
| `to` | string | yes | Target state id |
| `enabled` | bool | no | Disable without deleting. Defaults to true. |
| `requires_provider` | bool | no | If true, this auto-transition only fires when a git provider is configured. Manual fallback needed in pure-git mode. Defaults to false. |
| `side_effects` | []string | no | Additional operations. See §7. |

#### TOML format

```toml
[[workflow.auto_transitions]]
on                = "event:branch_push_first"
from              = "ready"
to                = "in_progress"
requires_provider = false   # fires from local pre-push hook too

[[workflow.auto_transitions]]
on                = "event:pr_opened"
from              = "in_progress"
to                = "implemented"
requires_provider = true    # no equivalent in pure-git mode

[[workflow.auto_transitions]]
on                = "event:pr_all_merged"
from              = "implemented"
to                = "accepted"
requires_provider = true
```

---

## 4. Trigger types

| Trigger | Category | Description |
|---------|----------|-------------|
| `manual` | manual | Explicit `apm state N <state>` command |
| `command:ask` | manual | `apm ask N "..."` — agent adds a question to Open questions |
| `command:reply` | manual | `apm reply N "..."` — supervisor adds a reply to Open questions |
| `command:start` | manual | `apm start N` — begin implementation; renames ticket branch to feature branch |
| `command:take` | manual | `apm take N` — take over an in-progress ticket |
| `event:branch_push_first` | auto | First push of a branch matching `ticket/<id>-*` |
| `event:pr_opened` | auto | PR opened with branch or magic words linking to this ticket |
| `event:pr_all_merged` | auto | All `closes`-type PRs for this ticket are now merged |
| `event:pr_draft` | auto | PR converted to draft |
| `event:pr_review_approved` | auto | A reviewer approved the PR |
| `event:pr_review_changes` | auto | A reviewer requested changes |
| `event:pr_review_dismissed` | auto | A previously submitted review was dismissed |

### How named command triggers work

`apm ask` is not hardcoded to transition to a `question` state. It:

1. Appends the question text to the ticket's Open questions section (always), commits to ticket branch
2. Looks for a transition from the current state with `trigger = "command:ask"` in the state machine
3. If found: fires that transition
4. If not found: no state change; Open questions is updated only

The same logic applies to `command:reply`.

---

## 5. Actor types

Who may enact a manual transition.

| Actor | Description |
|-------|-------------|
| `engineer` | Any engineer (any human with repo access). |
| `supervisor` | Specifically the ticket's `supervisor` field value. APM checks `APM_AGENT_NAME == ticket.supervisor`. |
| `agent` | Any named agent process. Identified by `APM_AGENT_NAME`. |
| `system` | APM itself, acting on a received event. Used only in auto-transitions. |
| `any` | No restriction. Any actor may enact this transition. |

**Enforcement:** If a transition declares `actor = "supervisor"` and the current `APM_AGENT_NAME` does not match `ticket.supervisor`, APM prints an error and aborts. Use `--force` to override if you are an engineer taking ownership.

**No actor field:** If `actor` is omitted on a manual transition, it defaults to `any`.

---

## 6. Precondition types

Conditions APM checks before allowing a transition to fire. If any precondition fails, the transition is blocked with an error message.

| Precondition | Description |
|-------------|-------------|
| `spec_not_empty` | The ticket's `## Spec` section contains Problem, Acceptance criteria, Out of scope, and Approach subsections, each with non-whitespace content |
| `spec_has_acceptance_criteria` | The Spec section contains a `### Acceptance criteria` subsection with at least one checkbox (`- [ ]`) |
| `spec_all_criteria_checked` | All checkboxes in Acceptance criteria are checked (`- [x]`) |
| `spec_all_amendments_addressed` | No unchecked `- [ ]` boxes in `### Amendment requests` |
| `branch_exists` | `ticket.branch` is set and the branch exists in the remote |
| `pr_exists` | At least one PR is linked to this ticket |
| `pr_all_closing_merged` | All `closes`-type PRs are merged |
| `pr_approved` | At least one linked PR has `review = "approved"` |
| `no_open_questions` | Every `**Q**` line in Open questions has a following `**A**` line |

Custom preconditions are not supported in V1.

---

## 7. Side effects

Additional operations APM performs when a transition fires, beyond updating the `state` field. All commits go to the ticket's current branch (ticket branch or feature branch depending on the phase).

### Implicit side effects (always happen)

| When | Implicit side effect |
|------|----------------------|
| Any transition fires | Update `state` and `updated_at` in frontmatter; append row to `## History`; commit both to the ticket's current branch; update local cache |
| `apm new` runs | Allocate ID from `apm/meta`; create `ticket/<id>-<slug>` branch; commit initial ticket file; push branch |
| `command:start` trigger fires | Set `branch` and `agent` fields in frontmatter; commit to `ticket/<id>-<slug>` branch |
| `event:branch_push_first` trigger fires | Record `branch` field if not already set; commit frontmatter update to ticket branch |
| `event:pr_opened` trigger fires | Create or update `prs` record in frontmatter; commit to feature branch |
| `event:pr_all_merged` trigger fires | Mark `prs.state = merged`; commit state update to `main` (the one post-merge main commit) |
| `event:pr_review_*` trigger fires | Update `prs.review_state`; commit frontmatter update to feature branch |
| Transition enters a `terminal = true` state | Commit final state to `main`; if `archive_dir` is configured: move ticket file to archive on `main` |
| `apm ask` runs | Append question to `### Open questions`; commit to ticket's current branch |
| `apm reply` runs | Append reply to `### Open questions`; commit to ticket's current branch |

### Explicit side effects (declared on a transition)

| Side effect | Description |
|-------------|-------------|
| `set_agent_null` | Clear the `agent` field |
| `set_branch_null` | Clear the `branch` field |
| `delete_branch` | Delete the remote feature branch (use with caution; declared on terminal transitions only) |
| `notify_supervisor` | Send a notification to the ticket's supervisor |
| `notify_agent` | Send a notification to the ticket's current agent |

```toml
# Example: rolling back from in_progress to ready clears agent and branch
[[workflow.states]]
id = "in_progress"

  [[workflow.states.transitions]]
  to           = "ready"
  trigger      = "manual"
  actor        = "any"
  label        = "Roll back to ready (abandons branch)"
  side_effects = ["set_agent_null", "set_branch_null"]
```

---

## 8. Event delivery: polling, not a daemon

APM is not a daemon. It does not run persistently in the background. Auto-transitions fire through two mechanisms only:

### Local git events → synchronous hooks

The `pre-push` hook calls `apm _hook pre-push` as a one-shot process when the agent runs `git push`. APM detects the first push of a `ticket/<id>-*` branch in `ready` state, fires the transition, commits the state update to the ticket branch, and exits.

```
agent runs: git push
  → pre-push hook fires
  → apm _hook pre-push (detects first push of ticket/<id>-* in ready state)
  → fires ready → in_progress
  → commits state update to ticket branch
  → exits
git push completes
```

### Remote events → polling via `apm sync`

```
apm sync runs:
  → git fetch --all
  → for each ticket/* branch:
      read ticket file from branch
      check: is branch merged into main? (git branch --merged main)
      check: does a PR exist for this branch?
      check: what is the PR review state?
  → fire auto-transitions whose conditions are newly met
  → commit state updates to relevant branches
  → update local cache
```

`apm sync` runs:
- Required: at session start
- Automatically: on `post-merge` hook
- Optionally: via cron for background refresh

---

## 9. Merge detection and post-merge state

### How APM knows a branch was merged

```
git branch --merged main
```

If `ticket/42-add-csv-export` appears in that list, ticket #42's branch is merged. APM detects this during `apm sync` and fires `event:pr_all_merged`. **No webhook required. Branches are not deleted until the ticket is closed.**

### What the merge commit contains

When `ticket/42-*` merges into `main`, the merge commit contains the complete ticket file as it existed on the ticket branch: full spec, full history, all frontmatter. No additional reconciliation needed — the file's git history tells the complete story.

### The one post-merge APM commit to `main`

After detecting the merged branch, APM must update the `state` field to reflect the auto-transition. This is the single APM-originated commit that goes directly to `main`:

```
commit a3f9b12
Author: apm <apm@local>

    ticket(42): implemented → accepted [branch merged]
```

Only `state` and `updated_at` change. The ticket body is already correct from the merge commit.

### Branch discipline

After `apm start`, the feature branch is owned by the agent. Supervisors do not push to the feature branch directly. Post-implementation feedback is delivered through PR reviews. The `event:pr_review_changes` auto-transition moves the ticket back to `in_progress` so the agent knows to address review feedback on the branch.

---

## 10. `apm sync` and branch reads

All commands read ticket data directly from git branch blobs via `git show <branch>:<rel-path>`. There is no local filesystem cache and no SQLite index.

```
git show ticket/0018-apm-init-config:tickets/0018-apm-init-config.md
```

If the local branch ref is absent (e.g., a branch only seen on the remote), the fallback is `origin/<branch>`.

`apm sync` runs `git fetch --all` to update remote refs, then detects merged branches and fires auto-transitions. After `apm sync`, all commands see the latest state.

`apm state`, `apm set`, and similar write commands commit directly to the ticket's branch — using the permanent worktree if one exists, or a temporary worktree otherwise. No filesystem cache is maintained.

---

## 11. Reference implementation: ticker workflow

This is the canonical state machine for AI-assisted development using the ticker workflow. It is the default when `apm init` is run without customization.

### States

| id | label | description | terminal | actionable |
|----|-------|-------------|----------|------------|
| `new` | New | Ticket created. No spec yet. Waiting to be picked up or assigned. | false | `["agent"]` |
| `question` | Question | Agent has an open question for the supervisor. Work is paused. | false | `["supervisor"]` |
| `specd` | Specd | Agent has written a complete spec. Waiting for supervisor review. | false | `["supervisor"]` |
| `ammend` | Ammend | Supervisor has requested spec changes. Agent is revising. | false | `["agent"]` |
| `ready` | Ready | Spec approved. Waiting for agent to begin implementation. | false | `["agent"]` |
| `in_progress` | In Progress | Implementation underway. Agent has a feature branch. | false | `[]` |
| `blocked` | Blocked | Agent hit a blocker during implementation. Questions written in Open questions. Supervisor must unblock. | false | `["supervisor"]` |
| `implemented` | Implemented | PR is open. Waiting for review. | false | `["supervisor"]` |
| `accepted` | Accepted | PR merged. Waiting for supervisor to confirm and close. | false | `["supervisor"]` |
| `closed` | Closed | Done. | true | `[]` |

### Branch phases

| States | Branch | Who writes |
|--------|--------|-----------|
| `new` through `accepted` | `ticket/<id>-<slug>` | agent (spec + code), supervisor (amendments, question answers), APM (frontmatter, history) |
| `closed` | `main` only (via merge) | APM post-merge commit only |

### Transitions — full reference

---

#### `new → question`

| Field | Value |
|-------|-------|
| trigger | `command:ask` |
| actor | `agent` |
| preconditions | none |
| git side effects | `state` + `updated_at` + question text committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated; supervisor notified (if configured) |

---

#### `question → new`

| Field | Value |
|-------|-------|
| trigger | `command:reply` |
| actor | `any` |
| preconditions | none |
| git side effects | `state` + `updated_at` + reply text committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated; agent notified (if configured) |

---

#### `question → specd`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `agent` |
| preconditions | `spec_not_empty`, `spec_has_acceptance_criteria` |
| git side effects | `state` + `updated_at` committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated |
| notes | Agent moves to `specd` directly after question is answered, skipping `new`. |

---

#### `new → specd`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `agent` |
| preconditions | `spec_not_empty`, `spec_has_acceptance_criteria` |
| git side effects | `state` + `updated_at` committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated |

---

#### `specd → ready`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `supervisor` |
| preconditions | none |
| git side effects | `state` + `updated_at` committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated; agent notified (if configured) |
| notes | Only the ticket's `supervisor` can enact this. |

---

#### `specd → ammend`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `supervisor` |
| preconditions | none |
| git side effects | `state` + `updated_at` committed to `ticket/<id>-<slug>`; `### Amendment requests` section ensured present in body |
| apm side effects | Local cache updated; agent notified |
| notes | Supervisor writes amendment items to `### Amendment requests` before or after changing state. Both the amendment items and the state change commit to the ticket branch. |

---

#### `ammend → specd`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `agent` |
| preconditions | `spec_not_empty`, `spec_has_acceptance_criteria`, `spec_all_amendments_addressed` |
| git side effects | `state` + `updated_at` committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated |

---

#### `ammend → question`

| Field | Value |
|-------|-------|
| trigger | `command:ask` |
| actor | `agent` |
| preconditions | none |
| git side effects | `state` + `updated_at` + question text committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated; supervisor notified |

---

#### `ready → in_progress`

| Field | Value |
|-------|-------|
| trigger | `command:start` (primary) or `event:branch_push_first` (secondary) |
| actor | `agent` (for `command:start`); `system` (for event) |
| preconditions | none |
| git side effects | `branch`, `agent`, `state`, `updated_at` committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated; `agent` field set to `APM_AGENT_NAME` |
| notes | `apm start N` is the primary path. `event:branch_push_first` is a fallback for agents that push without `apm start`. |

---

#### `in_progress → implemented`

| Field | Value |
|-------|-------|
| trigger | `event:pr_opened` (primary) or `manual` (pure-git fallback) |
| actor | `system` (event); `agent` or `engineer` (manual) |
| preconditions | none (event); `pr_exists` (manual) |
| git side effects | `state` + `updated_at` + `prs` committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated; supervisor notified |

---

#### `in_progress → ready`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `any` |
| preconditions | none |
| git side effects | `state` + `updated_at` committed to `ticket/<id>-<slug>` |
| side_effects | `set_agent_null`, `set_branch_null` |
| notes | Rollback. Spec is preserved on the feature branch. Ticket returns to queue. Feature branch is NOT deleted automatically. |

---

#### `implemented → accepted`

| Field | Value |
|-------|-------|
| trigger | `event:pr_all_merged` (primary) or `manual` (pure-git fallback) |
| actor | `system` (event); `engineer` (manual) |
| preconditions | `pr_all_closing_merged` |
| git side effects | `state` + `updated_at` committed to `main` (the single post-merge APM commit) |
| apm side effects | Local cache updated |

---

#### `implemented → in_progress`

| Field | Value |
|-------|-------|
| trigger | `event:pr_review_changes` (primary) or `manual` (secondary) |
| actor | `system` (event); `engineer` (manual) |
| preconditions | none |
| git side effects | `state` + `updated_at` committed to `ticket/<id>-<slug>` |
| apm side effects | Local cache updated; agent notified |

---

#### `accepted → closed`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `supervisor` or `engineer` |
| preconditions | none |
| git side effects | `state` + `updated_at` committed to `main`; if `archive_dir` set: file moved to archive on `main` |
| apm side effects | Ticket marked terminal in cache; no longer shown on board by default |
| notes | Explicit human close. Intentional: `accepted` is a holding state; closing is a conscious acknowledgment that the work is done. |

---

### Diagram

```
              ┌─────────────────────────┐
              │                         ▼
new ──ask──▶ question ──reply──▶ new ──▶ specd
 │                                       │  │
 └───────────────────────────────────────┘  │ supervisor approves
                                            ▼
ammend ◀── specd ──supervisor approves──▶ ready
  │                                         │
  └───────────agent revises───────────▶ specd   apm start
                                            │
                                            ▼
                                      in_progress ◀── pr review changes
                                            │
                                      pr opened │
                                            ▼
                                      implemented
                                            │
                                     all PRs merged
                                            ▼
                                        accepted
                                            │
                                         manual
                                            ▼
                                         closed
```

---

## 12. Alternative: minimal/trusting workflow

For a team or solo engineer who wants maximum agent autonomy — no spec approval gate, no question state:

```toml
[workflow]
terminal_states = ["closed"]

[[workflow.states]]
id = "todo"    label = "To Do"

  [[workflow.states.transitions]]
  to = "doing"   trigger = "command:start"   actor = "any"

[[workflow.states]]
id = "doing"   label = "Doing"

  [[workflow.states.transitions]]
  to = "done"    trigger = "manual"           actor = "any"

[[workflow.states]]
id = "done"    label = "Done"   terminal = true

[[workflow.auto_transitions]]
on = "event:pr_all_merged"   from = "doing"   to = "done"
```

Three states. No supervisor gate. Agent goes from `todo` → `doing` → `done` without waiting for human approval. `apm ask` still works — it appends to Open questions — but does not change state.
