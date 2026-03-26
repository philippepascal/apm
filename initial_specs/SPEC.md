# APM — Specification V2

> **Status:** Draft · **Date:** 2026-03-25
> Git-native, agent-first, Linear-grade UX. Rust implementation.

---

## 1. Design Principles

1. **Git is the database.** No external database. Tickets are files in the repo. Git history is the audit trail. The entire system can be reconstructed from a git clone.
2. **Fast by default.** The board loads from a user-global SQLite cache. Non-closed tickets load instantly. Every action feels immediate regardless of network.
3. **Agents do the work; engineers supervise.** Tickets are worked by named agents. Engineers watch the board, answer questions, approve specs, and review PRs. Any engineer can pick up any ticket — handoff is a one-command operation, not a process.
4. **Handoff is zero-friction.** The ticket document contains everything needed to continue: open questions, resolved decisions, the evolving spec, full history. Checkout the branch, read the ticket, keep going.
5. **CLI is complete.** Every operation available in the web client is available in the CLI. The web client is a local convenience, not a server or a requirement.
6. **Linear-grade feel.** Instant feedback, keyboard-first, dense but uncluttered, no unnecessary navigation.
7. **Configurable workflow.** The state machine, repos, and agent behavior are defined in `apm.toml` and `apm.agents.md`. APM ships with defaults suited to AI-assisted development; teams adapt them.
8. **Not a daemon.** APM is a CLI that runs on demand. State is derived from git on each `apm sync`. Local git hooks handle push events synchronously. Provider events are polled, not pushed. Correctness never depends on a background process.
9. **Integrity is verifiable.** A dedicated tool checks consistency of ticket files, the local cache, and the git history. Runs as a pre-commit hook.
10. **Provider-aware, not provider-locked.** APM is designed to be used with GitHub. It also works with GitLab, Gitea, Forgejo, and other providers through a provider abstraction layer. Pure-git mode (no provider) is a valid configuration for air-gapped or offline environments.
11. **Self-contained binary.** A single Rust binary provides the CLI, the local web server, and the embedded web client. No runtime dependencies beyond git.

---

## 2. Engineer / Agent Model

### Roles

**Engineers** supervise the work. Each ticket is assigned to one engineer — its **supervisor** — who is responsible for answering questions, approving specs, reviewing PRs, and deciding priority. An engineer sees their slice of the board: tickets they supervise. Any engineer can take over supervision of another's tickets; supervision is not exclusive.

**Agents** do the work. Each agent is a named process (`APM_AGENT_NAME`) that operates autonomously on one ticket at a time via the CLI. An agent writes the spec, asks clarifying questions, implements, and opens the PR. Its identity is recorded in the ticket's Conversation and History sections.

### Ticket ownership fields

| Field | Set by | Meaning |
|-------|--------|---------|
| `author` | APM on creation | Who created the ticket; never changes |
| `supervisor` | Set on creation or reassigned | The engineer responsible for this ticket; receives questions and approves specs |
| `agent` | APM on `apm start` or `apm take` | Who is currently doing the work; changes on handoff |

`supervisor` defaults to the ticket creator if an engineer creates it, or to unassigned if an agent creates it (the team should assign it promptly). `agent` is null until `in_progress` begins.

### The board: personal slice by default

The board defaults to showing tickets the current engineer supervises. A toggle switches to the full team view. State carries the action signal for supervisors:

| State | What the supervisor needs to do |
|-------|--------------------------------|
| `question` | Answer the agent's question (`apm reply`) |
| `specd` | Approve (`apm state N ready`) or request revision (`apm state N ammend`) |
| `ammend` | Nothing — agent is revising |
| `implemented` | Review the PR |
| `accepted` | Merge the PR |

States `new`, `ready`, `in_progress`, `closed` require no supervisor action unless stale.

### Handoff

**Supervision handoff** (reassigning the engineer):
```bash
apm supervise 42            # claim supervision: supervisor = APM_AGENT_NAME
apm set 42 supervisor alice # assign to a specific engineer
```

**Agent handoff** (another agent takes over implementation):
```bash
apm take 42                 # checkout branch, set agent = APM_AGENT_NAME
```

Both can happen independently. A ticket can change supervisor without changing agent, and vice versa. Branch history and conversation are always preserved.

---

## 3. Git Provider Integration

### Positioning

APM is designed to be used with **GitHub**. The full experience — automatic state transitions on push and PR events, PR review state on the board, remote web client access — requires a connected git provider. GitHub is the primary supported provider and the recommended setup.

APM is also **provider-aware**. The integration layer is abstracted so that GitLab, Gitea, Forgejo, and other self-hosted providers can be used in place of GitHub with equivalent functionality where their APIs allow it. For teams in air-gapped environments or with no remote provider, APM runs in pure-git mode using local git hooks for the subset of automation that doesn't require a remote API.

### What the provider enables

| Capability | With provider | Without provider |
|------------|--------------|-----------------|
| Board board load, ticket CRUD | ✓ (local git) | ✓ (local git) |
| `ready → in_progress` auto-transition | ✓ via push webhook or local hook | ✓ via local `pre-push` hook |
| `in_progress → implemented` auto-transition | ✓ via `apm sync` polling or webhook | Manual: `apm state 42 implemented` |
| `implemented → accepted` auto-transition | ✓ via `apm sync` (merged branch detection) | ✓ via `apm sync` (same — reads local git) |
| PR review state on board | ✓ | Not available |
| Activity indicator (recent branch commits) | ✓ via push event / polling | ✓ via local git |
| PR tracking in ticket frontmatter | ✓ automatic | Manual: `apm link-pr` |
| Remote web client (no local clone) | ✓ (provider API) | Not available |

### Provider abstraction

Internally, APM defines a `GitProvider` trait:

