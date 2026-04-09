# APM — Agent Project Manager

A free, open-source project management tool for small teams using AI agents for development.

**Goals:**

1. A free alternative to commercial project management offerings
2. A hosting-free solution — everything lives in your git repo, no SaaS required
3. An enabler for autonomous agent development that streamlines and speeds up project completion

**How APM achieves this:**

1. The git repo is the single repository for all tickets alongside the code — no database, no external state, works offline
2. All agents run independently on ticket and epic branches, in isolated worktrees that never interfere with each other
3. APM wraps git operations and offers a strict (but customizable) workflow that provides clarity and robustness

**Organization model:**

1. **Supervisors** (team members) configure APM — the workflow, the agents, the merge strategy — and control the overall flow. They create tickets, organize epics, review work throughout the workflow phases, and close tickets when done.
2. **Main agent** (optional) assists the supervisor with those tasks: putting together high-level plans, translating them into tickets, and managing the backlog.
3. **Workers** are agents that take tickets and work on them in their own branches. They use a subset of APM commands to guarantee a smooth flow. The number of parallel workers is up to the users, and they can be set to continuously pick work from the queue. Fine grain controls with default branches, epics and dependency allow parallel work while minimizing risks of conflicts.

APM ships with a default workflow, ticket format, and agent instructions, with Claude Code as the default worker agent. All of these are configurable — teams can swap in any CLI agent, (even use different agents for different tasks) and customize the workflow to match their process. This document walks through the main workflow using the defaults.

For more details, see the `docs/` directory:

- [docs/commands.md](docs/commands.md) — full command reference
- [docs/docker-workers.md](docs/docker-workers.md) — running workers in Docker containers
- [docs/external-tls-setup.md](docs/external-tls-setup.md) — TLS setup for the web UI

## The workflow

APM provides two binaries: `apm` (CLI) and `apm-server` (web UI). The UI replicates nearly all command-line functionality and includes its own agent dispatcher loop. Set the `EDITOR` environment variable to your preferred editor — `apm review` and other interactive commands use it to open tickets.

Tickets are Markdown files with TOML frontmatter, stored on per-ticket branches. Each ticket follows a state machine through its lifecycle:

```
new → groomed → in_design → specd → ready → in_progress → implemented → closed
                                 ↗ ammend ↗              ↗ blocked
                              question
```

Side paths handle amendments, open questions, and blocks. Supervisors control all transitions except the ones agents perform within their assigned phase.

### Happy path: one ticket from idea to merged

1. **Supervisor creates a ticket** — `apm new "Add rate limiting to API"`. APM creates a branch `ticket/a1b2c3d4-add-rate-limiting-to-api` with a Markdown skeleton.
2. **Supervisor grooms it** — adds context, sets priority, and moves it to `groomed` with `apm review a1b2` (which opens the ticket and presents available transitions).
3. **Spec agent picks it up** — but only if the ticket is assigned to it. The supervisor runs `apm assign a1b2 <agent-identity>` first. Then the dispatch loop (`apm work`) picks it up. The agent writes the Problem, Acceptance criteria, Out of scope, and Approach sections, sets effort/risk estimates, and submits with `apm state a1b2 specd`.
4. **Supervisor reviews the spec** — `apm review a1b2` opens the ticket and offers transitions: approve (moves to `ready`) or request amendments (moves to `ammend` with checkboxes). If amended, the agent addresses each item and resubmits.
5. **Implementation agent picks it up** — `apm start a1b2` claims the ticket, provisions a worktree, and merges the latest default branch in. The agent codes, tests, and commits inside the worktree.
6. **Agent marks it done** — `apm state a1b2 implemented` triggers the completion strategy: opens a PR, merges into the target branch, or merges into the epic branch.
7. **Supervisor reviews and closes** — `apm review a1b2` after the PR is merged (or the merge completes). Alternatively, `apm sync` auto-detects merged branches and transitions tickets to `closed`. `apm clean` removes the worktree and local branches.

The whole cycle runs without the supervisor touching any code or the agent touching the main branch.

## Install

### Homebrew (macOS)

```bash
brew install philippepascal/tap/apm
```

### Quick install

The install script detects your platform, downloads the latest release, verifies checksums, and adds the binaries to your `PATH`:

```bash
curl -fsSL https://raw.githubusercontent.com/philippepascal/apm/main/scripts/install.sh | sh
```

To uninstall:

```bash
curl -fsSL https://raw.githubusercontent.com/philippepascal/apm/main/scripts/uninstall.sh | sh
```

Pre-built binaries are available for macOS (Apple Silicon) and Linux (x86_64).

### From source

Requires Rust and Node.js 20+:

```bash
git clone https://github.com/philippepascal/apm.git
cd apm
(cd apm-ui && npm ci && npm run build)
cargo install --path apm
cargo install --path apm-server
```

## Try the demo

