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
ticket's Conversation section but does not change state. The command is decoupled from the
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
| `layer` | 1 or 2 | yes | Storage layer. `1` = frontmatter changes commit to `main`. `2` = ticket body changes commit to the feature branch. Set to `2` for all states at and after implementation begins. |
| `terminal` | bool | no | If true, no transitions out. Tickets in terminal states are excluded from board by default. Defaults to false. |

### TOML format

```toml
[[workflow.states]]
id          = "question"
label       = "Question"
description = "The agent has an open question for the supervisor. Work is paused until answered."
color       = "#f59e0b"
layer       = 1
terminal    = false
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
| `label` | string | no | Optional description of when to use this transition. Shown in `apm help state`. |
| `preconditions` | []string | no | Conditions that must be true before the transition is allowed. See §6 for values. |
| `side_effects` | []string | no | Additional operations APM runs when this transition fires. See §7 for values. |

#### TOML format — transitions declared inside a state

```toml
[[workflow.states]]
id    = "specd"
label = "Specd"
layer = 1

  [[workflow.states.transitions]]
  to           = "ready"
  trigger      = "manual"
  actor        = "supervisor"
  label        = "Supervisor approves the spec"
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
| `requires_provider` | bool | no | If true, this auto-transition only fires when a git provider (GitHub, GitLab, etc.) is configured. Manual fallback needed in pure-git mode. Defaults to false. |
| `side_effects` | []string | no | Additional operations. See §7. |

#### TOML format

```toml
[[workflow.auto_transitions]]
on               = "event:branch_push_first"
from             = "ready"
to               = "in_progress"
requires_provider = false   # fires from local pre-push hook too

[[workflow.auto_transitions]]
on               = "event:pr_opened"
from             = "in_progress"
to               = "implemented"
requires_provider = true    # no equivalent in pure-git mode

[[workflow.auto_transitions]]
on               = "event:pr_all_merged"
from             = "implemented"
to               = "accepted"
requires_provider = true
```

---

## 4. Trigger types

| Trigger | Category | Description |
|---------|----------|-------------|
| `manual` | manual | Explicit `apm state N <state>` command |
| `command:ask` | manual | `apm ask N "..."` — agent or engineer adds a question to Conversation |
| `command:reply` | manual | `apm reply N "..."` — supervisor or engineer adds a reply to Conversation |
| `command:start` | manual | `apm start N` — begin implementation; creates branch |
| `command:take` | manual | `apm take N` — take over an in-progress ticket |
| `event:branch_push_first` | auto | First push of a branch matching `feature/<id>-*` |
| `event:pr_opened` | auto | PR opened with branch or magic words linking to this ticket |
| `event:pr_all_merged` | auto | All `closes`-type PRs for this ticket are now merged |
| `event:pr_draft` | auto | PR converted to draft |
| `event:pr_review_approved` | auto | A reviewer approved the PR |
| `event:pr_review_changes` | auto | A reviewer requested changes |
| `event:pr_review_dismissed` | auto | A previously submitted review was dismissed |

### How named command triggers work

`apm ask` is not hardcoded to transition to a `question` state. It:

1. Appends the question text to the ticket's Conversation section (always)
2. Looks for a transition from the current state with `trigger = "command:ask"` in the state machine
3. If found: fires that transition
4. If not found: no state change; conversation is updated only

This means a state machine without a `question` state still supports `apm ask` as a conversation tool — it just won't change state. A highly trusting workflow (agent self-sufficient) simply doesn't configure a `command:ask` transition.

The same logic applies to `command:reply`: it appends to Conversation and fires any outgoing transition from the current state with `trigger = "command:reply"`, if one is defined.

---

## 5. Actor types

Who may enact a manual transition.

| Actor | Description |
|-------|-------------|
| `engineer` | Any engineer (any human with repo access). Identified by `APM_AGENT_NAME` being set to a non-agent identity, or inferred from git config. |
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
| `spec_not_empty` | The ticket's `## Spec` section contains at least one non-whitespace paragraph |
| `spec_has_acceptance_criteria` | The Spec section contains a `### Acceptance criteria` subsection with at least one checkbox (`- [ ]`) |
| `spec_all_criteria_checked` | All checkboxes in Acceptance criteria are checked (`- [x]`) |
| `branch_exists` | `ticket.branch` is set and the branch exists in the remote |
| `pr_exists` | At least one PR is linked to this ticket |
| `pr_all_closing_merged` | All `closes`-type PRs are merged |
| `pr_approved` | At least one linked PR has `review = "approved"` |
| `no_open_questions` | Conversation section has no unanswered question entries (heuristic: last entry is a reply, not a question) |