```
GitProvider
  open_pr(branch, title, body) → PR reference
  get_pr(pr_ref) → { state, review_state, merged_at }
  list_branches() → [branch_name]
  get_branch_last_commit(branch) → { sha, timestamp }
  receive_push_event(payload) → PushEvent
  receive_pr_event(payload) → PrEvent
```

Each provider implements this trait. The rest of APM is provider-agnostic. Adding a new provider is a matter of implementing the trait — no changes to the state machine, ticket format, or CLI.

### Provider configuration

The `[provider]` section in `apm.toml` is optional. Omitting it runs APM in pure-git mode.

**GitHub (recommended):**
```toml
[provider]
type = "github"
token_env = "APM_GITHUB_TOKEN"         # env var holding the PAT
poll_interval_secs = 60                # fallback polling when webhooks aren't configured
webhook_secret_env = "APM_WEBHOOK_SECRET"  # enables webhook receiver in `apm serve`
```

**GitLab:**
```toml
[provider]
type = "gitlab"
host = "https://gitlab.example.com"   # self-hosted or gitlab.com
token_env = "APM_GITLAB_TOKEN"
webhook_secret_env = "APM_WEBHOOK_SECRET"
```

**Gitea / Forgejo:**
```toml
[provider]
type = "gitea"                         # also works for forgejo
host = "https://gitea.example.com"
token_env = "APM_GITEA_TOKEN"
webhook_secret_env = "APM_WEBHOOK_SECRET"
```

**Pure-git (no provider):**
```toml
# Omit [provider] entirely, or:
[provider]
type = "none"
```

### Provider capability matrix

| Feature | GitHub | GitLab | Gitea/Forgejo | None |
|---------|--------|--------|---------------|------|
| Push events → auto-transition | ✓ webhook | ✓ webhook | ✓ webhook | local hook only |
| PR/MR open event | ✓ | ✓ (MR) | ✓ | — |
| PR/MR merge event | ✓ | ✓ | ✓ | — |
| Review state (approved/changes) | ✓ | ✓ | ✓ | — |
| Remote web client | ✓ Contents API | ✓ Files API | ✓ Files API | — |
| Activity polling | ✓ | ✓ | ✓ | local git |
| Auto PR link detection | ✓ | ✓ | ✓ | — |

Note: GitLab uses "merge requests" (MR) rather than pull requests. The ticket `prs` field and `apm link-pr` command work with MRs when provider type is `gitlab`.

### Local git hooks (auto-transitions without a provider or in addition to one)

`apm init --hooks` installs hooks that give partial automation even without a provider:

```sh
# .git/hooks/pre-push
# Detects first push of a feature/<id>-* branch → fires ready → in_progress
#!/bin/sh
apm _hook pre-push "$@"

# .git/hooks/post-merge
#!/bin/sh
apm sync --quiet --offline

# .git/hooks/post-checkout
#!/bin/sh
apm sync --quiet --offline
```

When a provider is also configured, hooks and provider events are both active. The state machine deduplicates — firing a transition that's already in the target state is a no-op.

---

## 4. Tech Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| CLI + server | **Rust** | Single binary, no runtime, fast startup, strong type safety for the ticket schema |
| Git access | **`git2` crate** (libgit2) | Programmatic access to commits, branches, blobs without shelling out |
| SQLite | **`rusqlite`** with `bundled` feature | Zero system dependency; cache is always available |
| Config parsing | **`toml` crate** | Native TOML support; aligns with Rust ecosystem conventions |
| Markdown parsing | **`pulldown-cmark`** | Parse and render ticket documents |
| Frontmatter | **`gray_matter`** or manual TOML block | Ticket frontmatter is TOML, body is markdown |
| Web server | **`axum`** | Ergonomic, async, tower-compatible |
| Web client | **React + TypeScript** | Rich interactive swimlane, drag-and-drop |
| Static asset embedding | **`include_dir!` / `rust-embed`** | React build embedded in binary; `apm serve` needs no separate files |
| GitHub API | **`octocrab`** or raw `reqwest` | For remote web client mode and PR event polling |

The binary is fully self-contained: `curl .../apm | install` and you're running.

---

## 5. Configuration Files

Two files live at the root of the ticket repo:

### `apm.toml` — machine-readable configuration