The [apm-demo](https://github.com/philippepascal/apm-demo) repo is a small Rust CLI project with 14 pre-populated tickets across all workflow states, an epic, cross-ticket dependencies, and an open amendment request. Clone it and explore:

```bash
git clone https://github.com/philippepascal/apm-demo.git
cd apm-demo
git fetch --all
apm list
apm-server   # browse at http://localhost:3000
```

Since apm-demo is read-only on GitHub, commands that push (state transitions, `apm start`, `apm work`) will fail. To get a fully functional copy you can write to, run `scripts/create-demo.sh` from the APM repo — it creates the demo under your own GitHub account.

## Getting started

```bash
# Initialize in an existing git repo
apm init

# Creates .apm/ with config.toml, workflow.toml, ticket.toml, and agent instructions
# Edit them to match your project's needs

# Start the web UI (defaults to http://localhost:3000)
apm-server
```

## Working with tickets

### Creating and managing tickets

```bash
# Create a ticket
apm new "Fix login timeout on slow connections"

# List tickets
apm list
apm list --state ready
apm list --mine

# Show full ticket with spec
apm show <id>

# Set priority (higher = more urgent)
apm set <id> priority 10

# Review a ticket — opens it and presents available transitions
apm review <id>

# Transition directly (lower-level alternative to review)
apm state <id> ready

# Force-close a ticket from any state
apm close <id> --reason "Superseded by #abcd1234"
```

### Epics

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

# Set max concurrent workers for an epic
apm epic set <epic-id> max_workers 2

# When all tickets are done, open a PR from epic branch to main
apm epic close <epic-id>
```

When a ticket reaches `implemented`, the completion strategy determines what happens next:

- **`pr`** — push the branch and open a pull request (default)
- **`merge`** — merge directly into the target branch
- **`pr_or_epic_merge`** — open a PR for standalone tickets, merge into the epic branch for epic tickets
- **`pull`** — pull the latest default branch into the ticket branch
- **`none`** — just push the branch, handle the rest manually

Strategies are configured per-transition in `workflow.toml`.

## Ticket ownership

Every ticket has two identity fields:

- **`author`** — set when the ticket is created; immutable. Records who created it.
- **`owner`** — who is responsible for the ticket. Dispatchers (`apm work`, `apm start --next`, the UI loop) only pick up tickets whose `owner` matches the current user's identity. Tickets with no owner are never auto-dispatched.

Assign a ticket before dispatching:

    apm assign <id> alice        # assign to alice
    apm assign <id> -            # clear the owner field

Bulk-assign all non-closed tickets in an epic at once:

    apm epic set <epic-id> owner alice

To filter the list by owner:

    apm list --owner alice       # tickets owned by alice
    apm list --mine              # tickets authored by the current user

### Identity setup

APM resolves the current user's identity in two modes:

**Config mode** (no `[git_host]` in `config.toml`): set `username` in `.apm/local.toml`:

    # .apm/local.toml
    username = "alice"

**GitHub mode** (`[git_host]` with `provider = "github"` in `config.toml`): identity is resolved from the `gh` CLI (if installed and authenticated) or from a GitHub token. No `local.toml` entry is needed — the GitHub login is used automatically.

## Agent workflow

Agents work autonomously through the spec and implementation phases. The supervisor dispatches them and reviews their output.

When a worker is spawned, APM automatically feeds it the right instruction file for the ticket's current state. By default, `apm init` creates two instruction files — `.apm/apm.spec-writer.md` for the spec phase and `.apm/apm.worker.md` for implementation. These are configured per-state in `workflow.toml` via the `instructions` field, and can be overridden per worker profile in `config.toml`.

The agent runtime itself is also configurable. The default is `claude --print`, but you can switch to any CLI agent by setting `command` and `args` under `[workers]` in `config.toml`. For example, to use Codex: `command = "codex"`, `args = ["--quiet"]`. You can also define named worker profiles under `[worker_profiles.<name>]` with their own `command`, `args`, `model`, `env`, and `instructions` — useful for running different agents for different phases or tasks.

### Dispatching agents

```bash
# Spawn a worker on the next actionable ticket
apm start --next --spawn

# Run a dispatch loop (spawns workers up to max_concurrent) until queue of actionable tickets is empty
apm work

# Run as a daemon — keeps dispatching as tickets become actionable
apm work --daemon
```

Dispatchers only pick up tickets whose `owner` matches the current user's identity. Assign tickets with `apm assign` before running `apm work`.

## Syncing and housekeeping

```bash
# Sync with remote — fetches ticket branches, detects merges, closes merged tickets
apm sync

# Archive closed tickets to archive/tickets/
apm archive
apm archive --dry-run --older-than 30d

# Clean up worktrees and branches for closed tickets
apm clean
apm clean --branches          # also remove local branches
apm clean --remote --older-than 30d  # also remove old remote branches
```

With `aggressive = true` in config, most commands auto-sync before running.

## Configuration

Configuration is split across files in `.apm/`:

| File | Purpose |
|------|---------|
| `config.toml` | Project settings, sync, workers, server |
| `workflow.toml` | State machine: states, transitions, completion strategies |
| `ticket.toml` | Ticket structure: sections, types, placeholders |
| `epics.toml` | Per-epic settings (e.g. `max_workers`) — untracked |
| `local.toml` | Per-user settings (username, worker overrides) — untracked |
| `agents.md` | Agent instructions: roles, workflow rules, shell discipline |
| `apm.spec-writer.md` | Instructions fed to agents during the spec phase |
| `apm.worker.md` | Instructions fed to agents during the implementation phase |

The workflow, ticket structure, completion strategies, and agent instructions are all fully customizable.

## How agents work

### Spec phase

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

The supervisor reviews with `apm review <id>`, which opens the spec and presents transition options: approve (moves to `ready`) or request amendments (moves to `ammend`). Specs have four required sections: Problem, Acceptance criteria, Out of scope, and Approach.

### Implementation phase

```bash
# Claim a ticket — provisions a worktree, sets state to in_progress
apm start <id>
# Prints the worktree path, e.g. ../myproject--worktrees/ticket-0001-fix-login-timeout

# Work in the worktree (never checkout in the main directory)
git -C <worktree-path> add src/auth.rs
git -C <worktree-path> commit -m "Increase default timeout to 30s"

# Mark as done — pushes branch and opens PR (depending on completion strategy)
apm state <id> implemented
```

## License

[Business Source License 1.1](LICENSE) — free to use, modify, and deploy. The one restriction: you may not offer APM as a hosted service. Converts to Apache 2.0 on 2030-04-05.
