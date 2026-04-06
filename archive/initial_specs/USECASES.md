# APM — Use Cases

> Concrete walkthroughs of APM at the file level, git level, and user experience level.
> Used to validate and revise SPEC-V2.md.

---

## Cast

- **Philippe** — engineer; creates tickets, supervises, reviews PRs
- **Alice** — engineer; picks up some tickets
- **claude-main** — AI agent; works the implementation queue
- Repo: `philippepascal/ticker` — a Rust financial ticker app

---

## Use Case 1: Setting up APM on a new repo

### Situation

Philippe has just created a new GitHub repo, `philippepascal/ticker`. The repo has one commit: the GitHub default README. He wants APM to manage the project.

### What Philippe does

```bash
git clone git@github.com:philippepascal/ticker.git
cd ticker
apm init
```

`apm init` prompts for a project name and optional description, then runs.

### What happens at the file level

```
ticker/
├── .git/
├── README.md              ← already existed
├── apm.toml               ← created by apm init
├── apm.agents.md          ← created by apm init
└── tickets/
    └── NEXT_ID            ← created by apm init; content: "1"
```

**`apm.toml`** — generated with defaults, opened in `$EDITOR` for review:

```toml
[project]
name = "ticker"
description = ""

[tickets]
dir = "tickets"
layer_boundary = "in_progress"

# Uncomment and configure to connect a git provider:
# [provider]
# type = "github"
# token_env = "APM_GITHUB_TOKEN"
# webhook_secret_env = "APM_WEBHOOK_SECRET"

[workflow]
terminal_states = ["closed"]

[[workflow.states]]
id = "new"
label = "New"
color = "#6b7280"
transitions_to = ["question", "specd"]

# ... (full default state machine)
```

**`apm.agents.md`** — generated with default agent instructions, opened in `$EDITOR` after `apm.toml`:

```markdown
# APM Agent Instructions

## Identity
Set `APM_AGENT_NAME` before running any apm command.
Convention: `claude-main` for the primary agent.
Engineers set `APM_AGENT_NAME` to their own name.

## Startup
...
```

**`tickets/NEXT_ID`** — plain text file:

```
1
```

### What happens at the git level

`apm init` stages and commits all three new items:

```
commit abc1234
Author: Philippe Pascal <philippe@example.com>
Date:   2026-03-25

    apm: initialize project management

 apm.toml                | 45 +++++++++++++++
 apm.agents.md           | 30 +++++++++++
 tickets/NEXT_ID         |  1 +
```

Then pushes to origin/main:

```bash
git push -u origin main
```

### Git hooks (local only — not committed)

```bash
apm init --hooks
```

This writes three files inside `.git/hooks/` of the local clone:

```
.git/hooks/pre-commit     → apm verify --fast
.git/hooks/pre-push       → apm _hook pre-push "$@"
.git/hooks/post-merge     → apm sync --quiet
.git/hooks/post-checkout  → apm sync --quiet --offline
```

**These hooks are not tracked by git.** Every engineer or agent that clones the repo must run `apm init --hooks` to get local automation. This is by design — `.git/` is never committed.

> **Implication for teams:** onboarding docs (or the repo README) should include `apm init --hooks` as a setup step. `apm.agents.md` should also mention it in the Startup section.

### What Philippe sees

After init, `apm status` returns:

```
ticker  (0 open tickets)
```

`apm serve` opens `http://localhost:7070` — an empty board with seven columns (NEW · QUESTION · SPECD · AMMEND · READY · IN PROGRESS · IMPLEMENTED · ACCEPTED).

### .gitignore considerations

`apm init` does **not** modify `.gitignore`. Nothing APM writes to the ticket repo should be ignored:

- `tickets/` — the database; always committed
- `apm.toml` — config; always committed
- `apm.agents.md` — agent instructions; always committed
- `tickets/NEXT_ID` — must be committed for ID coordination

The user-global cache (`~/.apm/apm.db`) lives outside the repo entirely; no `.gitignore` entry needed.

If the ticket repo is also the code repo (common for solo projects), the engineer's existing `.gitignore` applies to code. APM adds nothing to it.

> **One edge case:** if `apm serve` writes a socket file or PID file for the local server, that should be in `.gitignore`. The spec should clarify where any process-local files land. Suggested: `~/.apm/<repo-hash>.pid` (outside the repo), not in the tree.