```toml
[project]
name = "ticker"
description = "Rust financial ticker application"

[tickets]
dir = "tickets"
archive_dir = "tickets/archive"   # optional; where closed tickets move
layer_boundary = "in_progress"    # state that triggers Layer 2 (branch) storage

[[repos.code]]
path = "org/ticker"
default_branch = "main"

[[repos.code]]
path = "org/ticker-frontend"
default_branch = "main"

# Git provider — omit this entire section for pure-git mode
[provider]
type = "github"
token_env = "APM_GITHUB_TOKEN"
poll_interval_secs = 60
webhook_secret_env = "APM_WEBHOOK_SECRET"

[workflow]
terminal_states = ["closed"]

# Each state declares its own outgoing transitions with actor and trigger.
# Full ticker workflow — see STATE-MACHINE.md §12 for complete reference.

[[workflow.states]]
id    = "new"
label = "New"
color = "#6b7280"
layer = 1

  [[workflow.states.transitions]]
  to      = "specd"
  trigger = "manual"
  actor   = "agent"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

  [[workflow.states.transitions]]
  to      = "question"
  trigger = "command:ask"
  actor   = "agent"

[[workflow.states]]
id    = "question"
label = "Question"
color = "#f59e0b"
layer = 1

  [[workflow.states.transitions]]
  to      = "new"
  trigger = "command:reply"
  actor   = "any"

  [[workflow.states.transitions]]
  to      = "specd"
  trigger = "manual"
  actor   = "agent"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

[[workflow.states]]
id    = "specd"
label = "Specd"
color = "#3b82f6"
layer = 1

  [[workflow.states.transitions]]
  to      = "ready"
  trigger = "manual"
  actor   = "supervisor"

  [[workflow.states.transitions]]
  to      = "ammend"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id    = "ammend"
label = "Ammend"
color = "#ef4444"
layer = 1

  [[workflow.states.transitions]]
  to      = "specd"
  trigger = "manual"
  actor   = "agent"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

  [[workflow.states.transitions]]
  to      = "question"
  trigger = "command:ask"
  actor   = "agent"

[[workflow.states]]
id    = "ready"
label = "Ready"
color = "#10b981"
layer = 1

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
  actor   = "agent"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"
color = "#8b5cf6"
layer = 2

  [[workflow.states.transitions]]
  to      = "implemented"
  trigger = "manual"
  actor   = "agent"
  preconditions = ["pr_exists"]

  [[workflow.states.transitions]]
  to           = "ready"
  trigger      = "manual"
  actor        = "any"
  side_effects = ["set_agent_null", "set_branch_null"]

[[workflow.states]]
id    = "implemented"
label = "Implemented"
color = "#06b6d4"
layer = 2

  [[workflow.states.transitions]]
  to      = "accepted"
  trigger = "manual"
  actor   = "engineer"
  preconditions = ["pr_all_closing_merged"]

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "manual"
  actor   = "any"

[[workflow.states]]
id    = "accepted"
label = "Accepted"
color = "#84cc16"
layer = 2

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id       = "closed"
label    = "Closed"
color    = "#374151"
layer    = 1
terminal = true

# Auto-transitions: fired by apm sync (polling/merged-branch detection) or git hooks.
# No daemon required — see §8 for event delivery model.

[[workflow.auto_transitions]]
on               = "event:branch_push_first"
from             = "ready"
to               = "in_progress"
requires_provider = false   # fires from local pre-push hook; no provider needed

[[workflow.auto_transitions]]
on               = "event:pr_opened"
from             = "in_progress"
to               = "implemented"
requires_provider = true    # needs provider API to detect open PR

[[workflow.auto_transitions]]
on               = "event:pr_all_merged"
from             = "implemented"
to               = "accepted"
requires_provider = false   # detected via git branch --merged; no provider needed

[workflow.prioritization]
# Primary sort: priority. Secondary: effort (smaller = higher score). Tertiary: risk (lower = higher score).
# Tickets with unset effort/risk are treated as high/high (conservative — assess before dispatching).
priority_weights = { urgent = 1000, high = 100, medium = 10, low = 1, none = 0 }
effort_weights   = { low = 20, medium = 10, high = 0 }
risk_weights     = { low = 5, medium = 2, high = 0 }

[agents]
name_pattern      = "{role}-{qualifier}"  # session name convention; not enforced
max_concurrent    = 3                     # max subagents dispatched simultaneously
actionable_states = ["new", "ammend", "ready"]  # states apm next will surface
```

### `apm.agents.md` — prose instructions for agents

A companion markdown file agents read before acting. Written in natural language, same format as `CLAUDE.md`. Checked into the ticket repo alongside `apm.toml`.

```markdown
# APM Agent Instructions

## Identity
Generate a unique session name at the start of every session and export it
before running any apm command:

```bash
export APM_AGENT_NAME=claude-$(date +%m%d-%H%M)-$(openssl rand -hex 2)
# example: claude-0325-1430-a3f9
```

This name appears in all commits, conversation entries, and the `agent` field
of tickets you work. It lets supervisors trace exactly which session produced
which changes — useful when a session needs to be reset and its commits
reviewed.

Hold the same name for the entire session. Do not regenerate mid-session.

Engineers set `APM_AGENT_NAME` to their own name when working directly.

## Startup
1. `apm sync` — refresh local cache from git
2. `apm agents` — read these instructions (once per session)
3. `apm status` — overview of the board
4. `apm next --json` — find the highest-priority ticket I can act on now
5. `apm list --working` — tickets where I am the active agent (resume if any)
6. `apm list --supervising --needs-action` — (engineer) tickets needing my input

## Working a ticket
First, check what state the ticket is in — the state determines what to do:

**state = `new`** (write the spec):
1. `apm show <id>` — read the full ticket
2. `apm set <id> effort <low|medium|high>` — assess implementation scale
3. `apm set <id> risk <low|medium|high>` — assess technical risk
4. `apm spec <id>` — write Problem, Acceptance criteria, Out of scope, Approach
5. If blocked: write question in `### Open questions`, then `apm state <id> question`
6. `apm state <id> specd` — submit spec for supervisor review

**state = `ammend`** (revise the spec):
1. `apm show <id> --spec` — read the Amendment requests
2. `apm spec <id>` — address each item, check boxes, update Approach
3. `apm state <id> specd` — resubmit (only when all amendment boxes are checked)

**state = `ready`** (implement):
1. `apm show <id>` — re-read the full spec before starting
2. `apm start <id>` — creates branch, sets agent = your name, moves to in_progress
3. Commit code to the feature branch; `apm spec <id>` for spec changes
4. `apm state <id> implemented` — after opening the PR

## Taking over another agent's ticket
If you are asked to continue work on a ticket already in_progress:
1. `apm show <id>` — read the full ticket including conversation
2. `apm take <id>` — checks out the branch, sets agent = your name
3. Continue from where the previous agent left off
Do not discard or overwrite the previous agent's conversation or spec work.

## Spec quality bar
Every spec must have all four sections before moving to `specd`:
- **Problem** — what is broken or missing, and why it matters
- **Acceptance criteria** — checkboxes; each one testable independently
- **Out of scope** — explicit list of what this ticket does not cover
- **Approach** — how the implementation will work

Do not mark acceptance criteria as checked until the implementation is verified.

