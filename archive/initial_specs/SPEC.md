# APM — Specification V3

> **Status:** Draft · **Date:** 2026-03-25
> Git-native, agent-first, Linear-grade UX. Rust implementation.

---

## 1. Design Principles

1. **Git is the database.** No external database. Tickets are files in the repo. Git history is the audit trail. The entire system can be reconstructed from a git clone.
2. **Fast by default.** The board loads from a local cache populated by `apm sync`. Every action feels immediate regardless of network.
3. **Agents do the work; engineers supervise.** Tickets are worked by named agents. Engineers watch the board, answer questions, approve specs, and review PRs. Any engineer can pick up any ticket — handoff is a one-command operation, not a process.
4. **Handoff is zero-friction.** The ticket document contains everything needed to continue: open questions, resolved decisions, the evolving spec, full history. Check out the branch, read the ticket, keep going.
5. **CLI is complete.** Every operation available in the web client is available in the CLI. The web client is a local convenience, not a server or a requirement.
6. **Linear-grade feel.** Instant feedback, keyboard-first, dense but uncluttered, no unnecessary navigation.
7. **Configurable workflow.** The state machine, repos, and agent behavior are defined in `apm.toml` and `apm.agents.md`. APM ships with defaults suited to AI-assisted development; teams adapt them.
8. **Not a daemon.** APM is a CLI that runs on demand. State is derived from git on each `apm sync`. Local git hooks handle push events synchronously. Provider events are polled, not pushed. Correctness never depends on a background process.
9. **Integrity is verifiable.** A dedicated tool checks consistency of ticket files, the local cache, and the git history. Runs as a pre-commit hook.
10. **Provider-aware, not provider-locked.** APM is designed to be used with GitHub. It also works with GitLab, Gitea, Forgejo, and other providers through a provider abstraction layer. Pure-git mode (no provider) is a valid configuration for air-gapped or offline environments.
11. **Self-contained binary.** A single Rust binary provides the CLI, the local web server, and the embedded web client. No runtime dependencies beyond git.
12. **Main is clean.** Nothing commits directly to `main`. Ticket content arrives on `main` only through merged PRs. This is consistent with standard collaborative git workflows and safe for concurrent multi-engineer use.

---

## 2. Engineer / Agent Model

### Roles

**Engineers** supervise the work. Each ticket is assigned to one engineer — its **supervisor** — who is responsible for answering questions, approving specs, reviewing PRs, and deciding priority. An engineer sees their slice of the board: tickets they supervise. Any engineer can take over supervision of another's tickets; supervision is not exclusive.

**Agents** do the work. Each agent is a named process (`APM_AGENT_NAME`) that operates autonomously on one ticket at a time via the CLI. An agent writes the spec, asks clarifying questions, implements, and opens the PR. Its identity is recorded in the ticket's Open questions and History sections.

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
| `question` | Answer the agent's question (edit ticket file, change state back) |
| `specd` | Approve (`apm state N ready`) or request revision (`apm state N ammend`) |
| `ammend` | Nothing — agent is revising |
| `implemented` | Review the PR |
| `accepted` | Close the ticket (`apm state N closed`) |

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

Both can happen independently. A ticket can change supervisor without changing agent, and vice versa. Branch history and ticket content are always preserved.

---

## 3. Storage Model

### Branch per ticket

Every ticket has its own branch from the moment it is created. No ticket content ever commits directly to `main`.

**Branch naming:**

```
ticket/<id>-<slug>
```

Every ticket uses this name for its entire lifecycle — from creation through implementation to close. There is no rename at any point.

`apm new` creates the `ticket/<id>-<slug>` branch and commits the initial ticket file locally. The branch is **not pushed immediately** — `apm sync` is responsible for pushing all local ticket branches with unpushed commits.

The slug is derived from the title at creation: lowercase, hyphens, max 40 chars. It never changes.

### Local cache

Every `apm` command that reads tickets reads from a local cache. The cache is built and refreshed by `apm sync`.

```
apm sync:
  1. git fetch --all              → fetches all remote ticket/* branches
  2. For each ticket/* branch:
       git show <branch>:tickets/<id>-<slug>.md → parse frontmatter + body
  3. Check merged branches (git branch --merged main) → fire auto-transitions
  4. Write unified index to local tickets/ directory
  5. Update SQLite cache
  6. Push any local ticket/* branches with unpushed commits → origin
```

After sync, `apm list`, `apm next`, `apm show` read from the local `tickets/` directory — fast filesystem reads, no git or network overhead.

### What lands on `main`

The only ticket content that reaches `main` is what arrives via a merged PR. When a PR merges `ticket/<id>-<slug>` → `main`, the ticket file (with its complete spec, full history, and final frontmatter) is included in the merge commit. `apm sync` then detects the merged branch and fires the `event:pr_all_merged` auto-transition, committing the final state update (`implemented → accepted`) to `main` as a single post-merge commit.