### Agent setup (first time)

An agent connecting to this repo for the first time:

```bash
# In the agent's environment:
export APM_REPO=/path/to/ticker        # or the agent passes it per-command
export APM_AGENT_NAME=claude-main
export APM_GITHUB_TOKEN=ghp_...        # if GitHub integration is configured

git clone git@github.com:philippepascal/ticker.git ~/repos/ticker
cd ~/repos/ticker
apm init --hooks    # local hooks only
apm sync            # builds ~/.apm/apm.db entry for this repo
apm agents          # prints apm.agents.md — agent reads this
apm status          # confirms connection
```

The agent does not need to run `apm init` (only `apm init --hooks`). `apm init` is for first-time project setup by a human.

---

## Use Case 2: Setting up APM on an existing repo

### Situation

Philippe has been developing `ticker` for six months. It has 40 commits, `src/`, `tests/`, a `Cargo.toml`, and an existing CI workflow. He wants to add APM without disturbing the existing history or structure.

### Two topology options

**Option A — Tickets in the same repo (recommended for solo / small teams)**

The ticket files live alongside the code. One clone, one repo to think about.

```
ticker/
├── src/
├── tests/
├── Cargo.toml
├── .github/
├── apm.toml            ← new
├── apm.agents.md       ← new
└── tickets/
    └── NEXT_ID         ← new
```

```bash
cd ticker
apm init
apm init --hooks
git push
```

No branch naming conflicts. Feature branches (`feature/1-add-csv-export`) sit alongside existing branches in the same repo. The branch name prefix `feature/` is conventional; agents must use it. If the existing repo uses `feature/` branches for other purposes, the pattern `feature/<id>-*` is distinct enough not to collide.

**Option B — Dedicated tickets repo**

Tickets live in a separate repo (`philippepascal/ticker-tickets`). The code repo is untouched. APM is initialized in the tickets repo; the `apm.toml` there lists the code repo(s) as watched repos.

```
ticker-tickets/
├── apm.toml         # [[repos.code]] points to org/ticker
├── apm.agents.md
└── tickets/
    └── NEXT_ID

ticker/              # untouched — no APM files
├── src/
└── ...
```

```bash
mkdir ticker-tickets && cd ticker-tickets
git init
git remote add origin git@github.com:philippepascal/ticker-tickets.git
apm init
# In apm.toml, add:
# [[repos.code]]
# path = "philippepascal/ticker"
# default_branch = "main"
```

Trade-off: the code repo's branches are still where implementation happens (`feature/1-add-csv-export` in `philippepascal/ticker`). APM watches that repo via webhooks or polling. Ticket files and code are decoupled — the code repo stays clean.

> **Spec gap:** §13 describes multi-repo setup but doesn't explicitly address the tickets-in-separate-repo topology. The `apm.toml` in the tickets repo references code repos, but `apm start` needs to create branches in the _code_ repos. How does the agent know which local path corresponds to `org/ticker`? The spec needs a local path mapping, e.g.:
> ```toml
> [[repos.code]]
> path = "philippepascal/ticker"       # remote identifier
> local = "/Users/philippe/repos/ticker"  # local clone path
> default_branch = "main"
> ```

### Migrating existing issues

If Philippe has been tracking work in GitHub Issues, Notion, or a text file, he can create APM tickets manually:

```bash
apm new "Fix memory leak in tick processor"
apm new "Add WebSocket live feed"
apm new "Write integration tests for CSV export"
```

Each creates a ticket in `new` state. No migration tooling in V1 — it's a manual one-time import.

### What changes vs. a new repo

Everything else is identical. `apm init` on an existing repo:
- Does not touch `src/`, `Cargo.toml`, `.github/`, or any existing files
- Appends to an existing `.gitignore` only if the user confirms (prompt)
- Existing git history is undisturbed

---

## Use Case 3: Daily task management

### Setup state

- Repo: `philippepascal/ticker` (same repo as code; Option A)
- GitHub integration configured: `[provider] type = "github"` in `apm.toml`
- Hooks installed locally for Philippe and claude-main's clones
- Board is empty; `tickets/NEXT_ID` contains `1`

---

### Act 1: Philippe creates a ticket