## Spec discipline
- Set `effort` and `risk` before writing the spec — these drive prioritization
- Do not proceed on assumptions: write questions in `### Open questions`, change state to `question`
- Once a question is answered, reflect the decision in `### Approach`
- Do not delete answered questions or checked amendment items — they are the decision record

## Branch discipline
- All code changes on the feature branch (`feature/<id>-<slug>`)
- `apm spec` commits the full `## Spec` section to the feature branch
- Frontmatter and `## History` are committed to `main` by APM — never edit these directly
- Do not delete the feature branch until the ticket is closed — APM uses branch presence to detect merge state

## One ticket per agent process
- Work one ticket at a time per agent process
- For parallelism, run separate agent processes with separate clones
```

---

## 6. Ticket as a Document

Each ticket is a single markdown file with TOML frontmatter. The complete record of the work.

### File location

```
tickets/<id>-<slug>.md
```

Examples:
```
tickets/0042-add-dark-mode.md
tickets/0043-fix-csv-export.md
```

Zero-padded to 4 digits for stable lexicographic sort. Slug derived from title at creation (lowercase, hyphens, max 40 chars). Slug never changes even if title changes.

### Two-layer storage model

Tickets have two layers with strict section ownership. The split ensures that git's three-way merge at PR time produces a clean result with no conflicts.

**Layer 1 — Always on `main`**

| Section | Committed by |
|---------|-------------|
| Frontmatter | `apm state`, `apm set`, `apm start` |
| `## History` | APM on every state transition |

These sections are always current on `main`. The board and cache always read from `main`.

**Layer 2 — On the feature branch (after `apm start`)**

| Section | Committed by |
|---------|-------------|
| `## Spec` (all subsections) | `apm spec` |
| Code | Agent's normal git commits |

Once `in_progress` begins, `apm spec` commits the full spec section (including Open questions and Amendment requests subsections) to the feature branch. Main's spec is frozen at the moment the branch was created.

**At PR merge:** Git performs a clean three-way merge of the ticket file. The changes are non-overlapping: main changed the frontmatter and history; the branch changed only the spec. No conflict, no reconciliation step needed. The merge commit on `main` contains the complete ticket.

**After merge:** `apm sync` detects the branch in `git branch --merged main`, fires `event:pr_all_merged`, and commits the updated frontmatter (`state`, `updated_at`) to `main`. Routine state transition — not a special reconciliation.

**Why this split:** The board must show real state at all times (frontmatter on main). The spec evolves alongside the code and belongs with it (on branch). History is the permanent audit trail (on main).

### Ticket document format

Two top-level sections only: `## Spec` and `## History`. See `TICKET-SPEC.md` for the full schema.

```
+++
id          = 42
title       = "Add dark mode"
state       = "in_progress"
effort      = "medium"
risk        = "low"
priority    = 2
created_at  = "2026-03-22T10:00:00Z"
updated_at  = "2026-03-25T14:00:00Z"
author      = "philippe"
supervisor  = "philippe"
agent       = "claude-0322-b7d2"
branch      = "feature/42-add-dark-mode"
repos       = ["org/ticker"]

[[prs]]
number = 88
url    = "https://github.com/org/ticker/pull/88"
type   = "closes"
state  = "open"
review = "approved"
+++

## Spec

### Problem
Users cannot switch the UI to dark mode. All colors are hardcoded light-mode
values. This is a frequent support request.

### Acceptance criteria
- [ ] Dark mode toggle in Settings → Appearance
- [ ] Selection persists across sessions (localStorage)
- [ ] All components respond without page reload
- [ ] Tested on macOS Safari, Chrome, Firefox

### Out of scope
- System-level dark mode auto-detection
- Per-component dark mode overrides

### Open questions

**Q (claude-0322-b7d2, 2026-03-22):** Should we use existing design tokens or
create new dark-mode variants? The current token system has no dark definitions.

**A (philippe, 2026-03-22):** Create new ones with a `dark-` prefix. See Figma
wiki for the palette. No new Figma dependency needed in code.

### Approach
Use CSS custom properties. Define dark variants (`dark-bg`, `dark-surface`,
`dark-text`, `dark-border`) in `tokens.css`. Add `data-theme="dark"` to
`<html>`. Toggle via localStorage in Settings component.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-22T10:00Z | — | new | philippe |
| 2026-03-22T10:12Z | new | question | claude-0322-b7d2 |
| 2026-03-22T18:00Z | question | new | philippe |
| 2026-03-23T09:00Z | new | specd | claude-0322-b7d2 |
| 2026-03-23T09:30Z | specd | ready | philippe |
| 2026-03-25T10:00Z | ready | in_progress | claude-0322-b7d2 |
```

Frontmatter uses TOML delimiters `+++`. Body has exactly two top-level sections. Questions and amendments live inside `## Spec` as subsections, not in a separate conversation section.

### Field reference

| Field | Required | Set by | Type | Notes |
|-------|----------|--------|------|-------|
| `id` | yes | APM | integer | From `NEXT_ID`; never changes |
| `title` | yes | creator | string | Can be updated; slug does not change |
| `state` | yes | APM | string | Must be a state ID from `apm.toml` |
| `priority` | no | **supervisor** | integer | Business urgency: 0=none 1=urgent 2=high 3=medium 4=low |
| `effort` | no | **agent** | string | Implementation scale + complexity: `low` / `medium` / `high`. Set before writing spec. |
| `risk` | no | **agent** | string | Technical risk: `low` / `medium` / `high`. Set before writing spec. |
| `created_at` | yes | APM | RFC 3339 | Set once at creation; never changes |
| `updated_at` | yes | APM | RFC 3339 | Updated on every frontmatter write |
| `author` | yes | APM | string | Creator identity; never changes |
| `supervisor` | no | creator / `apm set` | string | Engineer responsible; can be reassigned |
| `agent` | no | `apm start` / `apm set` / `apm take` | string | Current worker; `apm set` reserves without branching |
| `branch` | no | `apm start` | string | Set when implementation begins; cleared on rollback |
| `prs` | no | APM via provider | array | See PR fields below |
| `repos` | no | creator / agent | array of strings | Code repos this ticket touches |

