# APM — Full Ticket Lifecycle Scenario

Imaginary ticket: **"Add CSV export to billing report"**

Every state, every transition, and every `apm` command is accounted for.
Commands that do not appear in the normal lifecycle are noted at the bottom.

---

## Actors

| Actor | Meaning |
|-------|---------|
| **supervisor** | The human (or supervisor Claude session) that owns the backlog and reviews specs/PRs |
| **delegator** | Finds the next actionable ticket, provisions the worktree, and spawns the appropriate subagent. Will be either a master agent or `apm work` — TBD. State-agnostic: it reads instructions from the state config and dispatches; it does not do the work itself. |
| **current_agent** | The subagent spawned by the delegator and currently assigned to the ticket (`agent` field in frontmatter) |
| **any** | Supervisor or any agent — no restriction |

`git`, `gh`, and `apm` are tools, not actors.

---

## One-time repository setup

**supervisor** runs `apm init`
  → creates `apm.toml` with the full reference state machine
  → creates `apm.agents.md` with agent instructions template
  → creates `apm.worker.md` with worker subprocess instructions
  → installs `.git/hooks/pre-push` pointing to `apm _hook pre-push`

---

## Session startup (every session, any actor)

**any** runs `apm sync`
  → runs `git fetch --all` to update remote refs
  → reads every `ticket/*` branch and detects state changes since last sync
  → checks `git branch --merged main` for each ticket branch: if merged and
    ticket is `implemented`, prints "PR merged — run `apm state <id> accepted`"
    but does NOT auto-transition
  → finds tickets in `accepted` state and offers to batch-close them

**any** runs `apm agents` *(optional — agent reads its own instructions)*
  → prints contents of `apm.agents.md`

**any** runs `apm list` *(optional — browse the board)*
  → reads all tickets; prints id, state, title, agent for non-terminal tickets
  → `--state <s>` filters by state; `--all` includes closed

**any** runs `apm next`
  → scans all tickets for states where `actionable` includes `"agent"` and
    `agent` field is unset
  → returns the highest-priority unassigned actionable ticket
  → returns null if nothing is available

---

## Phase 1a — Ticket creation by supervisor

**supervisor** runs `apm new "Add CSV export to billing report"`
  → allocates the next ticket ID
  → creates branch `ticket/<id>-add-csv-export-to-billing-report`
  → writes initial ticket file in frontmatter: `state = "new"`, `author = APM_AGENT_NAME`
  → scaffolds the spec body sections from `apm.toml` `[ticket.sections]` config
    (each section has a `name`, `type`, and optional `placeholder`; see note below)
  → opens `$EDITOR` with the scaffolded file
     → **supervisor** fills in the sections
  → on save and exit, apm commits the file to the ticket branch and pushes it
  → prints the ticket ID

*Optional* **supervisor** runs `apm set <id> priority <non-default priority>`

---

## Phase 1b — Ticket creation by agent

