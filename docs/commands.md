# APM Command Reference

## Introduction

APM (Agent Project Manager) is a git-native ticket system designed for teams where humans and AI
agents collaborate. Tickets are Markdown files stored on per-ticket branches (`ticket/<id>-<slug>`). Tickets provide a full history of all specs used to develop the project.
State is encoded in TOML frontmatter at the top of each file.
The state machine is defined in `.apm/workflow.toml` and is entirely customizable.
The ticket structure is defined in `.apm/tickets.toml` and is entirely customizable.
Merge strategies primitives, epics with dedicated branches, ticket with dedicated branches, and ticket dependencies, all allow to have worker agents work independantly, and optionally concurrently, while keeping minimal merge conflicts.

This document is the authoritative reference for every command exposed by the `apm` binary. It
covers the exact invocation syntax, all flags and arguments with their types and defaults, and a
detailed breakdown of the git operations each command performs internally. `apm-serve` internally uses the same `apm-core` library as the CLI, so all UI actions work the same way, apart from the UI dispatcher which is slightly different from `apm work`.

**How to navigate this reference:**

| Section | Commands |
|---------|----------|
| [Ticket lifecycle](#ticket-lifecycle) | `assign`, `close`, `new`, `set`, `state` |
| [Inspection](#inspection) | `list`, `next`, `show`, `spec` |
| [Workflow orchestration](#workflow-orchestration) | `review`, `start`, `sync`, `work`, `workers` |
| [Epics](#epics) | `epic new`, `epic close`, `epic list`, `epic show` |
| [Repository maintenance](#repository-maintenance) | `archive`, `clean`, `init`, `validate`, `verify`, `worktrees` |
| [Server & agent management](#server--agent-management-requires-apm-server) | `agents`, `register`, `revoke`, `sessions` |
| [Internal commands](#internal-commands) | `_hook` |

**Aggressive mode:** Many commands accept `--no-aggressive`. When `sync.aggressive = true` in
`.apm/apm.toml` (the default), commands automatically fetch from the remote before reading ticket
data and push after writing. Pass `--no-aggressive` to skip these implicit git operations.

**ID resolution:** Every command that accepts a ticket `<id>` accepts a plain integer (`42` →
padded to `00000042`), a 4+ character hex prefix (`00ab`), or the full 8-character hex ID.

---

## Ticket lifecycle

Commands that create or mutate ticket metadata.

---

### apm assign

**Set the owner field on a ticket.**

#### Synopsis

    apm assign <id> <username>
    apm assign <id> -

#### Description

Sets the `owner` field in the ticket's TOML frontmatter to `<username>`, then commits the change
directly to the ticket's branch. The `owner` field records who is responsible for the ticket
regardless of its state, making it distinct from the `agent` field (set automatically by
`apm start`).

Pass `-` as the username to clear the `owner` field.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Ticket ID to update |
| `<username>` | positional | — | Username to assign, or `-` to clear |
| `--no-aggressive` | flag | false | Skip implicit git fetch and push |

#### Git internals

| Command | Why |
|---------|-----|
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Enumerate all local and remote ticket branches to resolve the ID |
| `git show <branch>:<path>` | Read ticket file content for every discovered branch |
| `git fetch origin <branch>` | (aggressive only) Pull latest commits on the ticket branch before writing |
| `git add <path>` + `git commit -m "ticket(<id>): assign owner = <user>"` | Persist the updated `owner` field to the ticket branch |
| `git push origin <branch>` | (aggressive only) Publish the owner change to the remote |

---

### apm close

**Force-close a ticket from any state (supervisor only).**

#### Synopsis

    apm close <id> [--reason <text>] [--no-aggressive]

#### Description

Bypasses the normal state machine and closes the ticket immediately, regardless of its current
state. An optional `--reason` string is appended to the ticket's `## History` section for the
record.

This is an escape hatch for tickets that are abandoned, duplicated, or otherwise need to be removed
from the active queue without following the normal workflow. Prefer `apm state <id> closed` when
the ticket has been properly resolved through the standard flow.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Ticket ID to close |
| `--reason` | string | — | Optional reason appended to the history entry |
| `--no-aggressive` | flag | false | Skip implicit git fetch and push |

#### Git internals

| Command | Why |
|---------|-----|
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Enumerate ticket branches to resolve the ID |
| `git show <branch>:<path>` | Read all ticket content to locate the target ticket |
| `git fetch origin <branch>` | (aggressive only) Sync before writing to avoid conflicts |
| `git add <path>` + `git commit -m "ticket(<id>): closed"` | Write the closed state and history entry to the ticket branch |
| `git push origin <branch>` | (aggressive only) Publish the closure to the remote |

---

### apm new

**Create a new ticket and its branch.**

#### Synopsis

    apm new <title> [--no-edit] [--side-note] [--context <text>] [--context-section <name>]
            [--section <name> --set <text>] ... [--epic <id>] [--depends-on <ids>]
            [--no-aggressive]

#### Description

Creates a ticket Markdown file on a new branch (`ticket/<id>-<slug>`) with state `new`, then
opens `$VISUAL` / `$EDITOR` (falling back to `vi`) so the author can fill in the spec immediately.
After the editor closes the spec is committed to the ticket branch and the working tree returns to
the branch that was active before.

Agents must always pass `--no-edit` to skip the interactive editor:

    apm new --no-edit "Short title"

Use `--side-note` together with `--context` to capture an out-of-scope observation without
interrupting the current ticket. Use `--epic <id>` to create the ticket branch from the tip of an
epic branch rather than from HEAD.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<title>` | positional | — | Short title for the ticket |
| `--no-edit` | flag | false | Skip opening `$EDITOR` after creation |
| `--side-note` | flag | false | Mark ticket as a side-note (out-of-scope observation) |
| `--context` | string | — | Text to insert into the first spec section (or `--context-section`) |
| `--context-section` | string | — | Section name for `--context` (requires `--context`) |
| `--section` | string (repeatable) | — | Section name to pre-populate (paired with `--set`) |
| `--set` | string (repeatable) | — | Content for the preceding `--section` |
| `--epic` | string | — | Epic ID; ticket branch is created from the epic branch tip |
| `--depends-on` | string (repeatable) | — | Comma-separated ticket IDs this ticket depends on |
| `--no-aggressive` | flag | false | Skip implicit git fetch and push |

#### Git internals

| Command | Why |
|---------|-----|
| `git rev-parse <epic-branch>` | (with `--epic`) Resolve the epic branch tip to use as the new branch base |
| `git branch ticket/<id>-<slug> <sha>` | Create the ticket branch at the epic or HEAD commit |
| `git worktree add --detach` + `git checkout -B` | (temp worktree path) Write ticket skeleton to the new branch without touching the working tree |
| `git add <path>` + `git commit -m "ticket(<id>): create <title>"` | Commit the initial ticket file to the branch |
| `git push origin ticket/<id>-<slug>` | (aggressive only) Push the new branch to the remote |
| `git rev-parse --abbrev-ref HEAD` | Record the current branch so we can return to it after editing |
| `git checkout ticket/<id>-<slug>` | Check out the ticket branch so the editor can write to the working tree |
| `git add <path>` + `git commit --allow-empty -m "write spec"` | Stage and commit the spec after the editor closes |
| `git checkout <prev-branch>` | Return to the branch that was checked out before `apm new` was invoked |

---

### apm set

**Set a single metadata field on a ticket.**

#### Synopsis

    apm set <id> <field> <value> [--no-aggressive]

#### Description

Updates one metadata field in the ticket's TOML frontmatter and commits the change to the ticket's
branch. Valid field names are `priority`, `effort`, `risk`, `title`, `agent`, `supervisor`,
`branch`, and `depends_on`. Pass `-` as the value to clear the `agent`, `supervisor`, `branch`, or
`depends_on` fields.

    apm set 42 priority 5
    apm set 42 agent alice
    apm set 42 agent -              # clear agent field
    apm set 42 depends_on abc123,def456

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Ticket ID to update |
| `<field>` | positional | — | Field name: `priority`, `effort`, `risk`, `title`, `agent`, `supervisor`, `branch`, `depends_on` |
| `<value>` | positional | — | New value; use `-` to clear `agent`, `supervisor`, `branch`, or `depends_on` |
| `--no-aggressive` | flag | false | Skip implicit git fetch and push |

#### Git internals

| Command | Why |
|---------|-----|
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Enumerate ticket branches to resolve the ID |
| `git show <branch>:<path>` | Read ticket content for all discovered branches |
| `git fetch origin <branch>` | (aggressive only) Sync before writing |
| `git add <path>` + `git commit -m "ticket(<id>): set <field> = <value>"` | Persist the field change to the ticket branch |
| `git push origin <branch>` | (aggressive only) Publish the change to the remote |

---

### apm state

**Transition a ticket to a new state.**

#### Synopsis

    apm state <id> <state> [--force] [--no-aggressive]

#### Description

Transitions a ticket from its current state to `<state>`. The allowed transitions are defined in
`.apm/apm.toml` under `[[workflow.states]]`; illegal transitions are rejected. Transitioning to
`specd` validates that all required spec sections are non-empty; transitioning to `implemented`
validates that all acceptance criteria checkboxes are checked.

Pass `--force` to bypass the transition graph (escape hatch for stuck tickets). The target state
must still exist in the configuration; document-level validations still apply.

Certain transitions trigger additional side-effects depending on the `completion` strategy
configured for that transition: `pr` opens a GitHub pull request, `merge` merges the branch into
the default branch, and `pull` pulls the default branch.

Transitioning to `in_design` also provisions a permanent worktree for the ticket branch.

    apm state 42 in_design      # claim a new ticket for spec writing
    apm state 42 specd           # submit spec for review
    apm state 42 implemented     # mark implementation done
    apm state 42 new --force     # reset a stuck ticket

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Ticket ID to transition |
| `<state>` | positional | — | Target state name (e.g. `in_design`, `specd`, `ready`, `in_progress`, `implemented`, `closed`) |
| `--force` | flag | false | Bypass transition rules; target state must still exist in config |
| `--no-aggressive` | flag | false | Skip implicit git fetch and push |

#### Git internals

| Command | Why |
|---------|-----|
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Enumerate ticket branches to resolve the ID |
| `git show <branch>:<path>` | Read ticket content |
| `git fetch origin <branch>` | (aggressive only) Sync before writing |
| `git add <path>` + `git commit -m "ticket(<id>): <old> → <new>"` | Write the new state and history entry to the ticket branch |
| `git push origin <branch>` + `gh pr create` | (`completion = "pr"`) Push branch and open a GitHub PR targeting the default branch |
| `git push origin <branch>` + `git merge <branch>` | (`completion = "merge"`) Push branch and merge it into the default branch |
| `git pull origin <default-branch>` | (`completion = "pull"`) Pull latest default branch |
| `git push origin <branch>` | (aggressive + `completion = "none"`) Publish the state change |
| `git fetch origin <branch>` + `git worktree add <path> <branch>` | (`in_design` target) Provision a permanent worktree for the ticket |

---

## Inspection

Read-only commands for querying ticket data.

---

### apm list

**List tickets with optional filtering.**

#### Synopsis

    apm list [--state <state>] [--unassigned] [--all] [--supervisor <name>]
             [--actionable <actor>] [--mine] [--author <username>] [--owner <username>]
             [--no-aggressive]

#### Description

Prints a formatted table of tickets, one per line, with ID, state, and title. By default, tickets
in terminal states (e.g. `closed`) are hidden; pass `--all` to include them. Multiple filter flags
may be combined.

    apm list                          # all non-terminal tickets
    apm list --state ready            # only tickets awaiting an agent
    apm list --unassigned             # no agent assigned yet
    apm list --actionable agent       # tickets an agent can act on now
    apm list --mine                   # your own tickets

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--state` | string | — | Show only tickets in this state |
| `--unassigned` | flag | false | Show only tickets with no `agent` field set |
| `--all` | flag | false | Include terminal-state tickets (e.g. `closed`) |
| `--supervisor` | string | — | Show only tickets with this supervisor |
| `--actionable` | string | — | Show only tickets actionable by this actor (`agent`, `supervisor`, `engineer`) |
| `--mine` | flag | false | Show only tickets authored by the current user |
| `--author` | string | — | Show only tickets authored by this username (conflicts with `--mine`) |
| `--owner` | string | — | Show only tickets owned by this username (conflicts with `--mine`) |
| `--no-aggressive` | flag | false | Skip implicit git fetch |

#### Git internals

| Command | Why |
|---------|-----|
| `git fetch --all --quiet` | (aggressive only) Sync all remote refs before reading ticket data |
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Build the full list of ticket branches |
| `git show <branch>:<path>` | Read each ticket file directly from its branch blob |

---

### apm next

**Return the highest-priority actionable ticket.**

#### Synopsis

    apm next [--json] [--no-aggressive]

#### Description

Selects the highest-priority ticket that the current actor (always `agent`) can act on right now,
using a weighted score of priority, effort, and risk. Tickets whose blockers are not yet resolved
are excluded. Returns nothing (exit 0, empty output) when no actionable ticket exists.

With `--json` the result is a single JSON object:

    {"id": "0042ab12", "title": "...", "state": "ready", "score": 3.5}

or `null` when the queue is empty.

    apm next --json   # check for work in agent startup scripts

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--json` | flag | false | Output result as JSON instead of human-readable text |
| `--no-aggressive` | flag | false | Skip implicit git fetch |

#### Git internals

| Command | Why |
|---------|-----|
| `git fetch --all --quiet` | (aggressive only) Sync all remote refs before selecting the next ticket |
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Enumerate all ticket branches |
| `git show <branch>:<path>` | Read every ticket to evaluate priority and state |

---

### apm show

**Show the full content of a ticket.**

#### Synopsis

    apm show <id> [--edit] [--no-aggressive]

#### Description

Reads the ticket file directly from its branch blob in the git object store and prints it to
stdout. The working tree does not need to be checked out on the ticket branch.

If the ticket ID cannot be found on any ticket branch, `apm show` falls back to searching
`tickets/` on the default branch, then the configured `archive_dir` if set. This allows reading
archived tickets that no longer have an active branch.

Pass `--edit` to open the ticket in `$VISUAL` / `$EDITOR`. After saving, any changes are committed
back to the ticket branch. `--edit` is not supported for archived tickets.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Ticket ID (8-char hex, 4+ char prefix, or plain integer) |
| `--edit` | flag | false | Open in `$EDITOR` and commit changes back to the branch |
| `--no-aggressive` | flag | false | Skip implicit git fetch |

#### Git internals

| Command | Why |
|---------|-----|
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Find the ticket branch by ID |
| `git fetch origin <branch>` | (aggressive only) Pull latest before reading |
| `git show <branch>:<path>` | Read the ticket file from the branch blob without checking it out |
| `git ls-tree <branch> <dir>` | (fallback) List files on the default branch when no ticket branch matches the ID |
| `git show <default-branch>:<path>` | (fallback) Read from the default branch or archive dir |
| `git add <path>` + `git commit -m "ticket(<id>): edit"` | (`--edit` only) Commit the edited content back to the ticket branch |

---

### apm spec

**Read or write individual spec sections of a ticket.**

#### Synopsis

    apm spec <id> [--section <name>] [--set <text> | --set-file <path>]
              [--check] [--mark <substring>] [--no-aggressive]

#### Description

A surgical tool for reading or modifying a single named section of a ticket's spec without opening
the whole file. When called with only `--section`, it prints the named section to stdout. Combined
with `--set` or `--set-file`, it overwrites that section's content and commits the change.

`--check` validates that all required sections defined in `apm.toml` are present and non-empty;
exits with status 1 and prints the errors if any are missing.

`--mark` checks off the first unchecked item in `--section` whose text contains the given
substring, and commits the result.

    apm spec 42 --section Problem
    apm spec 42 --section Approach --set "New approach text"
    echo "text" | apm spec 42 --section Approach --set -
    apm spec 42 --section "Acceptance criteria" --mark "output is JSON"

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Ticket ID |
| `--section` | string | — | Section name to read or write (e.g. `Problem`, `Approach`) |
| `--set` | string | — | New content for `--section`; use `-` to read from stdin |
| `--set-file` | path | — | Read new section content from this file (conflicts with `--set`) |
| `--check` | flag | false | Validate that all required sections are non-empty |
| `--mark` | string | — | Check off the first unchecked item in `--section` matching this text |
| `--no-aggressive` | flag | false | Skip implicit git fetch and push |

#### Git internals

| Command | Why |
|---------|-----|
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Find the ticket branch by ID |
| `git fetch origin <branch>` | (aggressive only) Sync before reading or writing |
| `git show <branch>:<path>` | Read the current ticket content |
| `git add <path>` + `git commit -m "ticket(<id>): set section <name>"` | (`--set` / `--set-file`) Write the updated section to the branch |
| `git add <path>` + `git commit -m "ticket(<id>): mark \"<item>\" in <name>"` | (`--mark`) Commit the checked-off item |
| `git push origin <branch>` | (aggressive only, `--set` / `--mark`) Publish the change to the remote |

---

## Workflow orchestration

Commands that drive the agent+human workflow loop.

---

### apm review

**Supervisor: edit a ticket spec and optionally transition state.**

#### Synopsis

    apm review <id> [--to <state>] [--no-aggressive]

#### Description

Opens `$VISUAL` / `$EDITOR` on the ticket file so a supervisor can read the spec, add amendment
requests, or update acceptance criteria. After the editor closes, the command prompts for a state
transition unless `--to` is supplied.

The editor receives a temporary file that contains a comment header listing available transitions,
a sentinel separator, and then the editable spec body. Lines starting with `# ` in the header are
stripped on save. The `## History` section is preserved automatically and is not shown in the editor.

If `--to ammend` is chosen, plain-bullet amendment requests in the spec are automatically converted
to Markdown checkboxes.

    apm review 42 --to specd       # approve spec as-is
    apm review 42 --to ammend      # request changes (add bullets in editor first)
    apm review 42 --to ready       # approve and queue for implementation
    apm review 42 --to implemented # accept implementation

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Ticket ID to review |
| `--to` | string | — | Transition to this state after editing (skips interactive prompt) |
| `--no-aggressive` | flag | false | Skip implicit git fetch and push |

#### Git internals

| Command | Why |
|---------|-----|
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Find the ticket branch |
| `git fetch origin <branch>` | (aggressive only) Pull latest before editing |
| `git show <branch>:<path>` | Read the current ticket content into the editor temp file |
| `git add <path>` + `git commit -m "ticket(<id>): review edit"` | (if spec changed) Commit the edited spec back to the ticket branch |
| `git push origin <branch>` | (aggressive only, if spec changed) Publish the edit |
| *(state transition operations)* | Delegates to `apm state` for the transition step; see `apm state` git internals |

---

### apm start

**Claim a ticket and provision its permanent worktree.**

#### Synopsis

    apm start <id> [--spawn] [-P] [--no-aggressive]
    apm start --next [--spawn] [-P] [--no-aggressive]

#### Description

Claims a ticket by setting its `agent` field to `$APM_AGENT_NAME` and transitioning its state to
`in_progress` (or the state reachable via the `command:start` trigger defined in the workflow
config). It then provisions a permanent git worktree for the ticket branch under the configured
`worktrees.dir` (typically `apm--worktrees/`), merging the default branch into the worktree so it
starts with the latest base.

Prints the worktree path so the caller can `cd` into it or pass it to `git -C`.

`--next` auto-selects the highest-priority actionable ticket, making `apm start --next` equivalent
to `apm next` + `apm start <id>` in a single call.

`--spawn` launches a `claude` subprocess in the worktree that picks up the ticket autonomously.
`-P` (also `--skip-permissions`) passes `--dangerously-skip-permissions` to the spawned `claude`
process.

    apm start 42                   # claim ticket 42
    apm start --next               # claim the next actionable ticket
    apm start --spawn 42           # hand ticket 42 to a background claude agent
    apm start --spawn --next -P    # background agent, skip permissions

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Ticket ID; omit when using `--next` |
| `--next` | flag | false | Auto-select the highest-priority actionable ticket |
| `--spawn` | flag | false | Launch a `claude` worker subprocess in the background |
| `-P` / `--skip-permissions` | flag | false | Pass `--dangerously-skip-permissions` to the worker (requires `--spawn`) |
| `--no-aggressive` | flag | false | Skip implicit git fetch |

#### Git internals

| Command | Why |
|---------|-----|
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Enumerate ticket branches to resolve the ID |
| `git show <branch>:<path>` | Read all ticket data to locate the target ticket |
| `git fetch origin <branch>` | (aggressive only) Sync ticket branch before writing |
| `git fetch origin <default-branch>` | (aggressive only) Sync default branch so the merge in the worktree is up to date |
| `git add <path>` + `git commit -m "ticket(<id>): start — <old> → <new>"` | Write the agent name and new state to the ticket branch |
| `git fetch origin <branch>` | (via `add_worktree`) Fetch the branch locally if no local ref exists |
| `git worktree add <wt-path> <branch>` | Provision the permanent worktree for the ticket branch |
| `git merge <origin/default-branch> --no-edit` | Fast-forward merge default branch into the new worktree so it starts current |

---

### apm sync

**Fetch from remote and reconcile the local ticket cache.**

#### Synopsis

    apm sync [--offline] [--quiet] [--auto-close] [--no-aggressive]

#### Description

The primary bookkeeping command. Fetches all remote refs, detects ticket branches that have been
merged (including squash-merges) into the default branch, and closes those tickets. Run `apm sync`
at the start of each agent session to ensure local state reflects what has happened on the remote.

Without `--offline`, always performs a `git fetch --all` regardless of the `sync.aggressive`
setting.

`--auto-close` closes all merged tickets without prompting; useful in CI pipelines.
`--quiet` suppresses non-error output.

    apm sync                    # fetch and close merged tickets (interactive)
    apm sync --offline          # re-process local branches without fetching
    apm sync --auto-close       # close all merged tickets silently

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--offline` | flag | false | Skip `git fetch`; re-process local branches only |
| `--quiet` | flag | false | Suppress non-error output |
| `--auto-close` | flag | false | Close all merged/stale tickets without prompting |
| `--no-aggressive` | flag | false | Skip implicit push when closing tickets |

#### Git internals

| Command | Why |
|---------|-----|
| `git fetch --all --quiet` | (unless `--offline`) Sync all remote refs including ticket and epic branches |
| `git branch -r --list origin/ticket/*` | List remote ticket branches to detect merges |
| `git branch -r --merged origin/<default>` | Find branches merged into the default branch via a regular merge |
| `git merge-base` + `git diff --shortstat` | Detect squash-merged branches that `--merged` misses |
| `git add <path>` + `git commit -m "ticket(<id>): closed (merged)"` | Close each merged ticket by writing the terminal state to its branch |
| `git push origin <branch>` | (aggressive only) Publish the closure to the remote |

---

### apm work

**Orchestrate workers: dispatch agents until no work remains.**

#### Synopsis

    apm work [-P] [--dry-run] [--daemon] [--interval <seconds>] [--epic <id>]

#### Description

Repeatedly calls `apm start --next --spawn` in a loop, launching one `claude` subprocess per
actionable ticket up to `agents.max_concurrent` (from `.apm/apm.toml`). The loop exits when
`apm next` returns no actionable tickets and all in-flight workers have finished.

`--dry-run` prints the tickets that would be started without spawning any subprocesses.

`--daemon` keeps the process alive after the queue is exhausted, polling at `--interval` seconds
(default 30) and dispatching new workers as slots open or new tickets become actionable. A single
Ctrl-C triggers a graceful drain; a second Ctrl-C forces exit.

`--epic <id>` restricts dispatching to tickets belonging to the specified epic.

    apm work --dry-run           # preview the work queue
    apm work                     # dispatch all actionable tickets
    apm work -P                  # dispatch with skipped permissions
    apm work --daemon            # keep running, poll every 30s

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `-P` / `--skip-permissions` | flag | false | Pass `--dangerously-skip-permissions` to every spawned worker |
| `--dry-run` | flag | false | Preview tickets that would be started without spawning |
| `--daemon` / `-d` | flag | false | Keep running after the queue is exhausted |
| `--interval` | integer | `30` | Poll interval in seconds when running as a daemon |
| `--epic` | string | — | Restrict dispatching to tickets in this epic (8-char hex ID) |

#### Git internals

All git operations are performed by `apm start` for each dispatched ticket. `apm work` itself
makes no direct git calls; it delegates entirely to the `start` command logic.

---

### apm workers

**List and manage running worker processes.**

#### Synopsis

    apm workers
    apm workers --log <id>
    apm workers --kill <id>

#### Description

Without flags, lists all known worker processes by scanning for `.apm-worker.pid` files in ticket
worktrees and checking whether the recorded PID is still alive. Columns: ticket ID, title, PID,
state, and elapsed time. Workers whose process has exited are shown with a `crashed` state if the
ticket is not in a terminal or `worker_end` state.

`--log <id>` tails the `.apm-worker.log` file for the given ticket (last 50 lines, following).

`--kill <id>` sends `SIGTERM` to the worker process for the given ticket.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--log` | string | — | Ticket ID whose worker log to tail |
| `--kill` | string | — | Ticket ID whose worker to terminate |

#### Git internals

| Command | Why |
|---------|-----|
| `git worktree list --porcelain` | Enumerate all registered worktrees to find `.apm-worker.pid` files |

---

## Epics

Commands for managing epics — long-running initiatives that group related tickets on a shared
branch.

---

### apm epic new

**Create a new epic branch.**

#### Synopsis

    apm epic new <title>

#### Description

Creates a new epic branch named `epic/<8-char-id>-<slug>` from the tip of `origin/main` (or the
local `main` if no remote exists). Commits an `EPIC.md` file to the branch and pushes it to the
remote. Prints the full branch name.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<title>` | positional | — | Human-readable title for the epic |

#### Git internals

| Command | Why |
|---------|-----|
| `git fetch origin main` | Sync main so the epic starts from the latest commit |
| `git branch epic/<id>-<slug> origin/main` | Create the epic branch at the remote main tip; falls back to local `main` |
| `git add EPIC.md` + `git commit -m "epic: init"` | Commit the EPIC.md stub to the new branch |
| `git push origin epic/<id>-<slug>` | Publish the epic branch to the remote |

---

### apm epic close

**Open a PR from an epic branch to the default branch.**

#### Synopsis

    apm epic close <id>

#### Description

Checks that all tickets belonging to the epic are in a `satisfies_deps`-flagged or terminal state,
then creates a GitHub pull request from the epic branch to the default branch using the `gh` CLI.
Idempotent: if an open PR already exists for the epic branch, prints its number and returns
successfully without creating a duplicate.

The PR title is derived from the epic branch slug (e.g. `epic/ab12cd34-user-auth` → `User Auth`).

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Epic ID (4–8 char hex prefix) |

#### Git internals

| Command | Why |
|---------|-----|
| `git branch --list epic/*` + `git branch -r --list origin/epic/*` | Resolve the epic ID to a branch name |
| `git branch --list ticket/*` + `git show <branch>:<path>` | Load all tickets to perform the readiness gate check |
| `gh pr list --head <epic-branch> --state open` | Check whether an open PR already exists to avoid duplicates |
| `gh pr create --base <default> --head <epic-branch>` | Open the pull request from the epic branch |

---

### apm epic list

**List all epics with derived state and ticket counts.**

#### Synopsis

    apm epic list

#### Description

Lists all local and remote epic branches with their derived state and a per-state ticket count.
The derived state is computed from the states of all tickets belonging to the epic: if all are
closed the epic is `done`; if any are `in_progress` it is `active`; otherwise `pending`.

#### Options

*No options.*

#### Git internals

| Command | Why |
|---------|-----|
| `git fetch --all --quiet` | (aggressive only) Sync before reading epic and ticket data |
| `git branch --list epic/*` + `git branch -r --list origin/epic/*` | Enumerate all epic branches |
| `git branch --list ticket/*` + `git show <branch>:<path>` | Load all tickets to compute per-epic state and counts |

---

### apm epic show

**Show an epic and its associated tickets.**

#### Synopsis

    apm epic show <id> [--no-aggressive]

#### Description

Prints the epic's title, branch name, derived state, and a table of every ticket that belongs to
it, including each ticket's ID, state, title, and blockers.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | Epic ID (4–8 char hex prefix) |
| `--no-aggressive` | flag | false | Skip implicit git fetch |

#### Git internals

| Command | Why |
|---------|-----|
| `git fetch --all --quiet` | (unless `--no-aggressive`) Sync before reading |
| `git branch --list epic/*` + `git branch -r --list origin/epic/*` | Resolve the epic ID to a branch name |
| `git branch --list ticket/*` + `git show <branch>:<path>` | Load all tickets to find those belonging to the epic |

---

## Repository maintenance

Commands for initializing, auditing, and cleaning up the repository.

---

### apm archive

**Move closed ticket files to the archive directory.**

#### Synopsis

    apm archive [--dry-run] [--older-than <threshold>]

#### Description

Moves terminal-state ticket files from `tickets/` to the configured `archive_dir` on the **default
branch**. Requires `archive_dir` to be set under `[tickets]` in `.apm/config.toml`.

`--dry-run` prints which files would be moved without modifying anything.

`--older-than` accepts a duration string (`30d`) or an ISO date (`2026-01-01`) and skips tickets
whose `updated_at` is more recent than the threshold.

    apm archive                        # archive all closed tickets
    apm archive --dry-run              # preview
    apm archive --older-than 30d       # only tickets inactive for >30 days

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--dry-run` | flag | false | Print which files would be moved without modifying any branches |
| `--older-than` | string | — | Only archive tickets whose `updated_at` is older than this threshold (`30d` or `2026-01-01`) |

#### Git internals

| Command | Why |
|---------|-----|
| `git ls-tree <default-branch> tickets/` | List ticket files present on the default branch |
| `git show <default-branch>:<path>` | Read each ticket file to check state and `updated_at` |
| `git add` + `git commit -m "archive: move closed tickets"` | Atomically move all qualifying files from `tickets/` to `archive_dir/` on the default branch |

---

### apm clean

**Remove worktrees and local/remote branches for closed tickets.**

#### Synopsis

    apm clean [--dry-run] [--branches] [--force] [-y] [--untracked]
              [--remote --older-than <threshold>]

#### Description

Removes permanent worktrees for tickets in terminal states. By default only worktrees are removed;
local and remote branches are never touched without an explicit flag.

`--branches` also deletes local `ticket/*` branches that are already merged into the default
branch.

`--remote --older-than <threshold>` deletes remote `ticket/*` branches whose last commit is older
than the threshold. Requires `--older-than`; each deletion is confirmed interactively unless `--yes`
is also passed.

`--force` bypasses the branch-merged check and prompts for confirmation before each removal.
`--untracked` removes untracked non-temp files from worktrees before removal.

Known temp files (`.apm-worker.pid`, `.apm-worker.log`, `pr-body.md`, `body.md`, `ac.txt`) are
always removed automatically.

    apm clean                              # remove worktrees only
    apm clean --branches                   # worktrees + merged local branches
    apm clean --remote --older-than 30d    # delete old remote branches

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--dry-run` | flag | false | Print what would be removed without modifying anything |
| `-y` / `--yes` | flag | false | Skip per-branch confirmation (used with `--remote`) |
| `--force` | flag | false | Bypass merge check; prompts before each removal |
| `--branches` | flag | false | Also delete local `ticket/*` branches |
| `--remote` | flag | false | Delete remote `ticket/*` branches older than `--older-than` |
| `--older-than` | string | — | Age threshold for `--remote`: e.g. `30d` or `2026-01-01` (requires `--remote`) |
| `--untracked` | flag | false | Remove untracked non-temp files from worktrees before removal |

#### Git internals

| Command | Why |
|---------|-----|
| `git worktree list --porcelain` | Enumerate all registered worktrees to find candidates |
| `git branch --list ticket/*` + `git show <branch>:<path>` | Load tickets to determine which are in terminal states |
| `git branch --merged <default-branch>` | Check whether a ticket branch is merged before deleting it |
| `git rev-parse refs/heads/<branch>` | Verify the local branch exists before attempting deletion |
| `git worktree remove [--force] <path>` | Remove the worktree directory and deregister it from git |
| `git branch -d <branch>` | (`--branches`) Delete the local ticket branch after worktree removal |
| `git for-each-ref refs/remotes/origin/ticket/ --format=%(refname:short) %(creatordate:unix)` | (`--remote`) List remote ticket branches with their last-commit dates |
| `git push origin --delete <branch>` | (`--remote`) Delete a remote ticket branch that meets the age threshold |

---

### apm init

**Initialize apm in the current repository.**

#### Synopsis

    apm init [--no-claude] [--migrate] [--with-docker]

#### Description

Creates the `.apm/` directory and writes the default configuration files:
`config.toml`, `workflow.toml`, `ticket.toml`, `agents.md`, `apm.spec-writer.md`, and
`apm.worker.md`. Also creates the `tickets/` directory, appends apm entries to `.gitignore`, and
creates the `apm--worktrees/` sibling directory.

If the repository has no commits yet, stages the key config files and creates an initial commit.

Unless `--no-claude` is passed, prompts to add apm command patterns to `.claude/settings.json`
(project-level) and `~/.claude/settings.json` (user-level) so that Claude Code's allow list does
not prompt on every `apm` call.

`--migrate` moves a root-level `apm.toml` and `apm.agents.md` into `.apm/`. Mutually exclusive
with the normal setup path.

`--with-docker` additionally writes `.apm/Dockerfile.apm-worker` and prints build instructions.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--no-claude` | flag | false | Skip updating `.claude/settings.json` allow list |
| `--migrate` | flag | false | Move root-level `apm.toml` and `apm.agents.md` to `.apm/` |
| `--with-docker` | flag | false | Generate `.apm/Dockerfile.apm-worker` and print build instructions |

#### Git internals

| Command | Why |
|---------|-----|
| `git ls-files --error-unmatch .claude/settings.json` | Check whether `.claude/settings.json` is tracked; warns if not, so agent worktrees will have the allow list |
| `git add .apm/config.toml .apm/workflow.toml .apm/ticket.toml .gitignore` | (only when the repo has no commits) Stage the core config files |
| `git commit -m "apm: initialize project"` | (only when the repo has no commits) Create the initial commit |

---

### apm validate

**Validate `.apm/` config and cross-ticket integrity.**

#### Synopsis

    apm validate [--fix] [--json] [--config-only] [--no-aggressive]

#### Description

Runs a suite of cross-ticket integrity checks:

- `.apm/` TOML files parse without errors
- All state transitions reference states that exist in the config
- Every ticket's `branch` field matches its actual branch name
- No two tickets share the same branch

`--fix` automatically repairs branch-field mismatches by committing the corrected frontmatter to
the ticket's branch.

`--json` outputs the full results as a structured JSON object — useful for CI pipelines:

    apm validate --json | jq '.errors'

`--config-only` skips per-ticket checks and validates only the config files.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--fix` | flag | false | Auto-fix repairable issues (branch field mismatches) |
| `--json` | flag | false | Output results as JSON |
| `--config-only` | flag | false | Run only config validation; skip per-ticket checks |
| `--no-aggressive` | flag | false | Skip implicit git fetch |

#### Git internals

| Command | Why |
|---------|-----|
| `git fetch --all --quiet` | (aggressive only) Sync all remote refs before reading ticket data |
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Enumerate all ticket branches |
| `git show <branch>:<path>` | Read every ticket file for validation |
| `git add <path>` + `git commit -m "ticket(<id>): fix branch field (validate --fix)"` | (`--fix`) Commit the corrected `branch` frontmatter field to the ticket's branch |

---

### apm verify

**Check ticket and local cache integrity.**

#### Synopsis

    apm verify [--fix] [--no-aggressive]

#### Description

Scans for inconsistencies between the local branch cache and the actual git state: dangling
worktrees, branches missing ticket files, stale cache entries, and tickets in active states whose
branch has already been merged into the default branch.

Prints the configured `completion` strategy for each state transition and the logging path (if
logging is enabled), which is useful for diagnosing unexpected behaviour.

`--fix` closes any in-progress or implemented tickets whose branch has been merged into the default
branch, resolving the most common class of stale-state issues.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--fix` | flag | false | Auto-close tickets in active states whose branch is merged |
| `--no-aggressive` | flag | false | Skip implicit git fetch |

#### Git internals

| Command | Why |
|---------|-----|
| `git fetch --all --quiet` | (aggressive only) Sync all remote refs before checking state |
| `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` | Enumerate all ticket branches |
| `git show <branch>:<path>` | Read every ticket file |
| `git branch -r --merged origin/<default>` + merge-base checks | Detect branches merged into the default branch (including squash-merges) |
| `git add <path>` + `git commit -m "ticket(<id>): closed"` | (`--fix`) Close tickets whose branch has been merged |

---

### apm worktrees

**List or remove permanent git worktrees.**

#### Synopsis

    apm worktrees
    apm worktrees --remove <id>

#### Description

Without flags, lists all known ticket worktrees under the configured `worktrees.dir` directory,
showing the worktree name and the current state of the associated ticket.

`--remove <id>` removes the permanent worktree for the given ticket ID, deregistering it from git.
The ticket branch itself is not deleted; only the working-tree checkout is removed.

    apm worktrees              # list all known worktrees
    apm worktrees --remove 42  # remove the worktree for ticket 42

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `--remove` | string | — | Ticket ID whose worktree to remove |

#### Git internals

| Command | Why |
|---------|-----|
| `git worktree list --porcelain` | Enumerate all registered worktrees to find ticket worktrees |
| `git branch --list ticket/*` + `git show <branch>:<path>` | (`--remove`) Load ticket data to resolve the ID and find the branch |
| `git worktree remove <path>` | (`--remove`) Deregister and delete the worktree directory |

---

## Server & agent management (requires apm-server)

Commands that communicate with a running `apm-server` instance. The server URL is read from
`server.url` in `.apm/apm.toml`. All HTTP calls require a valid session; authenticate first with
`apm register`.

---

### apm agents

**Print agent instructions from `.apm/agents.md`.**

#### Synopsis

    apm agents

#### Description

Reads the instructions file configured under `[agents] instructions` in `.apm/apm.toml` and prints
its contents to stdout. Use this to inject the project's agent onboarding instructions into a
new agent subprocess's context without requiring file-system access to the repository.

    apm agents | pbcopy          # copy to clipboard
    apm agents > /tmp/agents.md  # write to a temp file for injection

#### Options

*No options.*

#### Git internals

No git operations.

---

### apm register

**Generate a one-time password for device registration.**

#### Synopsis

    apm register [<username>]

#### Description

Calls `POST /api/auth/otp` on the configured `apm-server` with the given username (defaulting to
the GitHub username detected from `git_host` config), and prints the returned one-time password.
The OTP is used to complete device registration via the apm-server web UI or API.

If the username is inferred automatically, it is printed to stderr before the OTP so the operator
can confirm which account is being registered.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<username>` | positional | GitHub username from `git_host` config | Username to register |

#### Git internals

No git operations.

---

### apm revoke

**Revoke sessions (requires apm-server).**

#### Synopsis

    apm revoke <username> [--device <hint>]
    apm revoke --all

#### Description

Calls `DELETE /api/auth/sessions` on the configured `apm-server` to revoke active sessions.
Requires either a username or `--all`. Optionally filters by device hint when revoking a single
user's sessions.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<username>` | positional | — | Username whose sessions to revoke (required unless `--all`) |
| `--device` | string | — | Only revoke sessions matching this device hint |
| `--all` | flag | false | Revoke all sessions for all users (conflicts with `--device`) |

#### Git internals

No git operations.

---

### apm sessions

**List active sessions (requires apm-server).**

#### Synopsis

    apm sessions

#### Description

Calls `GET /api/auth/sessions` on the configured `apm-server` and prints a formatted table of
active sessions with columns: `USERNAME`, `DEVICE`, `LAST SEEN`, and `EXPIRES`.

#### Options

*No options.*

#### Git internals

No git operations.

---

## Internal commands

Commands used by apm's own infrastructure rather than directly by humans or agents. They are hidden
from `apm --help`.

---

### apm _hook

**Internal git hook dispatcher.**

#### Synopsis

    apm _hook <hook-name> [<extra-args>...]

#### Description

Dispatched by git hook scripts installed in `.git/hooks/`. Receives the hook name as the first
argument and any extra arguments git passes (e.g. remote URL for `pre-push`). Currently the only
handled hook is `pre-push`; all others are silently ignored.

The `pre-push` handler is currently a no-op: automatic state transitions on push have been removed
in favour of explicit `apm state` calls.

This command is invoked by git, not by users or agents. Do not call it directly.

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<hook-name>` | positional | — | Name of the git hook (e.g. `pre-push`) |
| `<extra-args>` | trailing | — | Extra arguments passed by git; currently ignored |

#### Git internals

No git operations.