PR fields: `number`, `url`, `type` (`closes`\|`refs`), `state` (`open`\|`merged`\|`closed`), `review` (null\|`review_requested`\|`changes_requested`\|`approved`).

---

## 7. Ticket ID Generation

**Mechanism: `tickets/NEXT_ID` file**

Plain text file containing the next available integer.

```
# tickets/NEXT_ID
48
```

On ticket creation:
1. `git pull` to ensure local is current
2. Read `NEXT_ID` — claim this value as the new ticket ID
3. Write incremented value to `NEXT_ID`
4. Write new ticket file `tickets/0048-<slug>.md`
5. `git commit tickets/NEXT_ID tickets/0048-<slug>.md -m "ticket(48): create"`
6. `git push` — if rejected (concurrent creation), retry from step 1

**Properties:**
- Sequential integers — `apm show 42` works naturally
- No GitHub dependency — works with any git host or local-only
- Self-healing — `apm verify` detects duplicates; `apm repair --next-id` fixes `NEXT_ID` if it drifts

---

## 8. State Machine

Defined entirely in `apm.toml`. APM validates all transitions against it at runtime and in `apm verify`. See `STATE-MACHINE.md` for the full schema reference.

Default machine (ticker workflow):
```
new → question → specd → ready → in_progress → implemented → accepted → closed
            ↑
         ammend  (from specd; returns to specd after revision)
```

The `layer_boundary` in `apm.toml` (default: `in_progress`) defines where Layer 1 ends and Layer 2 begins. Any state at or after this position in `workflow.states` is a Layer 2 state — `apm spec` commits to the branch.

### Event delivery: not a daemon

APM does not run persistently. Auto-transitions fire through exactly two mechanisms:

**Local git hooks (synchronous):** The `pre-push` hook calls `apm _hook pre-push` as a one-shot process during `git push`. APM fires the transition, commits the frontmatter, and exits. No network required for the hook itself.

**`apm sync` (polling):** All other events are detected during sync. `apm sync` checks:
- `git branch --merged main` → detects merged feature branches → fires `event:pr_all_merged`
- Provider API (if configured) → detects open PRs, review state → fires `event:pr_opened`, `event:pr_review_*`

`apm sync` runs automatically via `post-merge` and `post-checkout` hooks, and at session start. If `apm serve` is running with a webhook secret configured, provider events arrive in real time — but this is an enhancement, not a requirement. State is never permanently wrong, only temporarily behind the last sync.

### Auto-transitions

| Event | Detection method | `requires_provider` | Default transition |
|-------|-----------------|--------------------|--------------------|
| `event:branch_push_first` | local `pre-push` hook | false | `ready → in_progress` |
| `event:pr_opened` | provider API poll | true | `in_progress → implemented` |
| `event:pr_all_merged` | `git branch --merged main` | false | `implemented → accepted` |
| `event:pr_review_changes` | provider API poll | true | `implemented → in_progress` |
| `event:pr_review_approved` | provider API poll | true | updates `ticket_prs.review_state` only |

When `requires_provider = false`, the event fires in pure-git mode. When `requires_provider = true`, it only fires when a provider is configured; otherwise the transition must be triggered manually:
```bash
apm state 42 implemented   # after opening a PR (pure-git fallback)
```

Auto-transitions can be disabled per-entry with `enabled = false`.

---

## 9. User-Global SQLite Cache

```
~/.apm/apm.db
```

One database for all repos. Repo is a first-class dimension in the schema. Start APM once, work across multiple repos from the same cache.

### Startup

```bash
apm                        # interactive repo picker if multiple registered
apm /path/to/repo          # use repo at this local path
apm org/ticker             # use repo by name (must be registered)
```

First time a repo is used, APM registers it and runs the initial sync. Config for the repo is read from the `apm.toml` inside it.

`APM_REPO` env var can set the default repo for scripting and agent use:
```bash
export APM_REPO=/Users/philippe/repos/ticker
apm list  # operates on ticker without prompting
```

### Schema

```sql
CREATE TABLE repos (
  id           TEXT PRIMARY KEY,  -- canonical path: "/Users/pp/repos/ticker"
  name         TEXT NOT NULL,     -- from apm.toml [project].name
  ticket_dir   TEXT NOT NULL DEFAULT 'tickets',
  remote_url   TEXT,              -- git remote origin URL
  last_sync_at TEXT,
  settings_sha TEXT               -- SHA of apm.toml at last sync; detect config changes
);

-- Rebuilt from ticket frontmatter on sync. Source of truth is always the git files.
CREATE TABLE tickets (
  repo_id      TEXT REFERENCES repos(id),
  id           INTEGER NOT NULL,
  slug         TEXT NOT NULL,
  title        TEXT NOT NULL,
  state        TEXT NOT NULL,
  effort       TEXT,
  risk         TEXT,
  priority     INTEGER DEFAULT 0,
  author       TEXT,              -- ticket creator; never changes
  supervisor   TEXT,              -- responsible engineer; can be reassigned
  agent        TEXT,              -- current worker; null until in_progress, changes on handoff
  branch       TEXT,
  repos_json   TEXT,              -- JSON array of code repo paths
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  file_sha     TEXT,              -- git blob SHA; detect staleness without reading file
  layer        INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (repo_id, id)
);

CREATE TABLE ticket_prs (
  repo_id      TEXT NOT NULL,
  ticket_id    INTEGER NOT NULL,
  pr_number    INTEGER NOT NULL,
  pr_url       TEXT,
  link_type    TEXT NOT NULL DEFAULT 'closes',
  state        TEXT NOT NULL DEFAULT 'open',
  review_state TEXT,
  opened_at    TEXT,
  merged_at    TEXT,
  PRIMARY KEY (repo_id, ticket_id, pr_number),
  FOREIGN KEY (repo_id, ticket_id) REFERENCES tickets(repo_id, id)
);

-- Pre-computed from git log; rebuilt on sync for changed tickets
CREATE TABLE state_durations (
  repo_id      TEXT NOT NULL,
  ticket_id    INTEGER NOT NULL,
  state        TEXT NOT NULL,
  entered_at   TEXT NOT NULL,
  exited_at    TEXT,             -- NULL = current state
  duration_s   INTEGER,          -- NULL = current state
  FOREIGN KEY (repo_id, ticket_id) REFERENCES tickets(repo_id, id)
);
```

