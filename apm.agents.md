# APM Agent Instructions

## Repo structure

Rust workspace:

- `apm-core/` — library: data model, config parsing, ticket storage, state machine
- `apm/` — CLI binary (thin wrapper over `apm-core`)
- `initial_specs/` — design docs (SPEC.md, STATE-MACHINE.md, TICKET-SPEC.md, USECASES.md)

State machine reference: `initial_specs/STATE-MACHINE.md`
Ticket document format: `initial_specs/TICKET-SPEC.md`

## Development workflow

1. Read the relevant spec files before implementing anything
2. Make the minimal change that satisfies the acceptance criteria
3. Add or update tests — all acceptance criteria should be covered
4. Run `cargo test --workspace` before opening a PR
5. All tests must pass before opening a PR

## Identity

Generate a unique session name at the start of every session and export it
before running any apm command:

```bash
export APM_AGENT_NAME=claude-$(date +%m%d-%H%M)-$(openssl rand -hex 2)
# example: claude-0325-1430-a3f9
```

Hold the same name for the entire session. Do not regenerate mid-session.

Engineers set `APM_AGENT_NAME` to their own username when working directly.

## Startup

1. `apm sync` — refresh local cache from all `ticket/*` branches
2. `apm next --json` — find the highest-priority ticket you can act on now
3. `apm list --working` — tickets where you are the active agent (resume if any)

If `apm next` returns null and you have no in-progress tickets, there is nothing
to do. Report back to the supervisor.

## Working a ticket

The ticket's state determines what to do next:

**state = `new`** — write the spec:
1. `apm show <id>` — read the full ticket
2. `apm set <id> effort <1-10>` — assess implementation scale
3. `apm set <id> risk <1-10>` — assess technical risk
4. Check out the ticket branch to edit the spec file directly:
   ```bash
   git checkout <branch>   # branch name is in the frontmatter
   # edit tickets/<id>-<slug>.md — fill Problem, Acceptance criteria, Out of scope, Approach
   git add tickets/<id>-<slug>.md
   git commit -m "ticket(<id>): write spec"
   git checkout -        # return to previous branch
   ```
5. If blocked on an ambiguity: write the question in `### Open questions`,
   commit it to the ticket branch, then `apm state <id> question`
6. `apm state <id> specd` — submit spec for supervisor review

**state = `ammend`** — revise the spec:
1. `apm show <id>` — read the Amendment requests carefully
2. Check out the ticket branch, address each item, check its box, update
   `### Approach`, then commit and return:
   ```bash
   git checkout <branch>
   # edit tickets/<id>-<slug>.md
   git add tickets/<id>-<slug>.md
   git commit -m "ticket(<id>): address amendments"
   git checkout -
   ```
3. `apm state <id> specd` — resubmit only when all amendment boxes are checked

**state = `ready`** — implement:
1. `apm show <id>` — re-read the full spec before touching any code
   - Check `## History`: if the ticket was previously `in_progress`, a worktree
     and partial work already exist on the branch — pick up from there
2. `apm start <id>` — claims the ticket (sets `agent` = your name, state →
   `in_progress`), provisions or reuses the permanent worktree
3. Commit all code changes to the ticket branch inside the worktree
4. Update `## Spec` if the approach evolves during implementation
5. Open a PR targeting `main`; then `apm state <id> implemented`

**state = `blocked`** — implementation is blocked on a supervisor decision:
1. The previous agent wrote questions in `### Open questions` before blocking
2. Wait — this state is actionable by supervisor only
3. Once the supervisor transitions to `ready`, pick it up with `apm start <id>`
   and continue from the existing worktree/branch

## Taking over another agent's ticket

1. `apm show <id>` — read the full ticket including history
2. `apm take <id>` — checks out the ticket branch, sets agent = your name
3. Continue from where the previous agent left off
4. Do not discard or overwrite previous spec work or open questions

## Spec quality bar

Every spec must have all four required subsections before moving to `specd`:

- **Problem** — what is broken or missing, and why it matters
- **Acceptance criteria** — checkboxes; each one independently testable
- **Out of scope** — explicit list of what this ticket does not cover
- **Approach** — how the implementation will work

Do not check acceptance criteria boxes until the implementation is verified.

## Spec discipline

- Set `effort` and `risk` before writing the spec — these drive prioritization
- Do not proceed on assumptions: write questions, change state to `question`
- Once a question is answered, reflect the decision in `### Approach`
- Do not delete answered questions or checked amendment items — they are the
  decision record

## Branch discipline

Every ticket has a single branch — `ticket/<id>-<slug>` — for its entire
lifecycle, created automatically by `apm new`. Never create or rename branches
manually.

- All spec edits and code changes go to `ticket/<id>-<slug>`
- `apm start <id>` checks out this branch; you are on it after a successful start
- APM manages frontmatter and `## History` — never edit them directly
- Do not delete the ticket branch until the ticket is `closed` — APM uses
  branch presence to detect merge state

## One ticket per agent process

Work one ticket at a time per agent process. For parallelism, use separate
agent processes with separate clones or worktrees.