In pure-git mode (no provider), there are no PRs. `apm state N closed` commits the final ticket state directly to `main` as the only direct main commit APM ever makes.

### Concurrent safety

With one branch per ticket and the one-agent-per-ticket rule, concurrent write conflicts are structurally prevented:

- Multiple agents working in parallel each own a different branch → no contention
- Supervisors write to ticket branches during spec phases (pre-`in_progress`) — only one agent is assigned per ticket, so supervisor and agent don't overlap
- After `in_progress`, code review uses the PR process; supervisors don't push to the feature branch directly
- The only shared resource is the `apm/meta` branch (for `NEXT_ID`) — handled by optimistic-lock retry (see §3.1)

### 3.1 Ticket ID allocation

IDs are allocated from a dedicated `apm/meta` branch. The branch contains a single `NEXT_ID` file. `apm new` uses an optimistic-lock protocol:

```
1. git fetch origin apm/meta
2. Read NEXT_ID (default 1 if branch doesn't exist)
3. Claim the ID locally
4. Increment and push NEXT_ID to apm/meta
5. If push fails (concurrent allocation): fetch, re-read, retry from step 3
```

Ticket creation concurrency is low enough that retries are rare. The retry loop has a maximum of 5 attempts before failing with a clear error.

---

## 4. Git Provider Integration

### Positioning

APM is designed to be used with **GitHub**. The full experience — automatic state transitions on push and PR events, PR review state on the board, remote web client access — requires a connected git provider. GitHub is the primary supported provider and the recommended setup.

APM is also **provider-aware**. The integration layer is abstracted so that GitLab, Gitea, Forgejo, and other self-hosted providers can be used in place of GitHub with equivalent functionality where their APIs allow it. For teams in air-gapped environments or with no remote provider, APM runs in pure-git mode using local git hooks for the subset of automation that doesn't require a remote API.

### What the provider enables

| Capability | With provider | Without provider |
|------------|--------------|-----------------|
| Board load, ticket CRUD | ✓ (local cache) | ✓ (local cache) |
| `ready → in_progress` auto-transition | ✓ via push hook | ✓ via local `pre-push` hook |
| `in_progress → implemented` auto-transition | ✓ via `apm sync` polling | Manual: `apm state 42 implemented` |
| `implemented → accepted` auto-transition | ✓ via `apm sync` (merged branch detection) | ✓ via `apm sync` (same — reads local git) |
| PR review state on board | ✓ | Not available |
| Activity indicator (recent branch commits) | ✓ | ✓ via local git |
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
token_env = "APM_GITHUB_TOKEN"
poll_interval_secs = 60
webhook_secret_env = "APM_WEBHOOK_SECRET"
```

**GitLab:**
```toml
[provider]
type = "gitlab"
host = "https://gitlab.example.com"
token_env = "APM_GITLAB_TOKEN"
webhook_secret_env = "APM_WEBHOOK_SECRET"
```

**Gitea / Forgejo:**
```toml
[provider]
type = "gitea"
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
| Remote web client | ✓ | ✓ | ✓ | — |
| Activity polling | ✓ | ✓ | ✓ | local git |
| Auto PR link detection | ✓ | ✓ | ✓ | — |

### Local git hooks

`apm init` installs hooks that provide partial automation even without a provider:

```sh
# .git/hooks/pre-push
# Detects first push of ticket/<id>-* branch in ready state → fires ready → in_progress
#!/bin/sh
command -v apm >/dev/null 2>&1 && apm _hook pre-push "$@" || true

# .git/hooks/post-merge
#!/bin/sh
command -v apm >/dev/null 2>&1 && apm sync --quiet --offline || true
```

When a provider is also configured, hooks and provider events are both active. The state machine deduplicates — firing a transition that's already in the target state is a no-op.

---

## 5. Tech Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| CLI + server | **Rust** | Single binary, no runtime, fast startup, strong type safety for the ticket schema |
| Git access | **`git2` crate** (libgit2) | Programmatic access to commits, branches, blobs without shelling out |
| SQLite | **`rusqlite`** with `bundled` feature | Zero system dependency; cache is always available |
| Config parsing | **`toml` crate** | Native TOML support; aligns with Rust ecosystem conventions |
| Markdown parsing | **`pulldown-cmark`** | Parse and render ticket documents |
| Web server | **`axum`** | Ergonomic, async, tower-compatible |
| Web client | **React + TypeScript** | Rich interactive swimlane, drag-and-drop |
| Static asset embedding | **`include_dir!` / `rust-embed`** | React build embedded in binary; `apm serve` needs no separate files |
| GitHub API | **`octocrab`** or raw `reqwest` | For remote web client mode and PR event polling |

