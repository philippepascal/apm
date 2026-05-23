+++
id = "78eeb755"
title = "Rewrite apm.worker.md built-in default"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/78eeb755-rewrite-apm-worker-md-built-in-default"
created_at = "2026-05-22T23:22:24.735576Z"
updated_at = "2026-05-23T00:09:05.710671Z"
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

One file changes: `apm-core/src/default/agents/default/apm.worker.md`. No Rust source changes — the `include_str!` in `apm-core/src/agents.rs:89` references the same filename, and no test asserts on the content of the default worker.md.

1. **Remove the `agents.md` back-reference from the preamble.** The current second paragraph (lines 5–8) reads: `Read .apm/agents/default/agents.md for startup, identity, worktree setup, and shell discipline. This file covers the implementation phase only.` Replace it with: `Shell discipline, session identity, and startup sequence are covered by \`apm instructions\` — this file covers the implementation phase only.`

2. **Delete the `## Shell discipline` section** (lines 134–183) in its entirety, including its heading.

3. **Add a `## Ticket file discipline` section** immediately after `## Side tickets`. Copy the two subsections verbatim from `apm.spec-writer.md`:
   - `### Never hand-edit the History table` — full block including the three bullet points and the closing paragraph
   - `### Filename is fixed — never rename the ticket file` — full block including the **Rules:** list and the closing note

4. All other sections (`## Scope limits`, `## Before writing any code`, `## Minimal-change discipline`, `## Commit format`, `## Tests`, `## Finishing implementation`, `## Blocked state`, `## Capability limitations`, `## Path discipline`, and the frontmatter-override note) are preserved unchanged.

5. Run `cargo test --workspace` — all tests must pass before marking implemented.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:06Z | groomed | in_design | philippepascal |