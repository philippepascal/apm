# APM — Command Reference and Mechanics

> Defines what each CLI command does: its purpose, the git operations it
> runs, the file operations it performs, and how it interacts with git
> worktrees. This is the authoritative reference for implementors.
>
> **Worktree** throughout this document means a git worktree — a directory
> checked out to a specific branch via `git worktree add`. Each ticket in
> `in_progress` has one permanent worktree. This is the cornerstone of
> multi-agent parallelism: agents work in separate directories and never
> touch each other's branches or the main worktree.

---

## Conventions

### Worktree layout

```
~/projects/
  myrepo/                          ← main worktree, always on main
  myrepo--worktrees/               ← sibling dir; path in apm.toml [worktrees] dir
    ticket-0018-apm-init-config/   ← agent 1's permanent worktree
    ticket-0019-apm-list-closed/   ← agent 2's permanent worktree
```

The worktrees directory is configured in `apm.toml`:

```toml
[worktrees]
dir = "../myrepo--worktrees"   # relative to repo root
```

`apm init` sets a default of `../<repo-name>--worktrees`.

### Ticket branch naming

```
ticket/<id:04>-<slug>
```

This name is used for the entire ticket lifecycle, from creation through
close. It never changes.

### Reading tickets

All commands that read ticket data do so by calling `git show <branch>:<rel-path>`
against the local branch ref (falling back to `origin/<branch>` when the
local ref is absent). No filesystem cache is involved. No `apm sync` is
required before reading.

### Writing tickets (commits)

Commands that update a ticket (state change, field set, etc.) commit
directly to the ticket's branch:

- **If a permanent worktree exists** for that branch: write the file in the
  worktree, run `git add` + `git commit` there. Fast, no temp directory.
- **If no permanent worktree exists** (ticket not yet started): create a
  temporary worktree in the system temp directory, commit, remove it
  immediately. The caller's working directory is never disturbed.

This logic lives in `git::commit_to_branch`.

---

## `apm init`

**Purpose:** First-time APM setup in a git repository.

**Git operations:**
- `git rev-parse HEAD` — detect whether the repo has any commits
- `git add apm.toml .gitignore` + `git commit` — initial commit if no commits exist
- `git rev-parse --verify refs/heads/apm/meta` — check whether the meta branch exists
- Temp worktree → `git rm -rf .` + write `NEXT_ID=1` + `git commit` + remove worktree — create `apm/meta` branch

**File operations:**
- Create `tickets/` directory (holds `NEXT_ID` fallback only)
- Write `apm.toml` (default config, including `[worktrees] dir`)
- Write `apm.agents.md` (default agent instructions)
- Ensure `CLAUDE.md` imports `@apm.agents.md`
- Append `tickets/NEXT_ID` to `.gitignore`
- Write `.git/hooks/pre-push` (executable)
- Write `.git/hooks/post-merge` (executable)
- Optionally: update `.claude/settings.json` with `apm` allow-list entries

**Worktree effect:** Creates the worktrees directory (`[worktrees] dir`) if
it does not exist.

**No existing state required.** Safe to run on a fresh repo or an existing
repo with commits.

---

## `apm new`

**Purpose:** Create a new ticket: allocate an ID, commit the initial ticket
file to a new branch, and print the result.

**Git operations:**
- `git fetch origin apm/meta` — refresh the ID counter
- Temp worktree on `apm/meta` → read `NEXT_ID`, write `NEXT_ID+1`, `git commit`, `git push origin apm/meta`, remove worktree — allocate ticket ID (optimistic-lock retry up to 5×)
- Temp worktree → `git worktree add -b ticket/<id>-<slug> <path> HEAD`, write ticket file, `git add`, `git commit`, remove worktree — commit ticket to new branch

**File operations:** None to the working tree. The ticket file is written
only inside the temporary worktree, which is removed after commit.

**Worktree effect:** None. A permanent worktree is not created at `apm new`
time. It is created later by `apm start`.

**Output:** `Created ticket #<id>: <filename> (branch: ticket/<id>-<slug>)`

---

## `apm sync`

**Purpose:** Fetch remote updates and fire auto-transitions for merged branches.

**Git operations:**
- `git fetch --all` — update remote refs (skipped with `--offline`)
- `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` — enumerate branches
- `git show <branch>:<rel-path>` per merged branch — read ticket content for auto-transition
- `git branch --merged main` (local and/or remote) — detect merged branches
- `git commit_to_branch` (temp worktree on `main`) — commit `implemented → accepted` for each merged branch
- `git push origin <branch>` per local branch with unpushed commits — push (skipped with `--offline`)

**File operations:** None to the working tree.

**Worktree effect:** None. `apm sync` does not create or remove worktrees.

