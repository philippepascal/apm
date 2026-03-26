# APM Agent Instructions

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

1. `apm sync` — refresh local cache from git
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
4. Edit the ticket's `## Spec` section: fill Problem, Acceptance criteria,
   Out of scope, Approach
5. If blocked on an ambiguity: write the question in `### Open questions`,
   then `apm state <id> question`
6. `apm state <id> specd` — submit spec for supervisor review

**state = `ammend`** — revise the spec:
1. `apm show <id>` — read the Amendment requests carefully
2. Address each item; check its box when done
3. Update `### Approach` to reflect any decisions
4. `apm state <id> specd` — resubmit only when all amendment boxes are checked

**state = `ready`** — implement:
1. `apm show <id>` — re-read the full spec before touching any code
2. `apm start <id>` — creates branch `feature/<id>-<slug>`, sets agent = your
   name, moves ticket to `in_progress`
3. Commit code to the feature branch
4. Update `## Spec` on the branch if the approach evolves
5. Open a PR; then `apm state <id> implemented`

## Taking over another agent's ticket

1. `apm show <id>` — read the full ticket including history
2. `apm take <id>` — checks out the branch, sets agent = your name
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

- All code changes on the feature branch (`feature/<id>-<slug>`)
- `## Spec` section lives on the feature branch
- Frontmatter and `## History` are managed by APM on `main` — never edit them
  directly
- Do not delete the feature branch until the ticket is `closed` — APM uses
  branch presence to detect merge state

## One ticket per agent process

Work one ticket at a time per agent process. For parallelism, use separate
agent processes with separate clones (`apm dispatch` handles this).

**Git worktrees:** If running inside a `git worktree`, ticket state commits to
`main` require special handling — the feature branch worktree cannot checkout
`main` while the primary worktree holds it. APM will detect this automatically
and commit via the primary worktree (see ticket #15). Until #15 is implemented,
use separate clones rather than worktrees for parallel agent work.