Custom preconditions are not supported in V1. The list above is the full set APM checks.

---

## 7. Side effects

Additional operations APM performs when a transition fires, beyond updating the `state` field.

Side effects are either **implicit** (always happen for a given trigger type) or **explicit** (declared in `side_effects` on a transition).

### Implicit side effects (always happen)

| When | Implicit side effect |
|------|----------------------|
| Any transition fires | Update `state` and `updated_at` in frontmatter; commit to `main`; update SQLite cache |
| Transition enters the `layer_boundary` state | Set `layer = 2`; subsequent body edits commit to branch |
| `command:start` trigger fires | Create branch `feature/<id>-<slug>`; push; set `branch` field in frontmatter; set `agent = APM_AGENT_NAME`; commit frontmatter to `main` |
| `event:branch_push_first` trigger fires | Record `branch` field if not already set; record push timestamp |
| `event:pr_opened` trigger fires | Create or update `ticket_prs` record |
| `event:pr_all_merged` trigger fires | Mark `ticket_prs.state = merged`; merge reconciliation (see §8) |
| `event:pr_review_*` trigger fires | Update `ticket_prs.review_state`; commit frontmatter to `main` |
| Transition enters a `terminal = true` state | If `archive_dir` is configured in `apm.toml`: move ticket file to archive dir; commit deletion + creation to `main` |

### Explicit side effects (declared on a transition)

These are optional operations that don't happen by default.

| Side effect | Description |
|-------------|-------------|
| `set_agent_null` | Clear the `agent` field (e.g., when moving back to a pre-implementation state) |
| `set_branch_null` | Clear the `branch` field (e.g., on rollback to `ready`) |
| `delete_branch` | Delete the remote feature branch (use with caution; declared on terminal transitions only) |
| `notify_supervisor` | Send a notification to the ticket's supervisor (implementation depends on provider and config) |
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

APM is not a daemon. It does not run persistently in the background waiting for events. Auto-transitions fire through two mechanisms only:

### Local git events → synchronous hooks

The `pre-push` hook calls `apm _hook pre-push` as a one-shot process when the agent runs `git push`. APM fires the transition, commits the frontmatter update to `main`, and exits. Total runtime: under a second.

```
agent runs: git push
  → pre-push hook fires
  → apm _hook pre-push (detects first push of feature/<id>-*)
  → fires ready → in_progress
  → commits frontmatter to main
  → exits
git push completes
```

### Remote events (PR opened, merged, review) → polling via `apm sync`

APM never needs a webhook to be delivered. Instead, `apm sync` queries git and the GitHub API, compares against the current SQLite state, and fires any auto-transitions whose conditions are now met.

```
apm sync runs:
  → git pull
  → for each open ticket with a branch:
      check: is the branch merged into main? (git branch --merged main)
      check: does a PR exist for this branch? (GitHub API or local git)
      check: what is the PR review state? (GitHub API)
  → fire any auto-transitions whose conditions are newly true
  → commit frontmatter updates to main for each fired transition
  → update SQLite cache
```

`apm sync` runs:
- Automatically: on `post-merge` and `post-checkout` hooks
- Manually: `apm sync` at session start (required in `apm.agents.md`)
- Optionally: via cron (`0 * * * * apm sync --all`) for background refresh

### Real-time path (optional)

If `apm serve` is running and `webhook_secret_env` is configured, GitHub webhook events are processed immediately instead of waiting for the next sync. This is strictly an enhancement — correctness does not depend on it. If the webhook fires and `apm serve` is not running, the next `apm sync` catches up.

### Latency table

| What's running | `branch_push` latency | `pr_opened` latency | `pr_merged` latency |
|---|---|---|---|
| hooks + `apm serve` + webhooks configured | instant (hook) | seconds (webhook) | seconds (webhook) |
| hooks + manual `apm sync` | instant (hook) | next sync | next sync |
| pure-git (no provider) | instant (hook) | manual `apm state` | manual `apm state` |

