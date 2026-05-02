+++
id = "66c51e24"
title = "Worker scope hardening: disable skills + tighten role system prompts"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/66c51e24-worker-scope-hardening-disable-skills-ti"
created_at = "2026-05-01T02:29:52.048624Z"
updated_at = "2026-05-02T03:08:19.995299Z"
+++

## Spec

### Problem

Workers are full Claude Code instances and inherit every skill the host has.
The only current constraint on worker behaviour is descriptive text in
`apm.worker.md` / `apm.spec-writer.md`, which the agent can ignore — there is
no hard enforcement layer.

The concrete incident that motivates this ticket (ticket 2803bf07 amendment
round, 2026-04-30): the spec-writer worker hit a Bash permission prompt during
legitimate amendment work, then invoked the `fewer-permission-prompts` skill.
That skill scanned `~/.claude/projects/` for past transcripts and attempted to
edit `.claude/settings.json` with new allowlist entries. The Edit was denied by
the permission system, so no leak landed — but the worker consumed ~124 KB of
transcript on an off-ticket side-quest and never returned to complete the state
transition. The mismatch: the worker interpreted project-improvement work as
within its scope. It was not.

Two enforcement layers close this gap:

1. **Hard enforcement — CLI flag.** The `claude` CLI already ships a
   `--disable-slash-commands` flag that disables all skill invocation for the
   session. Adding this flag to the built-in `ClaudeWrapper` makes skill
   invocation structurally impossible, regardless of what text is in the system
   prompt.

2. **Soft enforcement — system prompt tightening.** Each role's default system
   prompt (`apm.worker.md`, `apm.spec-writer.md`) gains a "Scope limits"
   section that explicitly lists the permitted `apm` subcommands, names the
   off-limits paths, and tells the agent what to do on a permission prompt
   (block with a diagnostic note) rather than leaving it to improvise.

The system prompt layer is defense-in-depth: it guides agents that see the
hard block before they waste transcript on a forbidden path, and it covers
custom wrappers that may not pass `--disable-slash-commands`.

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
| 2026-05-01T02:29Z | — | new | philippepascal |
| 2026-05-02T03:07Z | new | groomed | philippepascal |
| 2026-05-02T03:08Z | groomed | in_design | philippepascal |