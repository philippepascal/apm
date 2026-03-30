+++
id = "4df807d8"
title = "apm worktrees --add: internalize as apm-core function, remove from public CLI"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/4df807d8-apm-worktrees-add-internalize-as-apm-cor"
created_at = "2026-03-30T06:15:20.855321Z"
updated_at = "2026-03-30T06:20:32.203764Z"
+++

## Spec

### Problem

`apm worktrees --add <id>` is documented in `apm.agents.md` and used by agents
for spec-writing states (`new` → `in_design`, `ammend` → `in_design`). But per
`TICKET-LIFECYCLE.md`, worktree provisioning is always an internal step driven
by another command — it is never a user or agent action in its own right.

`apm start` already provisions worktrees internally and prints the path. The
spec-writing path lacks an equivalent: agents must call `apm worktrees --add`
manually, which leaks an implementation detail into the public CLI and agent
instructions.

The fix: move worktree provisioning into an `apm-core` function shared by all
commands that need it, have `apm state <id> in_design` auto-provision and print
the worktree path (mirroring `apm start`), and remove `--add` from the public
`apm worktrees` interface. Update `apm.agents.md` to remove the manual
`apm worktrees --add` calls.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T06:15Z | — | new | claude-0330-0245-main |
| 2026-03-30T06:20Z | new | in_design | claude-0330-0245-main |