**Output:** `sync: N ticket branches visible, M auto-transitioned`

---

## `apm list`

**Purpose:** Show tickets, filtered by state and agent.

**Flags:** `--all` (include terminal states), `--state <s>`, `--unassigned`
(only tickets with no agent), `--supervisor <name>`, `--actionable <actor>`
(only tickets where the state's `actionable` list includes `<actor>` or `"any"`;
values: `agent`, `supervisor`, `engineer`).

**Git operations:**
- `git branch --list ticket/*` + `git branch -r --list origin/ticket/*` — enumerate branches
- `git show <branch>:<rel-path>` per branch — read all ticket files

**File operations:** None.

**Worktree effect:** None. For `in_progress` tickets, the output includes
the worktree path if one exists:

```
#18  [in_progress] apm init default config  agent=claude-0326  wt=../myrepo--worktrees/ticket-0018-...
```

---

## `apm show`

**Purpose:** Print the full content of one ticket.

**Git operations:**
- `git branch --list ticket/*` + remote — find the branch whose name starts with `ticket/<id:04>-`
- `git show origin/<branch>:<rel-path>` (fallback: `git show <branch>:<rel-path>`) — read content

**File operations:** None.

**Worktree effect:** None. If a worktree exists for the ticket, the command
reads from the branch ref, not the worktree filesystem (to guarantee
consistency with what is committed).

---

## `apm next`

**Purpose:** Find and print the highest-priority ticket that has no assigned
agent and is in a state where `actionable` includes `"agent"`. Scores
candidates using `priority_weight`, `effort_weight`, and `risk_weight`.

**Flags:** `--json` (output as JSON object or `null`).

**Git operations:** Same as `apm list` (enumerates and reads all branches).

**File operations:** None.

**Worktree effect:** None.

---

## `apm state`

**Purpose:** Transition a ticket to a new state, enforcing transition rules.
Appends a history row and updates `state` and `updated_at` in frontmatter.

**Git operations:**
- Enumerate branches + read all tickets (same as `apm list`)
- `git commit_to_branch` on the ticket's branch — commit updated ticket file
  - Uses permanent worktree if one exists, else temp worktree

**File operations:** If a permanent worktree exists for this ticket, writes
the updated ticket file directly into the worktree before committing.
Otherwise no writes to the working tree.

**Worktree effect:** None (does not create or remove worktrees). For closed
tickets the commit targets `main`, not the ticket branch.

**Transition enforcement:**
- Terminal states (`terminal = true` in config) are always reachable
- Other targets are checked against the `transitions` list of the current state
- If transitions list is empty, all non-terminal targets are allowed

---

## `apm set`

**Purpose:** Update a single frontmatter field (`priority`, `effort`, `risk`,
`title`, `supervisor`, `agent`, `branch`).

**Git operations:**
- Enumerate branches + read all tickets
- `git commit_to_branch` on the ticket's branch (permanent worktree if exists, else temp)

**File operations:** Same as `apm state`.

**Worktree effect:** None.

---

## `apm start`

**Purpose:** Claim a ticket for the current agent: transition `→ in_progress`,
set `agent` and confirm `branch` in frontmatter, and provision a permanent
git worktree for the ticket branch.

**Preconditions:** Ticket must be in a state whose `actionable` list includes
`"agent"` (by default: `new`, `ammend`, `ready`) and have no current agent.

**Git operations:**
1. Enumerate branches + read all tickets
2. `git commit_to_branch` on ticket branch — commit `→ in_progress` + agent/branch fields
3. `git fetch origin <branch>` — ensure branch is present locally (if not already)
4. `git worktree add <worktrees-dir>/ticket-<id>-<slug> <branch>` — create permanent worktree

**File operations:**
- Creates the worktrees directory if it does not exist
- The worktree directory is a full checkout of the ticket branch

**Worktree effect:** Creates `<worktrees-dir>/ticket-<id>-<slug>/` and
checks out the ticket branch there. The main worktree is not touched.
No `git checkout` in the main working directory.

**Output:**
```
#18: ready → in_progress (agent: claude-0326-a3f9, branch: ticket/0018-apm-init-...)
Worktree: /Users/philippe/projects/myrepo--worktrees/ticket-0018-apm-init-...
```

The agent `cd`s into the worktree and works there.

---

## `apm take`

**Purpose:** Take over an `in_progress` or `implemented` ticket from another
agent. Updates the `agent` field and provisions (or reuses) a permanent
worktree.

**Git operations:**
1. Enumerate branches + read all tickets
2. `git commit_to_branch` on ticket branch — commit handoff record (agent change)
3. `git fetch origin <branch>` if branch not local
4. `git worktree list` — check whether a worktree for this branch already exists
5. If no worktree: `git worktree add <path> <branch>` — create worktree
6. If worktree exists: reuse it (no-op at the git level)