The binary is fully self-contained: `curl .../apm | install` and you're running.

---

## 6. Configuration Files

Two files live at the root of the ticket repo:

### `apm.toml` — machine-readable configuration

```toml
[project]
name = "ticker"
description = "Rust financial ticker application"

[tickets]
dir         = "tickets"
archive_dir = "tickets/archive"   # optional; where closed tickets move on main

[[repos.code]]
path           = "org/ticker"
default_branch = "main"

[[repos.code]]
path           = "org/ticker-frontend"
default_branch = "main"

# Git provider — omit this entire section for pure-git mode
[provider]
type               = "github"
token_env          = "APM_GITHUB_TOKEN"
poll_interval_secs = 60
webhook_secret_env = "APM_WEBHOOK_SECRET"

[workflow]
terminal_states = ["closed"]

[agents]
max_concurrent    = 3
actionable_states = ["new", "ammend", "ready"]
instructions      = "apm.agents.md"

# Each state declares its own outgoing transitions with actor and trigger.
# Full ticker workflow — see STATE-MACHINE.md §12 for complete reference.

[[workflow.states]]
id    = "new"
label = "New"
color = "#6b7280"

  [[workflow.states.transitions]]
  to            = "specd"
  trigger       = "manual"
  actor         = "agent"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

  [[workflow.states.transitions]]
  to      = "question"
  trigger = "command:ask"
  actor   = "agent"

[[workflow.states]]
id    = "question"
label = "Question"
color = "#f59e0b"

  [[workflow.states.transitions]]
  to      = "new"
  trigger = "command:reply"
  actor   = "any"

  [[workflow.states.transitions]]
  to            = "specd"
  trigger       = "manual"
  actor         = "agent"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

[[workflow.states]]
id    = "specd"
label = "Specd"
color = "#3b82f6"

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

  [[workflow.states.transitions]]
  to            = "specd"
  trigger       = "manual"
  actor         = "agent"
  preconditions = ["spec_not_empty", "spec_has_acceptance_criteria"]

  [[workflow.states.transitions]]
  to      = "question"
  trigger = "command:ask"
  actor   = "agent"

[[workflow.states]]
id    = "ready"
label = "Ready"
color = "#10b981"

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "command:start"
  actor   = "agent"

[[workflow.states]]
id    = "in_progress"
label = "In Progress"
color = "#8b5cf6"

  [[workflow.states.transitions]]
  to            = "implemented"
  trigger       = "manual"
  actor         = "agent"
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

  [[workflow.states.transitions]]
  to            = "accepted"
  trigger       = "manual"
  actor         = "engineer"
  preconditions = ["pr_all_closing_merged"]

  [[workflow.states.transitions]]
  to      = "in_progress"
  trigger = "manual"
  actor   = "any"

[[workflow.states]]
id    = "accepted"
label = "Accepted"
color = "#84cc16"

  [[workflow.states.transitions]]
  to      = "closed"
  trigger = "manual"
  actor   = "supervisor"

[[workflow.states]]
id       = "closed"
label    = "Closed"
color    = "#374151"
terminal = true

# Auto-transitions

[[workflow.auto_transitions]]
on                = "event:branch_push_first"
from              = "ready"
to                = "in_progress"
requires_provider = false

[[workflow.auto_transitions]]
on                = "event:pr_opened"
from              = "in_progress"
to                = "implemented"
requires_provider = true

[[workflow.auto_transitions]]
on                = "event:pr_all_merged"
from              = "implemented"
to                = "accepted"
requires_provider = true

[[workflow.auto_transitions]]
on                = "event:pr_review_changes"
from              = "implemented"
to                = "in_progress"
requires_provider = true
```

### `apm.agents.md` — agent behavior instructions

Natural language instructions read by agent processes at session start. Describes how to pick up tickets, write specs, handle amendments, and follow branch discipline. See `apm.agents.md` at the repo root.

---

## 7. CLI Reference

### Core commands

```
apm init               Initialise APM in the current git repo
apm sync               Fetch all ticket/feature branches; rebuild local cache; fire auto-transitions
apm new                Create a ticket (branch + initial file)
apm list               List tickets from local cache
apm show <id>          Show a ticket (frontmatter + body)
apm next               Show the next actionable ticket (highest priority, unassigned)
apm state <id> <state> Transition a ticket to a new state
apm set <id> <field> <value>  Set a frontmatter field (priority, effort, risk, supervisor, agent)
apm start <id>         Begin implementation: rename ticket branch → feature branch, set agent
apm take <id>          Take over an in-progress ticket: checkout branch, set agent = self
apm spec <id>          Open the ticket's spec in $EDITOR (routing to the correct branch)
apm ask <id> "..."     Append a question to Open questions; fire question transition if configured
apm reply <id> "..."   Append a reply to Open questions; fire reply transition if configured
apm link-pr <id> <pr>  Manually link a PR to a ticket (pure-git fallback)
apm verify             Check ticket consistency (spec completeness, cache coherence, branch state)
apm _hook <event>      Internal: called by git hooks
apm serve              Start local web server with embedded web client
```