### Sync

```bash
apm sync                   # pull + rebuild cache for current repo
apm sync --all             # sync all registered repos
apm sync --include-closed  # include terminal-state tickets
apm sync --offline         # rebuild from local git only (no pull)
```

Process:
1. `git pull` (skipped with `--offline`)
2. Read `apm.toml` — if `settings_sha` changed, reload workflow config
3. Walk `tickets/` on `main` — compare each file's blob SHA to `file_sha` in cache
4. For changed/new files: parse frontmatter, upsert `tickets` + `ticket_prs`
5. Skip files where `state` is in `terminal_states` unless `--include-closed`
6. **Merged-branch detection:** run `git branch --merged main`; for any open ticket whose `branch` field appears in the merged set, fire the `event:pr_all_merged` auto-transition if configured — commit frontmatter update to `main`, update cache
7. For tickets with a branch not in the merged set: check last commit timestamp for the activity indicator dot; do not read body from branch during sync (lazy — only on ticket open)
8. If provider configured: poll for open PRs on `feature/*` branches → fire `event:pr_opened` for any new links; poll review state for linked PRs → update `ticket_prs.review_state`
9. Recompute `state_durations` via `git log -- tickets/<file>` for tickets whose `file_sha` changed
10. Update `last_sync_at` and `settings_sha` in `repos`

**Speed:** For a repo with 50 open tickets where 3 changed since last sync, the file-walking steps touch only those 3 files. The merged-branch check is a single git command. Under 100ms for the local-only path.

### Auto-refresh hooks

Installed by `apm init --hooks`:

```sh
# .git/hooks/post-merge
#!/bin/sh
apm sync --quiet

# .git/hooks/post-checkout
#!/bin/sh
apm sync --quiet --offline  # no pull needed; just re-index after branch switch
```

---

## 10. Integrity Tool

```bash
apm verify              # full check
apm verify --fast       # pre-commit: schema + state machine only (< 100ms)
apm verify --fix        # auto-fix safe issues (NEXT_ID drift, cache staleness)
```

### Checks

| Check | --fast | Description |
|-------|--------|-------------|
| Frontmatter schema | ✓ | Required fields present, correct types |
| State validity | ✓ | `state` is a known state ID from `apm.toml` |
| Transition validity | ✓ | History table shows only allowed transitions |
| Actor validity | ✓ | Each transition in History was enacted by a permitted actor |
| Layer consistency | ✓ | Layer 2 state ↔ branch set |
| PR consistency | ✓ | `prs` non-empty only at or after the first Layer 2 state |
| Section ownership | ✓ | Conversation and History commits are on `main`; Spec commits are on branch |
| Duplicate IDs | — | Two files with same `id` |
| NEXT_ID consistency | — | `NEXT_ID` > max existing ID |
| Branch merged but ticket open | — | `git branch --merged main` contains ticket's branch but state is not `accepted` or `closed` (likely a missed auto-transition; `--fix` fires it) |
| Branch deleted prematurely | — | `branch` field set, state is `implemented` or earlier, but branch no longer exists locally or remotely |
| Cache staleness | — | SQLite `file_sha` differs from git blob SHA |
| Orphaned files | — | Files in `tickets/` that don't parse as valid ticket documents |

### Pre-commit hook

```bash
apm init --hooks
# installs .git/hooks/pre-commit → apm verify --fast
# installs .git/hooks/post-merge → apm sync --quiet
# installs .git/hooks/post-checkout → apm sync --quiet --offline
```

---

## 11. CLI Reference

### Startup and repo management

```bash
apm                                 # interactive repo picker
apm /path/to/repo                   # open repo at path
apm org/ticker                      # open repo by name
apm repos                           # list registered repos
apm repos add /path/to/repo         # register a new repo
apm repos remove org/ticker         # unregister
```

### Ticket management

