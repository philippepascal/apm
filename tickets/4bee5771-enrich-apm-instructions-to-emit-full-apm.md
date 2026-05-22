+++
id = "4bee5771"
title = "Enrich apm instructions to emit full APM system knowledge"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4bee5771-enrich-apm-instructions-to-emit-full-apm"
created_at = "2026-05-22T23:22:16.080767Z"
updated_at = "2026-05-22T23:51:46.172038Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
+++

## Spec

### Problem

`apm instructions` currently emits a compact one-liner-per-command summary (`apm/src/cmd/instructions.rs`). For the prompt redesign it needs to emit full APM system knowledge so role-specific prompt files (`apm.worker.md`, `apm.spec-writer.md`) no longer need to duplicate state-machine, ticket-format, shell-discipline, or session-identity content. The function that generates this text must live in `apm-core/src/instructions.rs` so both the CLI command and the prompt builder (`apm-core/src/start.rs build_system_prompt`) can call it without a clap dependency in `apm-core`.

Emitting the full state machine to a worker that only touches `ready → in_progress → implemented` wastes context. The command needs a `--role <name>` flag that scopes the output to what is relevant for that role. With no flag the output is generic and complete (appropriate for the main agent). Role names match those defined in the workflow config: derived from transition `instructions` path basenames (e.g. `apm.spec-writer.md` → `spec-writer`) and from `WorkerProfileConfig.role`. Scoping affects the state machine section (only states and transitions the role acts in or needs awareness of) and the command reference (only commands the role uses). Shell discipline, session identity, and ticket format are role-independent and always emitted in full.

### Acceptance criteria

- [ ] `apm instructions` (no role) output contains all five sections in order: state machine, ticket format, shell discipline, session identity, command reference
- [ ] Output contains no ANSI escape codes regardless of flags used
- [ ] State machine section lists workflow states, their transitions, and actor information — read from the project's workflow config when present, falling back to a built-in static description otherwise
- [ ] Ticket format section lists required frontmatter fields and body sections (name, type, required flag) — read from ticket config when present, falling back to built-in static content otherwise
- [ ] `apm instructions --role <name>` state machine section includes only states and transitions where the named role acts or needs awareness; states the role never touches are omitted
- [ ] `apm instructions --role <name>` command reference includes only commands relevant to the named role (hard-coded per-role allowlists in `apm-core/src/instructions.rs`)
- [ ] Shell discipline and session identity sections are present and unabridged regardless of `--role`
- [ ] `apm_core::instructions::generate(root, role, commands)` is idempotent and callable without clap as a transitive dependency on `apm-core`

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
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-22T23:51Z | groomed | in_design | philippepascal |