### `apm spec <id>`

Opens the ticket file in `$EDITOR`. APM determines which branch the ticket is on, checks out that branch in a detached worktree (or switches to it), and opens the file. On save, APM commits the change to the ticket's branch and pushes.

This is the canonical way to edit a ticket spec. Direct file editing works in simple setups but bypasses APM's branch routing.

### `apm sync` detail

```
apm sync [--quiet] [--offline]

--quiet     Suppress output; suitable for git hooks
--offline   Skip git fetch; re-process local branches only (fast, no network)
```

Running `apm sync` at session start is required before any other command. It ensures the local cache reflects the current remote state.

---

## 8. Event Delivery: polling, not a daemon

APM is not a daemon. Auto-transitions fire through two mechanisms only:

### Local git events → synchronous hooks

The `pre-push` hook calls `apm _hook pre-push` as a one-shot process when the agent runs `git push`. APM fires the transition, commits the state update to the feature branch, and exits.

```
agent runs: git push (first push of ticket/<id>-*)
  → pre-push hook fires
  → apm _hook pre-push (detects ticket/<id>-* branch in ready state)
  → fires ready → in_progress
  → commits state update to feature branch
  → exits
git push completes
```

### Remote events → polling via `apm sync`

```
apm sync runs:
  → git fetch --all
  → for each ticket/* branch:
      check: is branch merged into main? (git branch --merged main)
      check: does a PR exist? (GitHub API or local git)
      check: PR review state? (GitHub API)
  → fire auto-transitions whose conditions are newly met
  → commit state updates to the relevant ticket/feature branch
  → update local cache
```

`apm sync` runs:
- Required: at session start
- Automatically: on `post-merge` hook
- Optionally: via cron for background refresh

### Latency table

| Setup | `branch_push` latency | `pr_opened` latency | `pr_merged` latency |
|---|---|---|---|
| hooks + `apm serve` + webhooks | instant (hook) | seconds (webhook) | seconds (webhook) |
| hooks + manual `apm sync` | instant (hook) | next sync | next sync |
| pure-git (no provider) | instant (hook) | manual `apm state` | manual `apm state` |

---

## 9. Merge Detection and Post-merge State

### How APM knows a branch was merged

APM asks git:

```
git branch --merged main
```

If `ticket/42-add-csv-export` appears in that list, ticket #42's branch is merged. APM detects this during `apm sync` and fires `event:pr_all_merged`. **No webhook required. Branches are not deleted until the ticket is closed.**

### What the merge commit contains

When `ticket/42-*` merges into `main`, the merge commit contains the complete ticket file as it existed on the ticket branch — full spec, full history, final frontmatter. No additional reconciliation is needed.

### The one post-merge commit

After detecting a merged branch, APM fires `event:pr_all_merged` and must update the `state` field. This produces one commit to `main`:

```
commit a3f9b12
Author: apm <apm@local>

    ticket(42): implemented → accepted [branch merged]
```

This is the only APM-originated commit that goes to `main`. Everything else arrives via merged PRs.

### Branch cleanup

After a ticket reaches `closed`, its feature branch can be deleted:

```bash
apm state 42 closed    # commits state update to main
git push origin --delete ticket/42-add-csv-export
```

APM does not auto-delete branches. Keeping them until explicitly closed preserves the `git branch --merged main` signal.

---

## 10. Multi-engineer Safety

All ticket writes go to that ticket's own branch. `main` is only written to via merged PRs (plus the single post-merge state commit from APM). This means:

- **No concurrent writes to `main`** from engineers or agents
- **No rebase fights** on a shared ticket branch — one agent per ticket prevents this
- **Supervisors edit their tickets** on the ticket branch during spec phases; since agents are not yet assigned, there is no concurrent writer
- **After `in_progress`** — agents and supervisors communicate via PRs and review comments, not by both pushing to the feature branch

The `apm/meta` branch (for ID allocation) is the only shared-write resource, protected by optimistic-lock retry.

---

## 11. `apm init` Setup

```bash
apm init
```

Creates:
- `tickets/` directory (local cache target)
- `apm.toml` with default workflow
- `.gitignore` entry for `tickets/NEXT_ID` (legacy; unused in V3)
- `.git/hooks/pre-push` (executable)
- `.git/hooks/post-merge` (executable)
- Initial commit of `apm.toml` and `apm.agents.md` to `main`
- `apm/meta` branch with `NEXT_ID = 1`

After `apm init`, the repo is ready. Run `apm new` to create the first ticket.
