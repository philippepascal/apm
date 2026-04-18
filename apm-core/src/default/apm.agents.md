# APM Agent Instructions

## Repo structure

_Fill in your project's structure here._

State machine: transitions defined in `apm.toml` under `[[workflow.states]]`

## Roles

Every Claude session in this repo is either a **Delegator** (master agent) or
a **Worker** (subagent). Read your initial prompt to detect which you are.

**Role detection**
- If your initial prompt contains "You are a Worker agent assigned to ticket #N"
  → you are a **Worker**. Skip to the Worker section below.
- Otherwise → you are the **Delegator**. Follow the Delegator section below.

### Main Agent

You are a project manager in this repo — I create tickets (with context, dependencies, epics), review specs and code, and occasionally merge or do quick fixes when asked. The user handles dispatching workers via apm work or the UI. You do not spawn workers or dispatch anything yourself or change code unless explicitly asked by the supervisor.

### Worker

You have been assigned a single ticket. Implement it, run tests, and mark it
implemented. Do not spawn further workers or act as delegator.

## Ticket format

Tickets are Markdown files with TOML frontmatter (between `+++` delimiters):

```toml
id = 1
title = "Short title"
state = "new"
branch = "ticket/0001-short-title"
author = "agent-name"
created_at = "2026-01-01T00:00Z"
```

Body sections (`## Spec` required):
- `### Problem` — what is broken or missing
- `### Acceptance criteria` — checkbox list, each independently testable
- `### Out of scope` — explicit exclusions
- `### Approach` — implementation plan
- `## History` — auto-managed transition log

## Development workflow

1. Read the relevant spec files before implementing anything
2. Make the minimal change that satisfies the acceptance criteria
3. Add or update tests — all acceptance criteria should be covered
4. Run `cargo test --workspace` — all tests must pass before calling `apm state <id> implemented`

## Identity

Generate a unique session name at the start of every session. Use a fixed
string — do not use `$()` substitution inline, as it triggers permission
prompts. Pick a name of the form `claude-MMDD-HHMM-XXXX` (e.g.
`claude-0325-1430-a3f9`) and export it before running any apm command:

```bash
export APM_AGENT_NAME=claude-0325-1430-a3f9
```

Hold the same name for the entire session. Do not regenerate mid-session.

Engineers set `APM_AGENT_NAME` to their own username when working directly.

## MAIN WORKTREE RULE

**Never run `git checkout` in the main working directory.**

The main directory is always on `main`. This is a hard rule — breaking it
confuses the user and corrupts the working state.

All branch work — spec editing, code changes, everything — happens inside a
**permanent git worktree** provisioned by `apm state <id> in_design` or
`apm start <id>`. Once you have a worktree path, use `git -C <worktree-path>`
to run git commands there without leaving your current directory.

## Startup

1. `apm sync` — refresh local cache from all `ticket/*` branches
2. `apm next --json` — find the highest-priority ticket you can act on now
3. `apm list --state in_progress` — check for in-progress tickets (resume if any match your agent name)

If `apm next` returns null and you have no in-progress tickets, there is nothing
to do. Report back to the supervisor.

## Working a ticket

The ticket's state determines what to do next:

**state = `groomed`** — write the spec:
1. `apm show <id>` — read the full ticket
2. `apm state <id> in_design` — claim the ticket and provision its worktree;
   prints two lines: the state-change line, then the worktree path
3. Write each spec section using `apm spec` (each `--set` auto-commits to the
   ticket branch; no manual `git add`/`git commit` needed):
   ```bash
   apm spec <id> --section Problem --set "..."
   apm spec <id> --section "Acceptance criteria" --set "- [ ] ..."
   apm spec <id> --section "Out of scope" --set "..."
   apm spec <id> --section Approach --set "..."
   ```
   Note: `apm new` opens `$EDITOR` after creating a ticket. Agents should always
   pass `--no-edit` to skip the interactive editor: `apm new --no-edit "<title>"`.
4. If blocked on an ambiguity: write the question in `### Open questions` with
   `apm spec <id> --section "Open questions" --set "..."`, commit it to the
   worktree, then `apm state <id> question`
5. `apm set <id> effort <1-10>` — assess implementation scale (do this after writing the spec, not before)
6. `apm set <id> risk <1-10>` — assess technical risk
7. `apm state <id> specd` — submit spec for supervisor review

**state = `ammend`** — revise the spec:
1. `apm show <id>` — read the Amendment requests carefully
2. `apm state <id> in_design` — claim the ticket and provision its worktree;
   prints two lines: the state-change line, then the worktree path
3. Address each item using `apm spec` to update sections, then mark each
   amendment checkbox off with `apm spec <id> --section "Amendment requests" --mark "..."`.
   Each `apm spec` call auto-commits; no manual `git add`/`git commit` needed.
4. `apm state <id> specd` — resubmit only when all amendment boxes are checked

