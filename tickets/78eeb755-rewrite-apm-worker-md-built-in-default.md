+++
id = "78eeb755"
title = "Rewrite apm.worker.md built-in default"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/78eeb755-rewrite-apm-worker-md-built-in-default"
created_at = "2026-05-22T23:22:24.735576Z"
updated_at = "2026-05-23T00:06:33.529498Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771"]
+++

## Spec

### Problem

`apm-core/src/default/agents/default/apm.worker.md` currently contains two categories of content that become redundant once T1 (4bee5771) lands:

1. **`## Shell discipline` (lines 134–183):** a verbatim copy of the agents.md shell-discipline block. After T1, `apm instructions` emits this section dynamically; keeping it in the role file would duplicate it.
2. **Preamble back-reference to `agents.md`:** line 6 says "Read `.apm/agents/default/agents.md` for startup, identity, worktree setup, and shell discipline." `agents.md` is being deleted in T7 (1fce91bd); after that deletion this reference is dead.

The file also lacks two rules that workers need when committing ticket files as part of the `implemented` or `blocked` flows:

- **Never hand-edit the `## History` table** — workers must use `apm state`, not write the table directly.
- **Filename is fixed** — renaming the ticket file breaks `apm list` / `apm show`.

Both rules exist in `apm.spec-writer.md` but are absent from `apm.worker.md`. Workers write spec content when blocking (open questions) and commit ticket files; without these guards they can corrupt the ticket in the same ways spec-writers can.

### Acceptance criteria

- [ ] The `## Shell discipline` section is absent from the rewritten file
- [ ] The preamble no longer contains a reference to `agents.md`
- [ ] A "Never hand-edit the History table" rule is present with the same normative content as in `apm.spec-writer.md`
- [ ] A "Filename is fixed — never rename the ticket file" rule is present with the same normative content as in `apm.spec-writer.md`
- [ ] All sections present in the original that are not being removed are unchanged in meaning
- [ ] `cargo test --workspace` passes

### Out of scope

- The `claude/` override (`apm-core/src/default/agents/claude/apm.worker.md`) — covered by T6 (02bbcc2f)
- Shell discipline content itself — owned by T1 (4bee5771)
- Session identity and startup sequence — covered by T1
- Deletion of `agents.md` — T7 (1fce91bd)
- Rewriting `apm.spec-writer.md` — T4 (34ad9126)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:06Z | groomed | in_design | philippepascal |