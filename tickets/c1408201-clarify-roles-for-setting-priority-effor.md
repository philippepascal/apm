+++
id = "c1408201"
title = "clarify roles for setting priority, effort and risk"
state = "closed"
priority = 3
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c1408201-clarify-roles-for-setting-priority-effor"
created_at = "2026-05-28T05:50:39.594077Z"
updated_at = "2026-05-28T06:43:33.690129Z"
+++

## Spec

### Problem

The agent instructions do not clearly assign ownership of the `priority` field. `apm.main-agent.md` describes the grooming step (`new → groomed`) without mentioning that the supervisor should set priority at that point. `apm.spec-writer.md` instructs the spec-writer to set `effort` and `risk` before transitioning to `specd`, but says nothing about `priority`.

The result is that priority is regularly left at `0` after grooming. A spec-writer who sets it is overstepping (priority is a business-value judgment), while one who leaves it unset produces a ticket that `apm next` cannot rank. Clarifying each role's responsibility closes this gap: the supervisor owns priority at groom time; the spec-writer sets it only as a fallback if the supervisor skipped it.

### Acceptance criteria

- [x] `apm.main-agent.md` states that setting priority with `apm set <id> priority <value>` is part of the `new → groomed` transition
- [x] `apm.spec-writer.md` "When you are done" lists `apm set <id> priority <1-10>` with the annotation "only if not already set by the supervisor"
- [x] `apm.spec-writer.md` still lists `apm set <id> effort <1-10>` and `apm set <id> risk <1-10>` as unconditional pre-transition steps

### Out of scope

- Changes to any agent files other than `apm.main-agent.md` and `apm.spec-writer.md`
- Changes to other agent profiles (phi4, pi)
- Runtime enforcement of the priority-fallback rule in the `apm` CLI
- Changes to the effort or risk fields or their documentation

### Approach

Two files change, both under `.apm/agents/claude/`.

#### `apm.main-agent.md`

In the `## Supervisor-only transitions` section, add a prose note under the `new → groomed` bullet explaining that the supervisor should set priority before grooming. The note should show the command and explain the rationale (priority is a business-value call that determines queue order):

```
- `new → groomed` — before grooming, set the ticket's priority:
  `apm set <id> priority <value>`  (1 = lowest, 10 = highest)
  Priority is the supervisor's business-value judgment; setting it here
  ensures `apm next` can rank the ticket correctly.
```

Alternatively, a short "## Grooming" section can be added if the supervisor-only transitions block becomes too dense. Either presentation satisfies the AC.

#### `apm.spec-writer.md`

In the `## When you are done` section, add the priority command after the existing two `apm set` lines, with the conditional annotation:

```
Before transitioning, set:
- `apm set <id> effort <1-10>`
- `apm set <id> risk <1-10>`
- `apm set <id> priority <1-10>`  — only if not already set by the supervisor

Then: `apm state <id> specd`
```

No other sections of either file change.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-28T05:50Z | — | new | philippepascal |
| 2026-05-28T06:09Z | new | groomed | philippepascal |
| 2026-05-28T06:16Z | groomed | in_design | philippepascal |
| 2026-05-28T06:18Z | in_design | specd | claude |
| 2026-05-28T06:27Z | specd | ready | philippepascal |
| 2026-05-28T06:27Z | ready | in_progress | philippepascal |
| 2026-05-28T06:28Z | in_progress | implemented | claude |
| 2026-05-28T06:43Z | implemented | closed | philippepascal(apm-sync) |
