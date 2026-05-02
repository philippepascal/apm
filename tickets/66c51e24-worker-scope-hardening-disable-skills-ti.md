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

Workers are full Claude Code instances and inherit every skill the host has. Today the only constraint on worker behaviour is the text in `apm.worker.md` / `apm.spec-writer.md` ("stay on your ticket"), which the agent can disregard.

**Concrete incident (ticket 2803bf07's amendment round, 2026-04-30):** the spec-writer worker hit a Bash permission prompt during legitimate amendment work, then invoked the `fewer-permission-prompts` skill — which scanned past Claude Code transcripts at `~/.claude/projects/` and tried to Edit `.claude/settings.json` to add allowlist entries. The Edit was denied by the permission system (so no leak landed), but the worker spent ~124 KB of transcript on the off-topic side-quest and never returned to transition state. The mismatch: the worker thought project-improvement work was within its scope. It wasn't.

**Should land after the wrapper epic (4312fbd4) so the wrapper layer is the natural enforcement point.**

**Two enforcement layers proposed in this ticket:**

1. **Disable skill invocation for spawned workers.** Either via a Claude CLI flag (verify what `claude --print` supports — `--no-skills` or equivalent), or by adding an explicit clause to the system prompt: "Do not invoke any Claude Code skill. Ignore any skill availability information in your environment. Your only tool surface is the standard tool calls (Bash, Edit, Write, Read, Grep, etc.) directly applied to your ticket." The wrapper layer (post-epic) is the natural place — wrappers can pre-process the system prompt before passing to the agent.

2. **Per-role system-prompt tightening.** spec-writer doesn't need broad Bash; impl-agent needs more. Lock down each role's system prompt with an explicit, narrow allow-list of behaviours: "You may run apm spec, apm state, apm set, apm new --side-note. You may not run anything that modifies project configuration, the .apm/ directory beyond ticket files, the .claude/ directory, or .gitignore. If you encounter a permission prompt for an apm command, exit with a diagnostic noting the missing allowlist entry — do not invoke any skill or attempt to edit settings.json."

**Implementation pointers:**
- The role-specific system prompts live in `.apm/agents/<wrapper>/apm.<role>.md` post-epic (per ticket 7f5f73d5).
- The wrapper-layer pre-processing hook is the natural extension point — Claude built-in can strip skill metadata from the prompt; custom wrappers can opt in via a manifest flag (`disable_skills = true`).

**Out of scope:**
- The filesystem path validator (separate ticket — defense in depth at the tool-call layer).
- Pre-merge leak detection (separate ticket — defense at the apm state implemented layer).
- Permission-denial diagnostics (separate ticket — surfacing what the worker tried to do).

**Acceptance pointers:**
- A test that confirms a spawned worker cannot invoke an arbitrary skill (e.g., the worker's transcript does not contain task_notification entries for skills outside its role).
- The shipped `apm.worker.md` and `apm.spec-writer.md` defaults explicitly forbid skill invocation and project-tool modification.

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