```bash
# Create
apm new "Add dark mode"             # create ticket, open in $EDITOR
apm new "Add dark mode" -p high     # with priority flag

# Prioritized queue
apm next                            # highest-priority ticket in an actionable state
apm next --count 3                  # top 3 actionable tickets
apm next --json                     # machine-readable (for agent consumption)

# List (reads SQLite — instant)
apm list                            # all open tickets, priority order
apm list --state ready              # filter by state
apm list --state new,ammend,ready   # multiple states (all actionable states)
apm list --closed                   # include terminal states
apm list --supervising              # tickets where supervisor = APM_AGENT_NAME
apm list --working                  # tickets where agent = APM_AGENT_NAME
apm list --supervisor philippe      # tickets supervised by this engineer
apm list --agent claude-main        # tickets where this agent is active
apm list --needs-action             # tickets needing supervisor input
apm list --needs-action --supervising  # my tickets needing my action (primary engineer command)
apm list --unassigned               # actionable tickets with no agent set (available to pick up)

# Show
apm show 42                         # full ticket document
apm show 42 --spec                  # spec section only
apm show 42 --history               # history section only

# Edit
apm edit 42                         # open full file in $EDITOR
apm spec 42                         # open spec section in $EDITOR

# State and handoff
apm state 42 specd                  # move to state (validates preconditions, commits, syncs cache)
apm start 42                        # create branch, set agent = APM_AGENT_NAME, move to in_progress
                                    # errors if ticket.agent already set to another agent (use --force)
apm take 42                         # take over in_progress: checkout branch, set agent = APM_AGENT_NAME
apm checkout 42                     # git checkout the ticket's branch (without changing agent)
apm supervise 42                    # claim supervision: supervisor = APM_AGENT_NAME
apm set 42 supervisor alice         # assign supervisor to specific engineer
apm set 42 agent claude-A           # reserve ticket for an agent (no branch created yet)
apm set 42 priority high            # set priority (supervisor action)
apm set 42 effort medium            # set effort assessment (agent action)
apm set 42 risk low                 # set risk assessment (agent action)

# Agent dispatch
apm dispatch                        # spawn subagents up to [agents].max_concurrent
apm dispatch --max 5                # override max_concurrent for this run
apm dispatch --dry-run              # show which tickets would be dispatched, in order

# PR tracking (manual fallback when provider not configured)
apm link-pr 42 88                   # link PR #88 to ticket 42 (type: closes)
apm link-pr 42 88 --type refs       # non-closing link
apm link-pr 42 88 --url https://... # explicit URL (for non-GitHub remotes)
apm unlink-pr 42 88                 # remove PR link

# Maintenance
apm sync                            # pull + rebuild cache
apm sync --all                      # all registered repos
apm verify                          # integrity check
apm status                          # per-state counts
```

### Plumbing (agents and scripts)

```bash
apm get 42 state                    # print single field value
apm get 42 effort                   # print effort value
apm json 42                         # full ticket as JSON
apm json --list --state ready       # ticket list as JSON array
apm json --next                     # next actionable ticket as JSON (primary agent entrypoint)
```

### Environment variables

| Variable | Purpose |
|----------|---------|
| `APM_REPO` | Default repo path; avoids interactive picker |
| `APM_AGENT_NAME` | Identity for commits, history entries, and ticket `agent` field |
| `APM_TICKET` | Target ticket ID; used when spawning dedicated subagents |
| `APM_GITHUB_TOKEN` | GitHub PAT for provider API polling |
| `APM_EDITOR` | Override `$EDITOR` for ticket editing |

---

## 12. Web Client

### Purpose

The web client is a convenience for engineers who prefer a visual board over the CLI. It is **not** a central server. It runs locally, backed by the same git repo and SQLite cache the CLI uses. Remote access is a future consideration.

### Local mode (`apm serve`)

```bash
apm serve               # localhost:7070
apm serve --port 8080
```

