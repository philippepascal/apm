+++
id = 7
title = "Implement apm start (branch creation, git mutex)"
state = "specd"
priority = 10
effort = 5
risk = 4
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

`apm start <id>` is the entry point for implementation. It must atomically:
create the feature branch, claim the ticket (set `agent`), and move the state to
`in_progress`. Without atomicity, two concurrent agents can race to start the
same ticket. The git commit+push sequence is the mutex: the second agent's push
is rejected, signaling it must retry with `apm next`.

### Acceptance criteria

- [ ] `apm start <id>` creates branch `feature/<id>-<slug>` from current HEAD of default branch
- [ ] Branch name is derived from the ticket slug (same logic as the filename)
- [ ] Frontmatter fields `agent`, `branch`, and `state` are updated and committed to `main`
- [ ] The commit is pushed to `origin/main`; a push rejection is surfaced as an error with a clear message ("ticket already claimed — run apm next")
- [ ] After a successful push, the working tree is checked out to the feature branch
- [ ] Running `apm start` on a ticket not in an actionable state fails with a clear error
- [ ] Running `apm start` on an already-assigned ticket (agent set) fails with a clear error before attempting the push

### Out of scope

- `apm take` (handoff from another agent) — tracked in #9
- `apm _hook pre-push` auto-transition — tracked in #8
- Precondition enforcement from the state machine config (spec checks, etc.)

### Approach

New subcommand `apm start <id>` in `apm/src/cmd/start.rs`:

1. Load ticket; check state is in `actionable_states` and `agent` is null — fail fast otherwise
2. Determine `APM_AGENT_NAME` from env (error if unset)
3. Set `frontmatter.agent`, `frontmatter.branch = "feature/<id>-<slug>"`, `frontmatter.state = "in_progress"`, `frontmatter.updated`
4. Append history row
5. Save ticket, `git add <ticket_path>`, `git commit -m "Start #<id>: claim ticket (<agent>)"`
6. `git push origin main` — on rejection, revert the commit (`git reset --soft HEAD~1`), restore ticket file, and print the retry message
7. On success: `git checkout -b feature/<id>-<slug>` (or `git checkout feature/<id>-<slug>` if it already exists locally)

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