```bash
export APM_AGENT_NAME=philippe
apm new "Add CSV export for portfolio data"
```

APM opens `$EDITOR` with a prefilled template:

```
tickets/0001-add-csv-export-for-portfolio-data.md
```

```
+++
id = 1
title = "Add CSV export for portfolio data"
state = "new"
effort = ""
risk = ""
priority = 0
created_at = "2026-03-25T10:00:00Z"
updated_at = "2026-03-25T10:00:00Z"
author = "philippe"
supervisor = "philippe"
agent = ""
branch = ""
repos = []
+++

## Spec

### Problem
(describe what is broken or missing)

### Acceptance criteria
- [ ]

### Out of scope

### Approach
```

Philippe fills in:

```
### Problem
Users cannot download their portfolio history as CSV. They must manually copy
values from the table view. The ticker app has all the data; it just needs an
export endpoint.

### Acceptance criteria
- [ ] GET /portfolio/export returns CSV with correct headers
- [ ] CSV includes all transactions in date range (default: all time)
- [ ] Filename is portfolio_YYYY-MM-DD.csv
- [ ] Empty portfolio returns valid CSV with headers only
```

On save and exit:

**File-level:** Two files written to disk:
- `tickets/NEXT_ID` → contents now `2`
- `tickets/0001-add-csv-export-for-portfolio-data.md` → as above

**Git-level:** APM commits both to `main`:

```
commit d4f89a2
Author: philippe <philippe@example.com>

    ticket(1): create "Add CSV export for portfolio data"
```

**SQLite cache:** `~/.apm/apm.db` updated — ticket #1 inserted into `tickets` table with `state = "new"`.

**Board:** NEW column shows one card:

```
┌─────────────────────────────┐
│ #1  Add CSV export for      │
│     portfolio data          │
│                      new 0d │
└─────────────────────────────┘
```

---

### Act 2: Agent picks up the ticket and asks a question

On `claude-main`'s machine, at the start of a session:

```bash
export APM_AGENT_NAME=claude-main
export APM_REPO=/home/agent/repos/ticker
apm sync              # pulls latest from origin/main
apm agents            # reads apm.agents.md
apm list --state new  # sees ticket #1
```

Output:

```
#1  Add CSV export for portfolio data   new  philippe  (unassigned)
```

Agent reads the ticket:

```bash
apm show 1
```

Agent has a question about the data model:

```bash
apm state 1 question
apm ask 1 "Should the CSV include unrealized gains (open positions) or only \
realized transactions? The current data model has both in separate tables."
```

**`apm state 1 question`** — what happens:

*File-level:* APM updates the frontmatter in `tickets/0001-add-csv-export-for-portfolio-data.md` on main:

```toml
state = "question"
updated_at = "2026-03-25T11:15:00Z"
```

Only these two lines in the frontmatter change. The body (Spec section) is unchanged.

*Git-level:*

```
commit 7c2e1f4
Author: claude-main <agent@example.com>

    ticket(1): state new → question
```

**`apm ask 1 "..."`** — what happens:

*File-level:* APM appends to the Conversation section of the ticket file on `main` (we are still in Layer 1 — body changes also go to main before `in_progress`):

```markdown
## Conversation

**2026-03-25T11:16Z claude-main:** Should the CSV include unrealized gains (open
positions) or only realized transactions? The current data model has both in
separate tables.
```

*Git-level:*

```
commit 3a8b5c1
Author: claude-main <agent@example.com>

    ticket(1): question from claude-main
```

**Board:** Card moves to QUESTION column. Philippe sees it highlighted (needs action).

---

### Act 3: Philippe answers the question

Philippe opens the board in `apm serve` or reads it in the terminal:

```bash
apm list --needs-action --supervising
```

```
#1  Add CSV export for portfolio data   question  (waiting for you)
```

```bash
apm show 1 --conv
```

```
## Conversation

2026-03-25T11:16Z claude-main: Should the CSV include unrealized gains (open
positions) or only realized transactions? The current data model has both in
separate tables.
```

Philippe replies:

```bash
apm reply 1 "Include both. Add a column 'unrealized_gain_pct' from the \
positions table. Date range filter applies to transactions; open positions \
are always included."
```

*File-level:* Conversation section appended on `main`:

```markdown
**2026-03-25T14:30Z philippe:** Include both. Add a column 'unrealized_gain_pct'
from the positions table. Date range filter applies to transactions; open
positions are always included.
```

*Git-level:*

```
commit 9d3f7e2
Author: philippe <philippe@example.com>

    ticket(1): reply from philippe
```

Philippe moves the ticket back to `new` (or the agent checks for replies and continues — the spec should clarify whether the question state is manual-exit or auto-exits on reply):

```bash
apm state 1 new
```

> **Spec gap:** The workflow doesn't define who exits the `question` state and how. Options: (a) supervisor manually moves back to `new` after replying; (b) `apm reply` automatically transitions to `new`; (c) the state stays `question` until the agent acknowledges. The state machine should specify a recommended convention and `apm.agents.md` should instruct agents on how to handle it.

---

### Act 4: Agent writes the spec

```bash
apm sync                # agent pulls the reply
apm show 1 --conv       # reads the answer
apm spec 1              # opens spec section in $EDITOR
```

Agent writes the full spec (problem, acceptance criteria, out of scope, approach) into the Spec section. On save, APM commits the body change to `main`:

```
commit 2f1a9b3
Author: claude-main <agent@example.com>

    ticket(1): update spec
```

Agent marks spec complete:

```bash
apm state 1 specd
```

```
commit 5e7c4d1
Author: claude-main <agent@example.com>

    ticket(1): state new → specd
```

**Board:** Card in SPECD column. Philippe's "needs action" list updates.

---

### Act 5: Philippe approves the spec

```bash
apm show 1 --spec       # reads the spec
apm state 1 ready       # approves
```

```
commit 8b2d6f9
Author: philippe <philippe@example.com>

    ticket(1): state specd → ready
```

**Board:** Card moves to READY.

---

### Act 6: Agent starts implementation

```bash
apm start 1
```

`apm start` does the following atomically:

1. **Creates the branch locally:**
   ```bash
   git checkout -b feature/1-add-csv-export-for-portfolio-data
   ```

2. **Updates frontmatter on `main`** (before pushing):
   ```toml
   state = "in_progress"
   agent = "claude-main"
   branch = "feature/1-add-csv-export-for-portfolio-data"
   updated_at = "2026-03-25T16:00:00Z"
   ```
   Committed to `main`:
   ```
   commit c3a7f12
   Author: claude-main <agent@example.com>

       ticket(1): state ready → in_progress [claude-main]
   ```
   Branch pointer changes: APM is now on the feature branch.

3. **Pushes the branch:**
   ```bash
   git push -u origin feature/1-add-csv-export-for-portfolio-data
   ```

4. **Pre-push hook fires** (if GitHub integration not configured, or as belt-and-suspenders):
   `apm _hook pre-push` detects this is the first push of a `feature/1-*` branch.
   If the ticket isn't already `in_progress`, it fires `ready → in_progress`.
   (In this case the transition already happened in step 2.)

5. **With GitHub integration:** the push webhook fires. APM's webhook handler receives it, confirms ticket #1 is `in_progress`, records the push timestamp for the activity indicator.

**File-level after `apm start`:**

The ticket file exists in two places now:

- `main:tickets/0001-add-csv-export-for-portfolio-data.md` — frontmatter at `in_progress` + frozen body (the spec as of when `apm start` ran)
- `feature/1-...:tickets/0001-add-csv-export-for-portfolio-data.md` — identical content (branch was created from this commit)

From this point: the agent only modifies the ticket body on the feature branch. All frontmatter changes go to `main` via `apm state` / `apm set`.

**Board:** Card moves to IN PROGRESS. Agent name `claude-main` appears on the card.

---

### Act 7: Agent implements — commits to the feature branch

Agent writes code. Normal git workflow:

```bash
# On feature/1-add-csv-export-for-portfolio-data
echo "mod csv_export;" >> src/lib.rs
# ... implements endpoint ...
git add src/csv_export.rs src/routes.rs tests/csv_export_tests.rs
git commit -m "feat: implement CSV export endpoint"
git push
```

The agent also updates the ticket's Conversation and History sections as work progresses. These are body changes — committed to the feature branch:

```bash
apm ask 1 "The positions table uses DECIMAL(18,6) but CSV doesn't preserve \
precision. Should I output 6 decimal places always, or use the display \
precision from user settings?"
```

