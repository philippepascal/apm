# APM — Agent Project Manager

A git-native ticket system for human+AI teams. Tickets are Markdown files with TOML frontmatter, stored on per-ticket branches. No database, no SaaS — just git.

APM ships as two binaries: `apm` (CLI) and `apm-server` (web UI). The UI replicates nearly all command-line functionality and includes its own agent dispatcher loop. The workflow states, ticket structure, completion strategies, and agent instructions are all fully customizable.

## Why APM

Traditional project management tools assume humans drive all work. APM is built for teams where AI agents and humans collaborate:

- **Agents pick up tickets autonomously** — `apm next` returns the highest-priority actionable ticket, `apm start` claims it and provisions a worktree
- **Supervisors review specs and implementations** — structured spec sections (Problem, Acceptance criteria, Approach) give reviewers clear decision points
- **State machine enforces workflow** — tickets follow defined transitions (new → groomed → specd → ready → in_progress → implemented → closed) with side paths for amendments, questions, and blocks
- **Everything is in git** — no external state to sync, no API keys to manage, works offline

## Design decisions

**Branch-per-ticket.** Each ticket lives on its own branch (`ticket/<id>-<slug>`). This means tickets can be created, edited, and synced without touching the working tree. `apm sync` polls all ticket branches and reconciles state.

**Permanent worktrees.** Implementation happens in git worktrees, not branch checkouts. The main directory always stays on `main`. Agents and engineers work in isolated worktrees that persist across sessions.

**Configurable state machine.** The workflow is defined in `.apm/config.toml` under `[[workflow.states]]`. States have properties like `terminal`, `actionable`, and allowed transitions. You can add states or change transitions to match your process.

**Completion strategies.** What happens when a ticket reaches `implemented` is configurable per-project:
- `pr` — push the branch and open a pull request
- `merge` — merge directly into the target branch
- `pr_or_epic_merge` — open a PR for standalone tickets, merge into the epic branch for epic tickets
- `none` — just push the branch, handle the rest manually

**Structured specs.** Tickets have required sections (Problem, Acceptance criteria, Out of scope, Approach) that agents write and supervisors review. Amendment requests and open questions are tracked as checkboxes, creating a decision record.

## Getting started

```bash
# Initialize in an existing git repo
apm init

# This creates .apm/config.toml with default workflow states
# Edit it to match your project's needs
```

## Working with tickets

```bash
# Create a ticket
apm new "Fix login timeout on slow connections"

# List tickets
apm list
apm list --state ready
apm list --state in_progress

# Show full ticket with spec
apm show <id>

# Set priority (higher = more urgent)
apm set <id> priority 10

# Transition a ticket's state
apm state <id> ready

# Review a spec — approve or request amendments
apm review <id>

# Force-close a ticket from any state
apm close <id> --reason "Superseded by #abcd1234"
```

## Epics

Group related tickets under an epic branch:

```bash
# Create an epic
apm epic new "Auth system rewrite"

# Create tickets targeting the epic
apm new "Migrate session storage" --epic <epic-id>

# List epics with ticket counts and derived state
apm epic list

# Show epic details
apm epic show <epic-id>

# When all tickets are done, open a PR from epic branch to main
apm epic close <epic-id>
```

With `pr_or_epic_merge` completion strategy, epic tickets merge into the epic branch when implemented, while standalone tickets open PRs to main.

## Syncing

```bash
# Sync with remote — fetches ticket branches, detects merges, updates state
apm sync
```

`apm sync` reconciles local state with remote. It detects merged PRs and transitions tickets to `closed`. With `aggressive = true` in config, most commands auto-sync before running.

## Housekeeping

```bash
# Archive closed tickets to archive/tickets/
apm archive
apm archive --dry-run
apm archive --older-than 30d

# Clean up worktrees and branches for closed tickets
apm clean
apm clean --dry-run
apm clean --branches        # also remove local branches
apm clean --remote          # also remove remote branches
```

## Configuration

All configuration lives in `.apm/config.toml`:

```toml
[project]
name = "myproject"

[tickets]
dir = "tickets"
archive_dir = "archive/tickets"

[worktrees]
dir = "../myproject--worktrees"

[[workflow.states]]
id = "new"
label = "New"

[[workflow.states]]
id = "ready"
label = "Ready"
actionable = true
transitions = ["in_progress"]

[[workflow.states]]
id = "in_progress"
label = "In Progress"
transitions = ["implemented", "blocked"]
completion_strategy = "pr"

[[workflow.states]]
id = "implemented"
label = "Implemented"

[[workflow.states]]
id = "closed"
label = "Closed"
terminal = true

[workers]
command = "claude"
args = ["--print"]

[agents]
instructions = ".apm/agents.md"
max_concurrent = 3

[sync]
aggressive = true
```

## Agent workflow

Agents work autonomously through the spec and implementation phases.

### Dispatching agents

```bash
# Spawn a worker on the next actionable ticket
apm start --next --spawn

# Run a dispatch loop (spawns workers up to max_concurrent)
apm work
```

### Spec phase (agent)

Agents pick up `groomed` tickets and write structured specs:

```bash
# Claim ticket for design
apm state <id> in_design

# Write spec sections
apm spec <id> --section Problem --set-file /tmp/problem.md
apm spec <id> --section "Acceptance criteria" --set "- [ ] Timeout is configurable..."
apm spec <id> --section Approach --set "Add a timeout_ms parameter to..."

# Set effort and risk estimates
apm set <id> effort 3
apm set <id> risk 2

# Submit for supervisor review
apm state <id> specd
```

The supervisor reviews and either moves the ticket to `ready` or `ammend` with amendment requests.

### Implementation phase (agent)

```bash
# Claim a ticket — provisions a worktree, sets state to in_progress
apm start <id>
# Prints the worktree path, e.g. ../myproject--worktrees/0001-fix-login-timeout

# Work in the worktree (never checkout in the main directory)
git -C <worktree-path> add src/auth.rs
git -C <worktree-path> commit -m "Increase default timeout to 30s"

# Mark as done — pushes branch and opens PR (depending on completion strategy)
apm state <id> implemented
```

### Agent instructions

APM generates agent instructions from `.apm/agents.md`. Agents receive these when spawned, covering workflow rules, shell discipline, and branch conventions.

```bash
# Print the current agent instructions
apm agents
```

## License

[Business Source License 1.1](LICENSE) — free to use, modify, and deploy. The one restriction: you may not offer APM as a hosted service. Converts to Apache 2.0 on 2030-04-05.