State is never permanently wrong — only temporarily behind the last sync.

---

## 9. Merge detection and the ticket file after PR merge

### How APM knows a branch was merged

APM never needs to be told that a PR merged. It asks git:

```
git branch --merged main
```

If `feature/42-add-csv-export` appears in that list, ticket #42's branch is merged. APM detects this during `apm sync` and fires `event:pr_all_merged` for any ticket whose branch is now in the merged set. **No webhook required. No daemon required. Branches are not deleted.**

This is why **branches must not be deleted** until the ticket is closed. The merged-branch list is APM's durable record that implementation was completed. Once a branch is deleted from the remote, this signal is gone. `apm state N closed` is the appropriate time to clean up the branch.

### What the merge commit contains

When a PR merges (`feature/42-*` → `main`), git performs a three-way merge of every changed file, including the ticket file. With the branch discipline rule in place:

| Section | Changed on `main` since branch | Changed on branch since branch |
|---------|-------------------------------|-------------------------------|
| Frontmatter | Yes (state, prs, review, updated_at) | No (frontmatter is never edited on branch) |
| `## Spec` | No | Yes (agent's work) |
| `## Conversation` | Yes (replies from supervisor) | No (all conversation commits to main) |
| `## History` | Yes (state transition rows) | No |

These changes are non-overlapping in the file. Git's three-way merge succeeds cleanly with no conflicts. The merge commit on `main` contains:
- The updated frontmatter (from main's side)
- The updated spec (from the branch's side)
- The full conversation (from main's side)
- The full history (from main's side)

**No additional reconciliation commit is needed.** The merge commit is the reconciliation.

### The one post-merge commit APM makes

After detecting a merged branch via `apm sync`, APM fires the `event:pr_all_merged` auto-transition and must update the `state` field in the frontmatter. This is a single commit:

```
commit a3f9b12
Author: apm <apm@local>

    ticket(42): state implemented → accepted [branch merged]
```

Only two lines in the frontmatter change: `state` and `updated_at`. This is not a reconciliation — it is a routine state transition commit, identical to any other state change. The body content is already correct from the merge commit itself.

### Branch discipline rule

For the merge commit to be conflict-free, APM enforces strict section ownership:

| Section | Committed to | Never committed to |
|---------|-------------|-------------------|
| Frontmatter | `main` only | feature branch |
| `## Spec` | feature branch only | `main` |
| `## Conversation` | `main` only | feature branch |
| `## History` | `main` only | feature branch |

**`apm ask` and `apm reply` always commit to `main`**, even when the agent is on a feature branch. The agent pulls the reply with `apm sync`, which fetches main without switching branches.

**`apm spec` is the only command that commits a body change to the feature branch.**

If a supervisor needs to edit the spec (e.g., to correct an acceptance criterion), they do so on `main` via `apm spec N` with `APM_REPO` set. The agent pulls the change on next sync. This avoids branch-side spec edits that would conflict.

---

## 11. Layer boundary and body storage

The `layer_boundary` setting in `apm.toml` names the state that begins Layer 2.

```toml
[tickets]
layer_boundary = "in_progress"
```

Any state whose position in the `workflow.states` array is at or after `layer_boundary` is a Layer 2 state. APM determines this by position, not by the `layer` field on the state (the `layer` field is display metadata; APM computes the actual boundary from the config).

When a ticket transitions into the `layer_boundary` state:
- `apm start` creates the feature branch
- Subsequent `apm spec` commits go to the branch
- Frontmatter commits, conversation, and history continue to go to `main`

When a ticket exits all Layer 2 states (moves to a terminal state):
- The feature branch is no longer modified by APM
- The branch should be kept (not deleted) until the ticket is closed — APM uses `git branch --merged main` to detect merged branches

---

## 12. Reference implementation: ticker workflow

This is the canonical state machine for AI-assisted development using the ticker workflow. It is the default when `apm init` is run without customization.

### States

| id | label | description | layer | terminal |
|----|-------|-------------|-------|----------|
| `new` | New | Ticket created. No spec yet. Waiting to be picked up or assigned. | 1 | false |
| `question` | Question | Agent has an open question for the supervisor. Work is paused. | 1 | false |
| `specd` | Specd | Agent has written a complete spec. Waiting for supervisor review. | 1 | false |
| `ammend` | Ammend | Supervisor has requested spec changes. Agent is revising. | 1 | false |
| `ready` | Ready | Spec approved. Waiting for agent to begin implementation. | 1 | false |
| `in_progress` | In Progress | Implementation underway. Agent has a branch. | 2 | false |
| `implemented` | Implemented | PR is open. Waiting for review. | 2 | false |
| `accepted` | Accepted | PR merged. Waiting for supervisor to confirm and close. | 2 | false |
| `closed` | Closed | Done. | 1 | true |

### Transitions — full reference

Each row describes one edge in the state machine: what causes it, who enacts it, and what APM does.

---

#### `new → question`

| Field | Value |
|-------|-------|
| trigger | `command:ask` |
| actor | `agent` |
| preconditions | none |
| git side effects | `updated_at` + `state` committed to `main` |
| github side effects | none |
| apm side effects | Conversation entry appended to `main`; cache updated; supervisor notified (if notify configured) |
| notes | Fires when agent calls `apm ask N "..."`. If state machine has no `command:ask` transition from `new`, the question is recorded but state stays `new`. |

---

#### `question → new`

| Field | Value |
|-------|-------|
| trigger | `command:reply` |
| actor | `any` |
| preconditions | none |
| git side effects | `updated_at` + `state` committed to `main` |
| github side effects | none |
| apm side effects | Conversation reply appended to `main`; cache updated; agent notified (if notify configured) |
| notes | Auto-exit from `question` when supervisor replies. The agent's next `apm sync` shows the reply and the new state. Alternatively, supervisor can manually set state to any valid target. |

---

#### `question → specd`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `agent` |
| preconditions | `spec_not_empty`, `spec_has_acceptance_criteria` |
| git side effects | `updated_at` + `state` committed to `main` |
| github side effects | none |
| apm side effects | cache updated |
| notes | Agent moves to `specd` directly after the question is answered, without going back through `new`. Valid shortcut. |

---

#### `new → specd`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `agent` |
| preconditions | `spec_not_empty`, `spec_has_acceptance_criteria` |
| git side effects | `updated_at` + `state` committed to `main` |
| github side effects | none |
| apm side effects | cache updated |
| notes | Agent writes spec without needing to ask questions (confident enough or ticket is well-described). |

---

#### `specd → ready`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `supervisor` |
| preconditions | none |
| git side effects | `updated_at` + `state` committed to `main` |
| github side effects | none |
| apm side effects | cache updated; agent notified (if notify configured) |
| notes | Supervisor approval. Only the ticket's `supervisor` can enact this. An agent or other engineer cannot self-approve a spec. |

---

#### `specd → ammend`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `supervisor` |
| preconditions | none |
| git side effects | `updated_at` + `state` committed to `main` |
| github side effects | none |
| apm side effects | cache updated; agent notified |
| notes | Supervisor requests spec revision. The supervisor should leave a comment or edit the spec inline before changing state. |

---

#### `ammend → specd`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `agent` |
| preconditions | `spec_not_empty`, `spec_has_acceptance_criteria` |
| git side effects | `updated_at` + `state` committed to `main` |
| github side effects | none |
| apm side effects | cache updated |
| notes | Agent has revised the spec and is resubmitting for review. |

---

#### `ammend → question`

| Field | Value |
|-------|-------|
| trigger | `command:ask` |
| actor | `agent` |
| preconditions | none |
| git side effects | `updated_at` + `state` committed to `main` |
| github side effects | none |
| apm side effects | Conversation entry appended; cache updated; supervisor notified |
| notes | Agent needs clarification before revising. Uncommon but valid. |

---

#### `ready → in_progress`

| Field | Value |
|-------|-------|
| trigger | `command:start` (primary) or `event:branch_push_first` (secondary) |
| actor | `agent` (for `command:start`); `system` (for event) |
| preconditions | none |
| git side effects | Branch `feature/<id>-<slug>` created and pushed; `branch`, `agent`, `updated_at`, `state` committed to `main` |
| github side effects | none at trigger time |
| apm side effects | Layer 2 begins; cache updated; `agent` field set to `APM_AGENT_NAME` |
| notes | `apm start N` is the primary path. The `event:branch_push_first` auto-transition is a fallback — if an agent manually creates and pushes the branch without using `apm start`, APM catches it. |

---

#### `in_progress → implemented`

| Field | Value |
|-------|-------|
| trigger | `event:pr_opened` (primary) or `manual` (pure-git fallback) |
| actor | `system` (event); `agent` or `engineer` (manual) |
| preconditions | none (event-triggered); `pr_exists` (manual) |
| git side effects | `updated_at` + `state` + `prs` array committed to `main` |
| github side effects | `ticket_prs` record created |
| apm side effects | cache updated; supervisor notified |
| notes | In pure-git mode (no provider): `apm state N implemented` after manually opening a PR, then `apm link-pr N <number>` to record it. |

---

#### `in_progress → ready`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `any` |
| preconditions | none |
| git side effects | `updated_at` + `state` committed to `main` |
| github side effects | none |
| apm side effects | cache updated |
| side_effects | `set_agent_null`, `set_branch_null` |
| notes | Rollback. Implementation abandoned. Spec is preserved. Ticket returns to the queue. The feature branch is NOT deleted automatically — use `--delete-branch` flag or delete manually. |

---

#### `implemented → accepted`

| Field | Value |
|-------|-------|
| trigger | `event:pr_all_merged` (primary) or `manual` (pure-git fallback) |
| actor | `system` (event); `engineer` (manual) |
| preconditions | `pr_all_closing_merged` (checked by APM from `ticket_prs` records) |
| git side effects | `updated_at` + `state` + `prs` (state=merged) committed to `main`; merge reconciliation commit |
| github side effects | none |
| apm side effects | cache updated; supervisor sees card in ACCEPTED |
| notes | The `event:pr_all_merged` trigger only fires when ALL `closes`-type PRs are merged. A `refs`-type PR merging does not trigger this. |

---

#### `implemented → in_progress`

| Field | Value |
|-------|-------|
| trigger | `event:pr_review_changes` (primary) or `manual` (secondary) |
| actor | `system` (event); `engineer` (manual) |
| preconditions | none |
| git side effects | `updated_at` + `state` + `prs` (review_state updated) committed to `main` |
| github side effects | none |
| apm side effects | cache updated; agent notified |
| notes | PR has changes requested. Ticket returns to in_progress so the agent knows to address review feedback. The branch still exists; no new branch needed. |

---

#### `accepted → closed`

| Field | Value |
|-------|-------|
| trigger | `manual` |
| actor | `supervisor` or `engineer` |
| preconditions | none |
| git side effects | `updated_at` + `state` committed to `main`; if `archive_dir` set: file moved to archive |
| github side effects | none |
| apm side effects | ticket marked terminal in cache; no longer shown on board by default |
| notes | Explicit human close. This is intentional — `accepted` is a holding state that confirms the merge happened; closing is a conscious acknowledgment that the work is done and any follow-up tickets have been created. |

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
  └───────────agent revises───────────▶ specd   apm start / branch push
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

### `apm.toml` — complete ticker workflow definition

```toml
[project]
name = "ticker"
description = ""

[tickets]
dir            = "tickets"
archive_dir    = "tickets/archive"
layer_boundary = "in_progress"

[workflow]
terminal_states = ["closed"]

[[workflow.states]]
id          = "new"
label       = "New"
description = "Ticket created. No spec yet."
color       = "#6b7280"
layer       = 1

  [[workflow.states.transitions]]
  to      = "specd"
  trigger = "manual"
  actor   = "agent"
  label   = "Agent submits spec"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

  [[workflow.states.transitions]]
  to      = "question"
  trigger = "command:ask"
  actor   = "agent"
  label   = "Agent asks a question"

[[workflow.states]]
id          = "question"
label       = "Question"
description = "Agent has an open question. Work is paused."
color       = "#f59e0b"
layer       = 1

  [[workflow.states.transitions]]
  to      = "new"
  trigger = "command:reply"
  actor   = "any"
  label   = "Supervisor replies; agent resumes"

  [[workflow.states.transitions]]
  to      = "specd"
  trigger = "manual"
  actor   = "agent"
  label   = "Agent moves to specd after getting answer"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

[[workflow.states]]
id          = "specd"
label       = "Specd"
description = "Agent submitted a spec. Waiting for supervisor review."
color       = "#3b82f6"
layer       = 1

  [[workflow.states.transitions]]
  to      = "ready"
  trigger = "manual"
  actor   = "supervisor"
  label   = "Supervisor approves spec"

  [[workflow.states.transitions]]
  to      = "ammend"
  trigger = "manual"
  actor   = "supervisor"
  label   = "Supervisor requests changes"

[[workflow.states]]
id          = "ammend"
label       = "Ammend"
description = "Spec needs revision. Agent is updating."
color       = "#ef4444"
layer       = 1

  [[workflow.states.transitions]]
  to      = "specd"
  trigger = "manual"
  actor   = "agent"
  label   = "Agent resubmits revised spec"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

  [[workflow.states.transitions]]
  to      = "question"
  trigger = "command:ask"
  actor   = "agent"
  label   = "Agent needs clarification before revising"

[[workflow.states]]
id          = "ready"
label       = "Ready"
description = "Spec approved. Waiting for implementation to begin."
color       = "#10b981"
layer       = 1

  [[workflow.states.transitions]]
  to           = "in_progress"
  trigger      = "command:start"
  actor        = "agent"
  label        = "Agent begins implementation"

[[workflow.states]]
id          = "in_progress"
label       = "In Progress"
description = "Implementation underway. Agent has a branch."
color       = "#8b5cf6"
layer       = 2

  [[workflow.states.transitions]]
  to      = "implemented"
  trigger = "manual"
  actor   = "agent"
  label   = "Agent marks as implemented (pure-git fallback)"
  preconditions = ["pr_exists"]

  [[workflow.states.transitions]]
  to           = "ready"
  trigger      = "manual"
  actor        = "any"
  label        = "Roll back to ready"
  side_effects = ["set_agent_null", "set_branch_null"]

[[workflow.states]]
id          = "implemented"
label       = "Implemented"
description = "PR is open. Waiting for review and merge."
color       = "#06b6d4"
layer       = 2

  [[workflow.states.transitions]]
  to      = "accepted"
  trigger = "manual"
  actor   = "engineer"
  label   = "Manual accept (pure-git fallback)"
  preconditions = ["pr_all_closing_merged"]

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "manual"
  actor   = "any"
  label   = "Return to in_progress (e.g., PR closed without merge)"

[[workflow.states]]
id          = "accepted"
label       = "Accepted"
description = "PR merged. Confirming work is done."
color       = "#84cc16"
layer       = 2

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"
  label   = "Supervisor closes the ticket"

[[workflow.states]]
id       = "closed"
label    = "Closed"
color    = "#374151"
layer    = 1
terminal = true

# Auto-transitions (event-driven, fired by git provider or local hooks)

[[workflow.auto_transitions]]
on               = "event:branch_push_first"
from             = "ready"
to               = "in_progress"
requires_provider = false

[[workflow.auto_transitions]]
on               = "event:pr_opened"
from             = "in_progress"
to               = "implemented"
requires_provider = true

[[workflow.auto_transitions]]
on               = "event:pr_all_merged"
from             = "implemented"
to               = "accepted"
requires_provider = true

[[workflow.auto_transitions]]
on               = "event:pr_review_changes"
from             = "implemented"
to               = "in_progress"
requires_provider = true
```

---

## 13. Alternative: minimal/trusting workflow

For a team or solo engineer who wants maximum agent autonomy — no spec approval gate, no question state — the state machine can be much simpler:

```toml
[workflow]
terminal_states = ["closed"]

[[workflow.states]]
id = "todo"    label = "To Do"      layer = 1

  [[workflow.states.transitions]]
  to = "doing"   trigger = "command:start"   actor = "any"

[[workflow.states]]
id = "doing"   label = "Doing"      layer = 2

  [[workflow.states.transitions]]
  to = "done"    trigger = "manual"           actor = "any"

[[workflow.states]]
id = "done"    label = "Done"       layer = 1   terminal = true

[[workflow.auto_transitions]]
on = "event:pr_all_merged"   from = "doing"   to = "done"
```

Three states. No supervisor gate. Agent goes from `todo` to `doing` to `done` without waiting for human approval at any step. `apm ask` still works — it appends to Conversation — but it does not change state. The agent answers its own questions and keeps going.

This shows the state machine is a first-class design decision, not a fixed workflow embedded in APM.
