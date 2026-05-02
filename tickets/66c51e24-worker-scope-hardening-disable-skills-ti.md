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

- [ ] `build_claude_args()` in `apm-core/src/wrapper/builtin/claude.rs` always includes `--disable-slash-commands` in its output, verified by a unit test that checks every call path (with and without model, with and without skip-permissions)
- [ ] The bundled default `apm-core/src/default/agents/claude/apm.spec-writer.md` contains a "Scope limits" section that explicitly states skill/slash-command invocation is prohibited
- [ ] The bundled default `apm-core/src/default/agents/claude/apm.worker.md` contains a "Scope limits" section that explicitly states skill/slash-command invocation is prohibited
- [ ] The spec-writer "Scope limits" section lists exactly the permitted `apm` commands for that role: `apm spec`, `apm state`, `apm set`, `apm new --side-note`, `apm show`
- [ ] The worker "Scope limits" section lists exactly the permitted `apm` commands for that role: `apm show`, `apm state`, `apm new --side-note`
- [ ] Both "Scope limits" sections name the off-limits paths: `.claude/`, `.apm/config.toml` (and any file in `.apm/` other than the ticket), `.gitignore`, `.github/`
- [ ] Both "Scope limits" sections instruct the agent: on a permission prompt for an `apm` command, set the ticket to `blocked` and include a diagnostic naming the missing allowlist entry — never invoke a skill or attempt to edit `settings.json`
- [ ] The project-level `.apm/apm.spec-writer.md` contains the same "Scope limits" content as the bundled default
- [ ] The project-level `.apm/apm.worker.md` contains the same "Scope limits" content as the bundled default
- [ ] `cargo test --workspace` passes after all changes

### Out of scope

- Filesystem path validator at the tool-call layer (separate ticket — defense-in-depth below the system prompt)
- Pre-merge leak detection (separate ticket)
- Permission-denial diagnostics surfacing to the supervisor (separate ticket)
- Config-driven per-profile `disable_skills` opt-out — the flag is always on for the built-in `ClaudeWrapper`; projects that genuinely need skills can use a custom wrapper
- Manifest `disable_skills` field for custom wrappers — future extension once there is a known use case
- Non-claude built-in wrappers (`mock-happy`, `mock-sad`, `mock-random`, `debug`) — they do not invoke the claude CLI and are unaffected
- Changes to `APM_DISABLE_SKILLS` env var or wrapper contract version bump — no contract change needed

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