*File-level:* Conversation section appended in the ticket file on the feature branch. This commit is on the branch, not main.

*Git-level:*
```
commit a1b2c3d  (on feature/1-...)
Author: claude-main <agent@example.com>

    ticket(1): question from claude-main
```

Philippe answers (he's on `main`; APM commits his reply to `main`):

```bash
apm reply 1 "Use 6 decimal places. Don't depend on user settings in the export."
```

*Git-level:*
```
commit d4e5f6a  (on main)
Author: philippe <philippe@example.com>

    ticket(1): reply from philippe
```

> **Spec gap — Body divergence:** At this point, the ticket file body has diverged:
> - `main` has the reply in the Conversation section
> - `feature/1-...` does NOT have the reply yet (it was committed to main)
>
> This is the key tension in the two-layer model: body changes from the _supervisor_ (on main) and body changes from the _agent_ (on branch) can interleave. When the branch merges, git must reconcile both sets of body changes.
>
> **Proposed resolution:** `apm reply` commits replies to both `main` AND the feature branch (cherry-pick or dual commit). This keeps the branch's conversation section in sync. Alternatively, the merge reconciliation step (see Act 9) handles it. The spec needs to define this explicitly.

**Activity indicator:** Each push to the feature branch updates `last_active_at` in the SQLite cache. The board shows a `●` dot on the IN PROGRESS card.

---

### Act 8: Agent opens a PR

Agent finishes implementation and verifies the acceptance criteria. Opens a PR on GitHub:

```bash
gh pr create \
  --title "Add CSV export for portfolio data" \
  --body "Closes #ticket/1

## Summary
- GET /portfolio/export returns CSV
- Includes both realized transactions and open positions
- Date range filter via ?from=&to= query params

## Test plan
- Unit tests: src/csv_export.rs
- Integration test: tests/csv_export_tests.rs" \
  --base main \
  --head feature/1-add-csv-export-for-portfolio-data
```

> **Note on PR body format:** APM links PRs to tickets via branch name detection (`feature/1-*`). The `Closes #ticket/1` in the PR body is optional documentation — it's not a GitHub Issues link. APM does not use GitHub Issues. The branch name is the primary detection mechanism.

**With GitHub integration:**

GitHub sends a `pull_request.opened` webhook to APM's receiver (via `apm serve --webhooks` or a relay).

APM webhook handler:
1. Detects branch `feature/1-add-csv-export-for-portfolio-data` → ticket #1
2. Creates `ticket_prs` record: `{ pr_number: 7, pr_url: "...", link_type: "closes", state: "open" }`
3. Fires `in_progress → implemented` auto-transition
4. Commits frontmatter to `main`:
   ```toml
   state = "implemented"
   updated_at = "2026-03-25T20:00:00Z"

   [[prs]]
   number = 7
   url = "https://github.com/philippepascal/ticker/pull/7"
   type = "closes"
   state = "open"
   review = ""
   ```
5. SQLite cache updated

**Board:** Card moves to IMPLEMENTED. PR badge `#7` appears. Philippe sees it under "needs action."

---

### Act 9: Philippe reviews — changes requested

Philippe reviews PR #7 on GitHub. He requests changes.

GitHub sends `pull_request_review` event (action: `submitted`, state: `changes_requested`).

APM updates `ticket_prs.review_state = "changes_requested"` in both SQLite and the frontmatter:

```toml
[[prs]]
number = 7
...
review = "changes_requested"
```

Committed to `main`:

```
commit f8a3c22
Author: apm-bot <apm@local>

    ticket(1): PR #7 review: changes_requested
```

**Board:** IMPLEMENTED card shows 🔴. Philippe can see at a glance this needs agent attention.

Agent sees the change on next `apm sync`:

```bash
apm show 1
# shows review = "changes_requested" in frontmatter
# agent checks GitHub for the specific review comments
```

Agent makes changes, pushes more commits. Philippe re-reviews and approves.

GitHub sends `pull_request_review` event (state: `approved`).

APM updates:
```toml
review = "approved"
```

**Board:** IMPLEMENTED card shows 🟢.

---

### Act 10: Philippe merges the PR — the two-layer reconciliation

Philippe merges PR #7 on GitHub (squash merge or regular merge, either works).

**The merge challenge:**

At this point the ticket file exists in two diverged versions:

```
main branch:
  tickets/0001-...md
    frontmatter: state="implemented", prs=[...], updated_at="..."
    body:        [frozen at apm start] + philippe's replies on main

feature branch:
  tickets/0001-...md
    frontmatter: state="in_progress" (unchanged on branch since apm start)
    body:        [frozen at apm start] + agent's spec updates + agent's conversation
```

Git's three-way merge (merge base = the commit when the branch was created):
- **Main changed:** frontmatter lines (state, prs, updated_at, review)
- **Branch changed:** body lines (spec revisions, conversation additions)
- **Overlap risk:** Both sides may have changed the Conversation section — main has Philippe's replies, branch has agent's questions and responses

If body changes are non-overlapping (Philippe's replies and agent's questions touch different lines), git auto-merges. If they interleave in ways git can't resolve, there's a merge conflict in the ticket file.

**Resolution strategy (spec must define this):**

Option A — Webhook reconciliation (recommended):
On `pull_request.merged`, APM's webhook handler:
1. Takes the merged commit's ticket file content (which git produced, possibly with conflicts resolved by the merge strategy)
2. Updates frontmatter: `state = "implemented"` (or fires auto-transition), `updated_at = now`
3. Commits the clean file to `main` as a follow-up commit

This makes APM the authority on the final file state, bypassing git's merge.

Option B — Branch discipline:
Agent never adds to the Conversation section on the branch. All conversation (both directions) goes to `main`. Agent reads replies from `main` via `apm sync`. Only spec and history changes go to the branch. This makes the body changes non-overlapping: `main` owns Conversation, branch owns Spec updates and History rows. Git auto-merge works cleanly.

Option B is simpler and more predictable. It requires `apm ask` to commit to `main` even when the agent is on a feature branch — which is a behavior change from the spec as written.

> **Spec gap — critical:** The spec does not define how the two-layer body reconciliation works at merge time. Option B (main owns Conversation, branch owns Spec and History) is the recommended path. This needs to be specified explicitly in §6 and in `apm.agents.md`.

**After merge:**

GitHub sends `pull_request.merged` event.

APM webhook handler:
1. Marks `ticket_prs` record: `state = "merged"`, `merged_at = now`
2. Checks: all `closes`-type PRs for ticket #1 are merged → yes (only one PR)
3. Fires `implemented → accepted` auto-transition
4. Commits frontmatter to `main`:
   ```toml
   state = "accepted"
   updated_at = "2026-03-26T09:00:00Z"
   ```
5. `post-merge` hook fires on Philippe's local clone: `apm sync --quiet` updates SQLite

**Board:** Card moves to ACCEPTED.

---

### Act 11: Philippe closes the ticket

```bash
apm state 1 closed
```

*File-level:* frontmatter updated on `main`.

*Optionally:* `apm.toml` can configure `archive_dir = "tickets/archive"`. If set, `apm state N closed` moves the file to `tickets/archive/0001-add-csv-export-for-portfolio-data.md` and commits both the deletion from `tickets/` and the creation in `tickets/archive/`.

**Board:** Card disappears from the default view (terminal states hidden). `apm list --closed` shows it.

---

## Agent setup: detailed

### What an agent needs to operate

**Environment variables** (set in the agent's shell or `.env` file for the session):

```bash
export APM_AGENT_NAME=claude-main          # identity in commits and ticket history
export APM_REPO=/home/agent/repos/ticker   # which repo to operate on
export APM_GITHUB_TOKEN=ghp_...            # if GitHub integration is enabled
```

**Files the agent reads:**

| File | When | Purpose |
|------|------|---------|
| `apm.agents.md` | Start of session via `apm agents` | Instructions, conventions, state machine overview |
| `apm.toml` | Implicitly via APM | State machine definition; the agent doesn't read this directly |
| `tickets/<id>.md` | Via `apm show` | Full ticket content |
| `~/.apm/apm.db` | Implicitly via APM | Board state; the agent doesn't query this directly |

**Session startup sequence (from `apm.agents.md`):**

```bash
apm sync                                     # pull latest
apm agents                                   # read instructions once
apm status                                   # per-state counts
apm list --needs-action --working            # my tickets needing attention
apm list --state new,ready                   # unworked tickets available
```

### Agent isolation: one ticket at a time

An agent should work one ticket at a time. Working two tickets simultaneously risks:
- Branch confusion (which branch are we on?)
- Frontmatter commit confusion (commits to `main` must know which ticket they're for)
- Conversation interleaving

If parallelism is needed, run two separate agent processes with separate clones of the repo.

> **Spec gap:** The spec does not state the one-ticket-per-agent-process constraint. `apm.agents.md` should include it. The CLI could warn if `APM_REPO` already has an in-flight ticket for this agent.

### Agent handoff

If `claude-main` is replaced by another agent instance mid-ticket:

```bash
# New agent takes over:
export APM_AGENT_NAME=claude-resume
apm take 1         # checks out feature/1-..., sets agent = "claude-resume"
apm show 1         # read the full history
apm show 1 --conv  # read the conversation
# continue from where claude-main left off
```

`apm take` commits to `main`:

```toml
agent = "claude-resume"
updated_at = "..."
```

The branch and all prior commits are preserved.

---

## .gitignore summary

| Path | In repo? | Notes |
|------|----------|-------|
| `apm.toml` | ✓ committed | Project config; should be version controlled |
| `apm.agents.md` | ✓ committed | Agent instructions; should be version controlled |
| `tickets/` | ✓ committed | The database; must be committed |
| `tickets/NEXT_ID` | ✓ committed | ID counter; must be committed |
| `tickets/archive/` | ✓ committed | Closed tickets; committed on `apm state N closed` |
| `.git/hooks/` | ✗ not committed | Local only; each clone runs `apm init --hooks` |
| `~/.apm/apm.db` | ✗ outside repo | User-global; not tracked |
| `~/.apm/*.pid` | ✗ outside repo | Server process files; not tracked |

**Nothing APM-related needs to be added to `.gitignore`** in a well-structured setup. `apm init` should make this explicit in its output.

If the ticket repo is also the code repo, engineers should ensure code build artifacts don't accidentally end up in `tickets/` (unlikely, but worth noting in `apm.agents.md`).

---

## Gaps and open questions surfaced

The following items need resolution in SPEC-V2.md:

| # | Issue | Where | Recommendation |
|---|-------|--------|----------------|
| G-1 | Who exits the `question` state after a supervisor reply? | §8 State Machine | `apm reply` should auto-transition back to the pre-question state (tracked in History). Add `on_reply` auto-transition to the state machine config |
| G-2 | Body changes in two-layer model: who owns the Conversation section on main vs branch? | §6 Ticket as Document | Define: Conversation always commits to `main`. Agent reads replies via `apm sync`. Branch owns Spec revisions and History appends only |
| G-3 | Merge reconciliation: how does the ticket file resolve at PR merge time? | §6, §8 | Specify Option B (branch discipline, defined ownership per section) as the canonical approach; webhook handler does a follow-up frontmatter-update commit |
| G-4 | `apm init` vs `apm init --hooks`: first-time project setup vs per-clone hook install | §11 CLI | Add `apm clone` shortcut (clone + init --hooks + sync) and document onboarding flow |
| G-5 | Tickets-in-separate-repo topology: `local` path for code repos | §13 Multi-repo | Add `local = "..."` field to `[[repos.code]]` for local path mapping |
| G-6 | Agent one-ticket-per-process constraint | §2, `apm.agents.md` | Document in `apm.agents.md`; consider CLI warning |
| G-7 | Server process files (PID, socket) location | §12 Web Client | Confirm these live in `~/.apm/`, not in the repo tree |
| G-8 | `apm start` order of operations: branch-then-frontmatter vs frontmatter-then-branch | §8 | Specify: frontmatter commit to `main` happens before branch push, so the board reflects in_progress even if push fails |
| G-9 | `apm ask` while on feature branch: commits to main or branch? | §6, §11 | Define: `apm ask` always commits to `main`; `apm reply` always commits to `main`; only `apm spec` and History appends commit to the feature branch |
| G-10 | The PR body format for linking APM tickets | §3 Provider, §11 CLI | Clarify that APM links via branch name; PR body `Closes #ticket/N` is documentation convention only, not parsed by APM |
