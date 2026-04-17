+++
id = "1ead1f53"
title = "Inject dependency context bundle into implementation workers"
state = "in_design"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1ead1f53-inject-dependency-context-bundle-into-im"
created_at = "2026-04-17T07:27:10.664091Z"
updated_at = "2026-04-17T07:36:40.640120Z"
epic = "35199c7f"
target_branch = "epic/35199c7f-give-workers-cross-ticket-context"
+++

## Spec

### Problem

When an implementation worker is spawned (at `in_progress`), it sees its ticket's spec but not the substance of what its dependencies actually produced: helper names, type shapes, decisions recorded in amendment cycles, commits that landed. The worker either re-reads the full dependency branches (slow, lossy) or guesses. The common failure modes are duplicating a helper that already exists upstream and choosing an API shape that doesn't compose with what a dependency actually delivered.

### Acceptance criteria

- [ ] When an implementation worker is spawned on a ticket with a `depends_on` set, APM generates a dependency context bundle (markdown) and prepends it to the worker's prompt.
- [ ] For each direct dependency, the bundle includes: ticket id + title, the full Approach section, and a commit-subject list of what landed on that dependency's branch (capped, e.g. 20 commits).
- [ ] Transitive dependencies are included one level deep with title + one-line summary only (to avoid bundle explosion).
- [ ] If a dependency is not yet closed/merged, the bundle notes its current state and flags that the dependency may still change — the worker should tread carefully.
- [ ] Tickets with no dependencies spawn with no bundle.
- [ ] Spec workers also receive this bundle (so the Approach section they write can reference real upstream APIs), gated on whether dependencies exist at spec time.
- [ ] Integration test covers bundle assembly with direct + transitive dependencies.

### Out of scope

- Full diff inclusion (commit subjects are enough; the worker can read the code on demand).
- Dependency ordering / scheduling changes (APM already gates ready on deps; this is purely context injection).
- Epic-level sibling context — handled by the epic context bundle ticket.

### Approach

- Add `apm-core::context::build_dependency_bundle(ticket_id)` returning a `String`.
- Hook into spawn paths for both `ready → in_progress` and `groomed/ammend → in_design` (so spec workers also get dependency context when their ticket has dependencies declared at spec time).
- Commit list: `git log --pretty=%s <target>..<dep_branch_tip>`.
- Direct dependencies include their ticket id, title, full Approach section, and capped commit list. Transitive dependencies (one level deep) include only title + one-line Problem summary.
- For not-yet-merged dependencies, include current state and a warning that the dependency may still change.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T07:27Z | — | new | philippepascal |
| 2026-04-17T07:35Z | new | groomed | claude-0417-1430-c7a2 |
| 2026-04-17T07:36Z | groomed | in_design | claude-0417-1430-c7a2 |