**File operations:** Creates worktree directory if provisioning.

**Worktree effect:** Ensures a permanent worktree exists. Prints the path.
The main worktree is not touched.

---

## `apm worktrees`

**Purpose:** List all provisioned permanent worktrees and their ticket status.

**Git operations:**
- `git worktree list --porcelain` — enumerate all worktrees
- `git show <branch>:<rel-path>` per worktree — read ticket state

**File operations:** None.

**Output:**
```
ticket-0018-apm-init-default-config   in_progress  agent=claude-0326
ticket-0019-apm-list-shows-closed     in_progress  agent=claude-0401
```

**Subcommand: `apm worktrees remove <id>`**

Removes the permanent worktree for ticket `<id>`:
- `git worktree remove <path>` (fails if there are uncommitted changes)
- Removes the directory

Agents run this after their ticket is closed and the worktree is no longer
needed.

---

## `apm _hook pre-push`

**Purpose:** Called by the `pre-push` git hook. Fires `ready → in_progress`
on the first push of a `ticket/<id>-*` branch.

**Git operations:**
- Enumerate branches + read all tickets (via git blobs)
- `git commit_to_branch` on the ticket's branch (temp worktree, since this
  fires before `apm start` provisions the permanent worktree)

**File operations:** None.

**Worktree effect:** None.

---

## `apm verify`

**Purpose:** Check ticket consistency: valid states, matching filenames,
branch presence for in-progress tickets, merged-but-not-accepted tickets,
and spec completeness.

**Git operations:**
- Enumerate branches + read all tickets
- `git branch --merged main` — detect merged branches
- With `--fix`: `git commit_to_branch` on `main` for each auto-fixable issue

**File operations:** None.

**Worktree effect:** None.

---

## `apm agents`

**Purpose:** Print the contents of `apm.agents.md` for the agent to read
at session start.

**Git operations:** None (reads file from working tree).

**File operations:** Read `apm.agents.md`.

**Worktree effect:** None.

---

## Summary table

| Command | Reads tickets | Writes ticket | Git checkout | Creates worktree | Removes worktree |
|---------|--------------|--------------|--------------|-----------------|-----------------|
| `init` | — | — | — | creates wt dir | — |
| `new` | — | temp wt | — | — | — |
| `sync` | branch blobs | temp wt (main) | — | — | — |
| `list` | branch blobs | — | — | — | — |
| `show` | branch blob | — | — | — | — |
| `next` | branch blobs | — | — | — | — |
| `state` | branch blobs | perm wt or temp | — | — | — |
| `set` | branch blobs | perm wt or temp | — | — | — |
| `start` | branch blobs | perm wt | — | **yes** | — |
| `take` | branch blobs | perm wt | — | if needed | — |
| `worktrees` | branch blobs | — | — | — | subcommand |
| `worktrees remove` | — | — | — | — | **yes** |
| `_hook pre-push` | branch blobs | temp wt | — | — | — |
| `verify` | branch blobs | temp wt (--fix) | — | — | — |

`git checkout` in the main working directory: **never used by any command**.

---

## `apm/meta` branch and ID allocation

The `apm/meta` branch contains a single file `NEXT_ID`. `apm new` uses an
optimistic-lock protocol:

```
1. git fetch origin apm/meta
2. Read NEXT_ID from branch (default 1 if branch absent)
3. Claim the current value as the new ticket's ID
4. Write NEXT_ID+1 to the branch via temp worktree + commit
5. git push origin apm/meta
6. If push is rejected (concurrent allocation): fetch, re-read, retry (max 5×)
```

In pure-git mode (no remote), the push is skipped. The local commit is
sufficient since there are no concurrent writers.

The `tickets/NEXT_ID` file in the working tree is a fallback used only when
the repo has no commits yet (git worktree operations require at least one
commit). Once an initial commit exists, `apm/meta` is the canonical source.

---

## Post-merge state on `main`

When a ticket branch is merged to `main` via PR, the ticket file arrives on
`main` as a tracked, committed file (part of the merge commit). `apm sync`
detects the merged branch and fires `implemented → accepted`, committing
only the frontmatter update to `main`. This is the only APM-originated
commit that goes directly to `main`.

After the ticket reaches `closed` (`apm state N closed`), the ticket branch
may be deleted. The ticket file remains on `main` permanently.

```
apm worktrees remove <id>                           # remove local worktree
git push origin --delete ticket/<id>-<slug>         # delete remote branch
```

`apm sync` will not remove the ticket file from `main` — it only reads and
auto-transitions; it never deletes tracked files.