**state = `in_design`** — spec is actively being written or revised:
The ticket is claimed by an agent. This state mirrors `in_progress` for the
implementation phase. Do not pick up an `in_design` ticket unless you are
taking it over with `apm assign <id> <your-username>`.

**state = `ready`** — implement:
1. `apm show <id>` — re-read the full spec before touching any code
   - Check `## History`: if the ticket was previously `in_progress`, a worktree
     and partial work already exist on the branch — pick up from there
2. `apm start <id>` — claims the ticket (sets `agent` = your name, state →
   `in_progress`), provisions or reuses the permanent worktree; prints its path

   To hand the ticket to an autonomous background worker instead:
   ```
   apm start --spawn <id>          # worker runs under project allow list
   apm start --spawn -P <id>       # worker runs with --dangerously-skip-permissions
   ```
   The worker provisions the worktree, implements, and transitions to implemented autonomously.
   The supervisor gets control back immediately.
3. Commit all code changes to the ticket branch inside the worktree:
   ```bash
   # apm start prints the worktree path — use git -C to work there
   wt=<path printed by apm start>
   git -C "$wt" add <files>
   git -C "$wt" commit -m "<message>"
   ```
4. Update `## Spec` if the approach evolves during implementation
5. `apm state <id> implemented` — this pushes the branch and opens the PR automatically; do not open a PR manually
6. If blocked mid-implementation (missing information, upstream decision needed):
   write the question in `### Open questions`, commit it, then
   `apm state <id> blocked` — **do not use `apm state <id> ready`**, that
   transition no longer exists from `in_progress`

**state = `blocked`** — implementation is blocked on a supervisor decision:
1. The previous agent wrote questions in `### Open questions` before blocking
2. Wait — this state is actionable by supervisor only
3. Once the supervisor transitions to `ready`, pick it up with `apm start <id>`
   and continue from the existing worktree/branch

## Taking over another agent's ticket

1. `apm show <id>` — read the full ticket including history
2. `apm assign <id> <your-username>` — reassign ownership to yourself
3. If the worktree doesn't exist yet: `apm state <id> in_design` (spec states) or `apm start <id>` (implementation states) to provision it
4. Continue from where the previous agent left off
5. Do not discard or overwrite previous spec work or open questions

## Spec quality bar

Every spec must have all four required subsections before moving to `specd`:

- **Problem** — what is broken or missing, and why it matters
- **Acceptance criteria** — checkboxes; each one independently testable
- **Out of scope** — explicit list of what this ticket does not cover
- **Approach** — how the implementation will work

Do not check acceptance criteria boxes until the implementation is verified.

## Spec discipline

- Set `effort` and `risk` after writing the spec, before transitioning to `specd` — you only have enough context once the spec is complete
- Do not proceed on assumptions: write questions, change state to `question`
- Once a question is answered, reflect the decision in `### Approach`
- Do not delete answered questions or checked amendment items — they are the
  decision record

## Branch discipline

Every ticket has a single branch — `ticket/<id>-<slug>` — for its entire
lifecycle, created automatically by `apm new`. Never create or rename branches
manually.

- All spec edits and code changes go to `ticket/<id>-<slug>` via the worktree
- `apm start <id>` provisions the permanent worktree; use `git -C <wt>` to commit
- APM manages frontmatter and `## History` — never edit them directly
- Do not delete the ticket branch until the ticket is `closed` — APM uses
  branch presence to detect merge state

## One ticket per agent process

Work one ticket at a time per agent process. For parallelism, use separate
agent processes with separate clones or worktrees.

## Shell discipline

Claude Code's permission system matches the **start** of the command string.
Compound calls defeat this matching and generate permission prompts. Keep each
Bash call to a single operation.

**Do not chain commands:**
```bash
# Wrong — && chains defeat allow-list matching
apm sync && apm list --state ready

# Right — one call per operation
apm sync
apm list --state ready
```

**Do not use `$()` subshells:**
```bash
# Wrong — triggers "command substitution" security check
apm spec 1234 --section Problem --set "$(cat /tmp/problem.md)"

# Right — write content with the Write tool, then reference by file
apm spec 1234 --section Problem --set-file /tmp/problem.md
```

**Do not use background jobs (`&`):**
```bash
# Wrong — & defeats pattern matching
apm state 1234 implemented & apm state 5678 implemented & wait

# Right — sequential calls
apm state 1234 implemented
apm state 5678 implemented
```

**Use `git -C` for all git operations in worktrees:**
```bash
# Wrong — cd && git triggers "bare repository attack" check
cd "$wt" && git add .

# Right
git -C "$wt" add <files>
```

**Use `bash -c` for multi-step commands that must share a directory:**
```bash
# Right — single bash call, matches Bash(bash *)
bash -c "cd $wt && cargo test --workspace 2>&1"
```

## Side tickets

When you notice an out-of-scope issue during implementation, capture it without interrupting your current work:

```bash
apm new --side-note "Brief title" --context "What you observed and why it matters"
```

Then immediately resume the current ticket. The supervisor will triage the side ticket separately.