**current_agent** runs `apm new --no-edit "Add CSV export to billing report" --context "The billing API returns all rows but the UI has no way to download them. Users are exporting manually via copy-paste. Discovered while implementing the reporting dashboard (#37)."`
  → allocates the next ticket ID
  → creates branch `ticket/<id>-add-csv-export-to-billing-report`
  → writes initial ticket file in frontmatter: `state = "new"`, `author = APM_AGENT_NAME`
  → scaffolds the spec body sections from `apm.toml` `[ticket.sections]` config
    (each section has a `name`, `type`, and optional `placeholder`; see note below)
  → reads `context_section` from the creation transition config (`"Problem"`)
  → inserts the `--context` value into that section
    *(differs from 1a: no editor is opened; `--context` replaces the supervisor's manual input)*
  → commits the file to the ticket branch and pushes it
  → prints the ticket ID

*Optional* **current_agent** runs `apm set <id> priority <non-default priority>`

---

## Phase 2 — Spec writing

**delegator** runs `apm start --next`  *(transition: `new → in_design`)*
  → runs `apm next` internally to find the highest-priority actionable ticket
  → provisions worktree if it doesn't already exist
  → verifies the transition with `trigger: command:start` from the current state
  → generates `APM_AGENT_NAME` for the subagent
  → sets `state = "in_design"`, `agent = APM_AGENT_NAME`, `updated_at = now()`
  → appends history row; commits and pushes
  → reads `instructions` file from the state config (`"apm.spec-writer.md"`)
  → reads ticket content via `apm show <id>`
  → spawns subagent: `claude --print --system-prompt <instructions> <ticket-content>`
    with `APM_AGENT_NAME` in environment, worktree as working directory
  → prints ticket ID, subagent PID, and log path; returns immediately

*(From here the subagent is current_agent. It follows the instructions in
`apm.spec-writer.md`. The delegator is done — it can immediately loop and
call `apm start --next` again for the next ticket.)*

---

**current_agent** runs `apm spec <id> --section "Problem" --content "..."`
  → reads section config: `type = "free"` — accepts prose content as-is
  → writes content into the `### Problem` section of the ticket file
  → commits to ticket branch

**current_agent** runs `apm spec <id> --section "Acceptance criteria" --content "..."`
  → reads section config: `type = "tasks"` — wraps each line as `- [ ] ...`
  → writes items into the `### Acceptance criteria` section
  → commits to ticket branch

**current_agent** runs `apm spec <id> --section "Out of scope" --content "..."`
  → reads section config: `type = "free"`
  → commits to ticket branch

**current_agent** runs `apm spec <id> --section "Approach" --content "..."`
  → reads section config: `type = "free"`
  → commits to ticket branch

*(The agent knows which sections to fill and in what format because the
delegator's initial message included the ticket state and the list of required
sections with their types — all from `[ticket.sections]` config. No file
editing or git commands needed from the agent.)*

---

### Detour A — Agent has questions before finishing the spec

**current_agent** runs `apm spec <id> --section "Open questions" --content "Should hidden columns be included in the export?"`
  → reads section config: `type = "qa"` — formats content as `**Q:** ...`
  → appends the question to `### Open questions`; commits to ticket branch

**current_agent** runs `apm spec <id> --section "Open questions" --content "Should the 50 MB limit apply per request or per user per day?"`
  → same — appends a second question; commits

*(The agent adds all questions before transitioning — one `apm spec` call per
question, each committed independently.)*

**current_agent** runs `apm state <id> question`  *(transition: `in_design → question`)*
  → sets `state = "question"`, `updated_at = now()`
  → appends history row; commits and pushes

**supervisor** runs `apm show <id>`
  → reads ticket; sees the open question

**supervisor** runs `apm review <id>`
  → opens `$EDITOR` with the ticket file
     → **supervisor** adds the answer below each question in `### Open questions`:
       `**A:** Include all columns including hidden ones — they are used in reconciliation.`
       `**A:** 50 MB limit applies per request.`
  → on save, apm:
    → commits the edited file to ticket branch
    → determines valid outgoing transitions for the supervisor from the current state
    → only one exists (`question → new`) — applies it automatically
    → sets `state = "new"`, `updated_at = now()`
    → appends history row; pushes

**delegator** runs `apm start --next`  *(transition: `new → in_design`)*
  → finds this ticket (now actionable again, agent unassigned)
  → worktree already exists — reuses it
  → generates a new `APM_AGENT_NAME` for the new subagent
  → sets `state = "in_design"`, `agent = APM_AGENT_NAME`, `updated_at = now()`
  → appends history row; commits and pushes
  → spawns new subagent with `apm.spec-writer.md` instructions and ticket content
  → previous subagent was done when it transitioned to `question`; this is a fresh one

*(The new subagent finds the answers already in `### Open questions` and
continues filling in the remaining spec sections.)*

---

**current_agent** runs `apm spec <id> --section "Problem" --content "The billing API returns all rows but the UI has no export mechanism. Users are copying data manually."`
  → reads section config: `type = "free"` — writes prose as-is
  → commits to ticket branch

**current_agent** runs `apm spec <id> --section "Acceptance criteria" --content "GET /billing/export?format=csv returns valid CSV\nAll columns including hidden ones are included\nFile size capped at 50 MB; larger exports return 413\nMalformed rows are skipped and counted in a response header"`
  → reads section config: `type = "tasks"` — wraps each line as `- [ ] ...`
  → commits to ticket branch

**current_agent** runs `apm spec <id> --section "Out of scope" --content "Excel format\nAsync export jobs"`
  → reads section config: `type = "free"`
  → commits to ticket branch

**current_agent** runs `apm spec <id> --section "Approach" --content "Add a GET /billing/export endpoint that streams rows from the billing service, serialises all columns to CSV including hidden ones, and enforces a 50 MB cap. Malformed rows are logged and skipped."`
  → reads section config: `type = "free"`
  → commits to ticket branch

**current_agent** runs `apm set <id> effort 3`
  → spec is complete; agent now has enough context to assess implementation scale
  → updates `effort = 3` in frontmatter; commits and pushes

**current_agent** runs `apm set <id> risk 2`
  → updates `risk = 2` in frontmatter; commits and pushes

**current_agent** runs `apm state <id> specd`  *(transition: `in_design → specd`)*
  → verifies transition allowed (actor: agent, trigger: manual)
  → checks preconditions derived from `[ticket.sections]` config: all sections
    with `required = true` have content; all `tasks` sections have at least one item
  → optionally: `effort` and `risk` are set (can be declared as preconditions)
  → sets `state = "specd"`, `updated_at = now()`
  → appends history row; commits and pushes

---

## Phase 3 — Spec review

**supervisor** runs `apm show <id>`
  → reads ticket; reviews Problem, Acceptance criteria, Approach

### Detour B — Supervisor requests changes

**supervisor** runs `apm review <id>`
  → opens `$EDITOR` with the ticket file
     → **supervisor** reads the spec and adds amendment items to `### Amendment requests`:
       `- [ ] Add error handling section for malformed CSV rows`
       `- [ ] Specify the 50 MB limit in the AC, not just the Approach`
  → on save, apm:
    → commits the edited file to ticket branch
    → two transitions are valid for supervisor from `specd`: `ready` and `ammend`
    → prompts: `Transition to: (1) ready  (2) ammend  > `
       → **supervisor** selects `ammend`
    → ensures `### Amendment requests` section exists (side effect)
    → sets `state = "ammend"`, `updated_at = now()`
    → appends history row; pushes

**delegator** runs `apm start --next`  *(transition: `ammend → in_design`)*
  → finds this ticket (state `ammend`, actionable to `agent`, unassigned)
  → worktree already exists — reuses it
  → generates a new `APM_AGENT_NAME` for the subagent
  → sets `state = "in_design"`, `agent = APM_AGENT_NAME`, `updated_at = now()`
  → appends history row; commits and pushes
  → reads `instructions` from state config (`"apm.spec-writer.md"`)
  → spawns subagent with instructions and ticket content (including amendment items)

**current_agent** runs `apm spec <id> --section "Approach" --content "...updated approach including error handling for malformed rows..."`
  → reads section config: `type = "free"` — overwrites existing content
  → commits to ticket branch

**current_agent** runs `apm spec <id> --section "Acceptance criteria" --content "...updated with 50 MB limit and malformed row criterion..."`
  → reads section config: `type = "tasks"` — merges new items with existing checked ones
  → commits to ticket branch

**current_agent** runs `apm spec <id> --section "Amendment requests" --check "Add error handling section for malformed CSV rows"`
  → reads section config: `type = "tasks"` — finds matching item, marks `- [x]`
  → commits to ticket branch

**current_agent** runs `apm spec <id> --section "Amendment requests" --check "Specify the 50 MB limit in the AC, not just the Approach"`
  → marks matching item `- [x]`; commits

**current_agent** runs `apm state <id> specd`  *(transition: `in_design → specd`)*
  → checks preconditions from `[ticket.sections]` config: all required sections have content
  → checks `spec_all_amendments_addressed`: no unchecked `- [ ]` in `### Amendment requests`
    — fails with error listing open items if not
  → sets `state = "specd"`, `updated_at = now()`
  → appends history row; commits and pushes

---

### Supervisor approves

**supervisor** runs `apm state <id> ready`  *(transition: `specd → ready`)*
  → verifies transition allowed (actor: supervisor, trigger: manual)
  → sets `state = "ready"`, clears `agent` field (side effect: `set_agent_null`)
  → appends history row
  → commits and pushes

---

## Phase 4 — Implementation

**delegator** runs `apm start --next`  *(transition: `ready → in_progress`)*
  → finds this ticket (state `ready`, actionable to `agent`, unassigned)
  → worktree already exists — reuses it
  → runs `git fetch origin` then merges `origin/main` into the ticket branch
    so agent starts from current code
  → generates a new `APM_AGENT_NAME` for the subagent
  → sets `state = "in_progress"`, `agent = APM_AGENT_NAME`, `updated_at = now()`
  → appends history row; commits and pushes
  → reads `instructions` from state config (`"apm.worker.md"`)
  → spawns subagent with instructions and ticket content
  → prints ticket ID, subagent PID, and log path; returns immediately

*(The current_agent follows the instructions in `apm.worker.md` for the
remainder of the implementation: writing code, running tests, committing. The
specific commands — how to build, how to test, what to commit — are
project-specific and come from those instructions, not from this document.)*

---

### Detour C — Agent is blocked mid-implementation

**current_agent** runs `apm spec <id> --section "Open questions" --content "The billing API does not expose column visibility flags. Add a new endpoint or hardcode the column list?"`
  → reads section config: `type = "qa"` — formats as `**Q:** ...`
  → appends to `### Open questions`; commits to ticket branch

**current_agent** runs `apm state <id> blocked`  *(transition: `in_progress → blocked`)*
  → verifies transition allowed (actor: agent, trigger: manual)
  → transitions, commits, pushes

**supervisor** runs `apm show <id>`
  → reads the open question

**supervisor** runs `apm review <id>`
  → opens `$EDITOR` with the ticket file
     → **supervisor** adds the answer in `### Open questions`:
       `**A:** Hardcode the column list for now. Follow-up ticket will add the endpoint.`
  → on save, apm:
    → commits the edited file to ticket branch
    → only one transition valid for supervisor from `blocked`: `ready`
    → applies it automatically
    → clears `agent` (side effect: `set_agent_null`)
    → sets `state = "ready"`, `updated_at = now()`
    → appends history row; pushes

**delegator** runs `apm start --next`  *(transition: `ready → in_progress`)*
  → finds this ticket (state `ready`, unassigned)
  → worktree already exists — reuses it; merges latest main
  → generates new `APM_AGENT_NAME`; sets `state = "in_progress"`, `agent`
  → appends history row; commits and pushes
  → spawns subagent with `apm.worker.md` instructions and ticket content

---

*(The current_agent follows `apm.worker.md` instructions for the remainder
of the implementation: writing code, running tests, committing. The specific
commands are project-specific and come from those instructions.)*

**current_agent** runs `apm state <id> implemented`  *(transition: `in_progress → implemented`)*
  → verifies transition allowed (actor: agent, trigger: manual)
  → reads `completion = "pr"` from transition config
  → pushes ticket branch to origin
  → creates PR: title from ticket frontmatter, body auto-generated from spec sections
  → sets `state = "implemented"`, `updated_at = now()`
  → appends history row; commits and pushes

---

## Phase 5 — Review

**supervisor** runs `apm show <id>`
  → reads spec and acceptance criteria to use as review checklist

**supervisor** reviews the PR on GitHub (outside APM)
  → reads the diff; verifies AC are met; leaves comments

### Detour D1 — Reviewer requests code changes

**supervisor** runs `apm review <id>`
  → opens `$EDITOR` with the ticket file
     → **supervisor** adds review feedback to `### Code review`:
       `- [ ] Stream rows instead of buffering the full result set`
       `- [ ] Add integration test for the 50 MB cap`
  → on save, apm:
    → commits the edited file to ticket branch
    → only one valid transition for supervisor from `implemented`: `ready`
    → applies it automatically
    → writes `focus_section = "Code review"` to frontmatter (transient side effect)
    → clears `agent` (side effect: `set_agent_null`)
    → sets `state = "ready"`, `updated_at = now()`
    → appends history row; pushes

**delegator** runs `apm start --next`  *(transition: `ready → in_progress`)*
  → finds this ticket (state `ready`, unassigned)
  → worktree already exists — reuses it; merges latest main
  → generates new `APM_AGENT_NAME`; sets `state = "in_progress"`, `agent`
  → reads `focus_section = "Code review"` from frontmatter; clears it
  → appends history row; commits and pushes
  → spawns subagent with `apm.worker.md` instructions, ticket content, and:
    `"Focus on the **Code review** section — the supervisor has left specific feedback there."`

*(current_agent follows `apm.worker.md` instructions to address the Code review items and commit)*

**current_agent** runs `apm spec <id> --section "Code review" --check "Stream rows instead of buffering the full result set"`
  → marks item `- [x]`; commits

**current_agent** runs `apm spec <id> --section "Code review" --check "Add integration test for the 50 MB cap"`
  → marks item `- [x]`; commits

**current_agent** runs `apm state <id> implemented`  *(transition: `in_progress → implemented`)*
  → reads `completion = "pr"` from transition config
  → pushes ticket branch to origin
  → PR already exists — skips creation; PR auto-updates from the new push
  → sets `state = "implemented"`, `updated_at = now()`
  → appends history row; commits and pushes

---

### Detour D2 — Reviewer requests spec changes

**supervisor** runs `apm review <id>`
  → opens `$EDITOR` with the ticket file
     → **supervisor** adds to `### Amendment requests`:
       `- [ ] The acceptance criteria do not cover the error response format for oversized exports`
  → on save, apm:
    → commits the edited file to ticket branch
    → two valid transitions for supervisor from `implemented`: `ready` and `ammend`
    → prompts: `Transition to: (1) ready  (2) ammend  > `
       → **supervisor** selects `ammend`
    → clears `agent` (side effect: `set_agent_null`)
    → sets `state = "ammend"`, `updated_at = now()`
    → appends history row; pushes

*(From here the flow is identical to Detour B — delegator spawns a spec-writer
agent via `ammend → in_design`, agent revises the spec using `apm spec`, checks
all amendment boxes, and runs `apm state <id> specd`. Supervisor reviews again
and transitions back to `ready` when satisfied. Then Phase 4 repeats.)*

---

**supervisor** merges the PR on GitHub:
  `gh pr merge <n> --squash`
  → ticket branch merged into `main`

---

## Phase 6 — Close out

**any** runs `apm sync`
  → detects that `ticket/<id>-*` now appears in `git branch --merged main`
  → ticket is in `implemented` state
  → if one ticket merged: prompts "PR merged for #N — accept it? [y/N]"
  → if multiple tickets merged: prompts "3 PRs merged — accept all? [y/N/pick]"
    → `y` — transitions all to `accepted` in one pass
    → `N` — skips; user can run `apm state <id> accepted` manually
    → `pick` — steps through each ticket individually
  → on confirmation, applies `implemented → accepted` for each:
    → checks precondition `pr_all_closing_merged` — skips any with open PRs
    → sets `state = "accepted"`, `updated_at = now()`
    → appends history row; commits and pushes
  → after accepting, detects newly accepted tickets and offers batch-close:
    "Accept complete — close all accepted tickets? [y/N]"

*(All transitions above can also be applied individually with `apm state <id> <state>` — the `apm sync` prompts are convenience shortcuts, not the only path.)*

**supervisor** runs `apm state <id> closed`  *(transition: `accepted → closed`)*
  → verifies transition allowed (actor: supervisor, trigger: manual)
  → sets `state = "closed"` (terminal state), `updated_at = now()`
  → appends history row; commits final state
  → ticket no longer shown in `apm list` by default

**supervisor** runs `apm clean` *(optional — reclaim local resources)*
  → finds all tickets in `closed` state
  → removes their worktrees
  → deletes their local branches
  → prints a summary: "Cleaned 3 tickets (worktrees + branches removed)"
  → remote branches are not deleted — they are the permanent record

---

## `apm.toml` transition changes required

*TODO: update `apm.toml` to match this design — remove the disallowed transitions
and add the missing ones listed below.*

### Transitions to add

These transitions are required by this design but not yet in `apm.toml`.

| Transition | Why needed |
|------------|------------|
| `implemented → ready` | Reviewer requests code changes (Detour D1); must carry `focus_section` side effect. |
| `implemented → ammend` | Reviewer requests spec changes (Detour D2). |

### Transitions to remove

These transitions currently exist in `apm.toml` but conflict with this design.

| Transition | Why disallowed |
|------------|----------------|
| `new → specd` | Every spec must be claimed via `in_design` first — visibility is not optional. |
| `new → question` | Agent must claim the ticket (`new → in_design`) before asking questions. |
| `question → specd` | After a question is answered the ticket returns to `new`; agent must go through `in_design` again. |
| `in_progress → ready` | Agent cannot silently abandon a ticket. Must transition to `blocked` to surface the reason. |

### Transitions that should already exist (verify)

These transitions are used in the scenario and should be present in `apm.toml`.

| Transition |
|------------|
| `new → in_design` |
| `ammend → in_design` |
| `in_design → specd` |
| `in_design → question` |
| `ammend → question` |
| `question → new` |
| `specd → ready` |
| `specd → ammend` |
| `ready → in_progress` |
| `in_progress → implemented` |
| `in_progress → blocked` |
| `blocked → ready` |
| `implemented → accepted` |
| `accepted → closed` |

---

## Not yet implemented — required by this design

### New commands

| Command | Purpose |
|---------|---------|
| `apm start --next` | Delegator primitive: find next actionable ticket, provision worktree, claim, spawn subagent |
| `apm start <id>` generalised | Currently only works from `ready`; needs to work from any state with `trigger: command:start` |
| `apm spec <id> --section <name> --content <text>` | Write or overwrite a spec section; validates format against section `type` (free/tasks/qa); commits |
| `apm spec <id> --section <name> --check <item>` | Mark a `tasks`-type section item as done (`- [ ]` → `- [x]`); commits |
| `apm new --context <text>` | Create ticket with inline context routed to the section named by `context_section` in the creation transition config |
| `apm review` redesigned | Opens `$EDITOR`; commits on save; auto-applies transition if only one valid option exists, prompts if multiple |
| `apm sync` interactive | After detecting merged PRs: prompts to accept individually, in bulk, or via pick list; chains into batch-close offer |
| `apm clean` | Remove worktrees and local branches for all `closed` tickets; remote branches are retained |

### New config properties

| Property | Where | Purpose |
|----------|-------|---------|
| `instructions` | `[[workflow.states]]` | Markdown file the delegator passes as the subagent system prompt when transitioning into this state |
| `context_section` | `[[workflow.states.transitions]]` | Section name that receives the `--context` value on ticket creation |
| `completion` | `[[workflow.states.transitions]]` | Side effect on `apm state`: `"pr"` (push + open/update PR), `"merge"` (push + squash-merge), `"none"` (default) |
| `focus_section` | `[[workflow.states.transitions]]` | Written as a transient frontmatter field on the ticket; `apm start --next` injects it as a focus hint into the subagent's initial context, then clears it |
| `[ticket.sections]` | top-level | Defines spec sections: `name`, `type` (free/tasks/qa), `required`, `placeholder` |

---

## APM commands not appearing in the normal lifecycle

| Command | Reason |
|---------|---------|
| `apm verify` | Maintenance/debugging tool. Run by an engineer when suspecting cache corruption or inconsistent ticket states. Not part of normal ticket flow. Accepts `--fix` to auto-repair. |
| `apm _hook <event>` | Internal. Called automatically by `.git/hooks/pre-push`. Never run by an actor directly. |

---

## Note — Ticket section configuration

Spec sections are not hardcoded. They are defined in `apm.toml` under
`[ticket.sections]`. Each section has:

| Field | Description |
|-------|-------------|
| `name` | Display heading (e.g. `"Problem"`) |
| `type` | `"free"` (prose), `"tasks"` (checkbox list), `"qa"` (question/answer pairs) |
| `required` | Whether the section must have content before certain transitions |
| `placeholder` | Optional hint text shown in the editor scaffold |

Transitions can declare a `context_section` property naming which section
receives the `--context` value passed to the command. This is what allows
`apm new --context "..."` to place text into the right section without
hardcoding "Problem" anywhere in the code.

Transitions can declare a `completion` property controlling what `apm state`
does in addition to the state change:

| Value | Behaviour |
|-------|-----------|
| `"pr"` | Push ticket branch; open PR (title from frontmatter, body from spec); if PR already exists, push only |
| `"merge"` | Push ticket branch; squash-merge into main directly |
| `"none"` | State change only — no git or GitHub action (default) |

This removes the need for the agent to call `git push` or `gh pr create`
directly. Project PR policy is encoded in `apm.toml`, not in agent instructions.

States can declare an `instructions` property pointing to a markdown file the
delegator passes as the subagent's system prompt:

```toml
[[workflow.states]]
id           = "in_design"
instructions = "apm.spec-writer.md"

[[workflow.states]]
id           = "in_progress"
instructions = "apm.worker.md"
```

The delegator reads `instructions` from the **target** state (the state the ticket
is being moved into) and spawns the appropriate subagent without knowing what
"spec writing" or "implementation" means — it only knows find → provision → spawn.

Example config (not yet implemented — this is the target):

```toml
[[ticket.sections]]
name        = "Problem"
type        = "free"
required    = true
placeholder = "What is broken or missing, and why does it matter?"

[[ticket.sections]]
name        = "Acceptance criteria"
type        = "tasks"
required    = true
placeholder = "Each item must be independently testable."

[[ticket.sections]]
name        = "Out of scope"
type        = "free"
required    = true
placeholder = "Explicit exclusions."

[[ticket.sections]]
name        = "Approach"
type        = "free"
required    = true
placeholder = "How the implementation will work."

[[ticket.sections]]
name        = "Open questions"
type        = "qa"
required    = false

[[ticket.sections]]
name        = "Amendment requests"
type        = "tasks"
required    = false

[[ticket.sections]]
name        = "Code review"
type        = "tasks"
required    = false
```

The preconditions `spec_not_empty` and `spec_has_acceptance_criteria` become
derivable from this config: "all sections with `required = true` have content"
and "all sections with `type = tasks` and `required = true` have at least one
item." The hardcoded section-name checks in Rust code go away.

---

## Open design questions

1. **`in_progress` has no `actionable`**: the state is `actionable = []`.
   Once a ticket is in `in_progress`, `apm next` will never return it, even if
   the agent crashes or disappears. Recovery requires `apm take <id>` (requires
   knowing the ticket ID) or `apm state <id> ready` followed by `apm start`.
   Should `in_progress` be `actionable = ["agent"]` with a "re-claim your own
   ticket" filter?

2. **`focus_section` storage**: the design stores `focus_section` as a transient
   frontmatter field on the ticket, written by one transition and cleared by
   `apm start --next`. An alternative is to have `apm start --next` look up
   which transition brought the ticket to its current state and read `focus_section`
   from that transition definition — no transient frontmatter needed. Which is
   cleaner?

3. **`apm sync` interactivity**: the design makes `apm sync` interactive when
   merged PRs are detected. Should this be opt-in (`apm sync --accept`)? Some
   actors (agents, CI) run `apm sync` for a silent refresh and should not be
   prompted.

4. **No `apm ask` / `apm reply` commands**: the `STATE-MACHINE.md` initial spec
   describes `command:ask` and `command:reply` triggers, but these commands were
   never implemented. The current design uses `apm spec --section "Open questions"`
   (agent) and `apm review` (supervisor). Is this the intended permanent design,
   or should dedicated `apm ask`/`apm reply` commands be built?