The axum server:
- Serves the embedded React build (compiled into the binary via `rust-embed`)
- Exposes a local REST API: board reads from SQLite, ticket body reads from local git
- Writes shell out to `git commit` + `git push` (option A — uses the user's existing git config and credentials)

No network required for read. Push requires network only when syncing to remote.

### Remote access (future)

Not in scope for M0–M4. When needed, the same React app can be pointed at the GitHub Contents API directly. No server required for remote read; writes go through the GitHub API. Auth via PAT stored in localStorage.

### Board layout

Single page. Columns driven by non-terminal states in `apm.toml` order. `closed` hidden by default.

```
NEW  │  QUESTION  │  SPECD  │  READY  │  IN PROGRESS  │  IMPLEMENTED  │  ACCEPTED
```

**Default view:** tickets supervised by the current engineer (`supervisor = APM_AGENT_NAME`). Toggle in the header switches to "All tickets" for the full team view.

Each card:
- ID + title (truncated)
- Agent name (small, below title) — who is doing the work; blank if unassigned
- Time in state: color-coded (green < 2d · yellow 2–7d · red > 7d)
- Effort + risk badges
- `●` active dot if branch has commits in last 24h (requires git or GitHub access)
- PR review dot on IMPLEMENTED cards (⚪ none · 🟡 requested · 🔴 changes · 🟢 approved) — shown only when GitHub integration is enabled
- Repo tag (multi-repo setup)

Click card → ticket detail drawer (right panel, no navigation). Tabs: Spec · Conversation · History · PRs.

The "My board" view (supervised tickets, needs-action states highlighted) is the primary engineer daily interface. One glance answers: what needs me right now?

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| `C` | Create ticket |
| `J` / `K` | Navigate cards |
| `Enter` | Open detail drawer |
| `M` | Move to next state |
| `Cmd+K` | Command palette |
| `/` | Filter |
| `Esc` | Close drawer / clear |
| `R` | Refresh / sync |
| `G` `H` | Board (home) |

---

## 13. Multi-Repo Setup

`apm.toml` lists all code repos watched for git events. Tickets can reference multiple repos in their `repos` frontmatter field.

```toml
[[repos.code]]
path  = "org/main-service"          # remote identifier (org/repo)
local = "/Users/philippe/repos/main-service"  # local clone path
default_branch = "main"

[[repos.code]]
path  = "org/frontend"
local = "/Users/philippe/repos/frontend"
default_branch = "main"
```

`local` is required when the ticket repo is separate from the code repos (Option B topology). When tickets live in the same repo as the code, `local` can be omitted — APM uses the current repo path.

`apm start 42` creates `feature/42-<slug>` in each repo listed in the ticket's `repos` field (or prompts if unset). It uses the `local` path to run git operations.

The ticket repo itself is whichever repo contains `apm.toml` and `tickets/`. It can be a dedicated repo (`org/tickets`) or one of the code repos.

---

## 14. Build Order

### M0 — Foundation
- `apm.toml` parser (TOML, configurable state machine)
- `apm.agents.md` read and display
- Ticket document format (TOML frontmatter + markdown body)
- `NEXT_ID` generation
- `apm init`, `apm new`, `apm show`, `apm list`, `apm state`, `apm sync`, `apm status`
- User-global SQLite cache (`~/.apm/apm.db`) with repos table
- Direct commits to `main` for Layer 1 state transitions via `git2`
- Repo picker / `APM_REPO` / `apm <path>` startup

### M1 — Agent workflow
- Layer 2: `apm start` (branch creation, frontmatter commit to main, body ownership split)
- `apm checkout` — switch to ticket branch
- `apm ask` / `apm reply` — conversation entries committed to `main` (not branch)
- `apm spec` — spec editing committed to feature branch
- `APM_AGENT_NAME` identity in commits and conversation
- JSON plumbing output for agent consumption

### M2 — Integrity and hooks
- `apm verify` — all checks
- `apm verify --fast` — pre-commit subset
- `apm verify --fix` — auto-repair NEXT_ID, cache staleness
- `apm init --hooks` — install git hooks

### M3 — Web client read
- React board with configurable swimlane columns
- Ticket detail drawer (read-only; lazy-loads body from branch)
- Local mode: axum server + SQLite backend
- `rust-embed` for static asset bundling

### M4 — Web client write
- Drag to move state (commits via git API)
- Inline spec editing
- Reply to conversation
- Create ticket from board

### M5 — GitHub event handling
- Push events → `ready → in_progress` auto-transition
- PR events → `implemented` + review state on `ticket_prs`
- Polling via GitHub API (no webhook server required)
- Optional webhook relay server for real-time updates

### M6 — Multi-repo and polish
- Multi-repo branch creation
- Command palette
- Closed ticket archive
- `apm sync --all`
- `apm sync --include-closed` for analytics

---

## 15. Decisions Log

All prior open questions are resolved. Recorded here for reference.

| # | Question | Decision |
|---|----------|----------|
| Q-A | Two-layer model (frontmatter on main, body on branch)? | Yes |
| Q-B | GitHub Issues integration? | No — APM is self-contained; GitHub Issues is a future optional add-on |
| Q-C | SQLite location? | User-global `~/.apm/apm.db`; repo is a dimension in the schema |
| Q-D | Config format? | TOML (`apm.toml`); Rust implementation |
| Q-E | Agent instructions: embedded in `apm.toml` or companion file? | Companion `apm.agents.md` |
| Q-I | Frontmatter delimiter? | `+++` (TOML convention) |
| Q-II | TUI board view? | Not for now |
| Q-III | `apm serve` write path? | Option A — shell out to `git commit` + `git push` |
| Q-IV | Remote auth? | Tabled — remote mode is out of scope for M0–M4 |
| Q-V | `apm show` print `apm.agents.md`? | No — explicit `apm agents` command; agents read it once at startup |
| — | Supervisor field? | Yes — `supervisor` on every ticket; engineer responsible for Q&A and spec approval |
| — | GitHub required? | No — pure-git mode is the baseline; GitHub is an optional integration layer |
| — | GitHub role? | Automation: auto-transitions from push/PR/review events; PR state tracking |
| — | Provider strategy? | GitHub-primary with `GitProvider` trait abstraction; GitLab, Gitea/Forgejo, and none all slot in via the same trait; `[provider]` in `apm.toml` |
| — | Daemon required? | No — APM is a CLI; local git hooks handle push events synchronously; all other events polled during `apm sync` |
| — | Merge detection? | `git branch --merged main` — no webhook needed; `event:pr_all_merged` fires during `apm sync` |
| — | Branch discipline? | Spec on feature branch; frontmatter, conversation, history on `main`; git three-way merge handles reconciliation cleanly |
| — | When to delete feature branch? | Only after ticket is `closed` — branch presence is how APM detects merge state |
| — | State machine format? | Transitions nested under states with `trigger`, `actor`, `preconditions`, `side_effects`; see `STATE-MACHINE.md` |
| — | Agent session names? | Convention, not enforced. Agents generate a unique name at session start (`claude-MMDD-HHMM-xxxx`) and hold it for the session via `APM_AGENT_NAME`. Value: git blame per session, visible seams in conversation history. APM uses `unknown` if unset. |
| — | `apm ask` / `apm reply` commands? | Dropped as protocol. State is the signal (whose turn it is). Questions and amendments live in structured subsections of `## Spec`: `### Open questions` and `### Amendment requests`. See `TICKET-SPEC.md`. |
| — | Separate Conversation section? | Removed. Q&A is inline in the spec. History is the only non-spec body section. Ticket document has exactly two top-level sections: `## Spec` and `## History`. |
| — | Who sets priority vs effort/risk? | `priority` = supervisor (business urgency). `effort` + `risk` = agent (technical assessment, set before writing spec). These feed the prioritization formula. |
| — | Prioritization formula? | Score = priority_weight + effort_bonus + risk_bonus. Priority dominates; effort/risk break ties within tier. Weights configurable in `[workflow.prioritization]`. Unassessed tickets rank conservatively (treated as high effort + high risk). |
| — | `apm next` actionable states? | Configurable via `[agents].actionable_states`. Default ticker workflow: `["new", "ammend", "ready"]`. Agent reads state to determine what action to take (write spec / revise spec / start implementation). |
| — | Agent coordination mechanism? | `apm start` is the mutex — atomic git push claims the ticket. Concurrent starts: one push wins, other retries with next ticket. `apm set N agent <name>` reserves without branching (for pre-assignment). `apm dispatch` spawns subagents up to `max_concurrent`. |
