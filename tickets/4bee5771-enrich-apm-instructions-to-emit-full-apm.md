+++
id = "4bee5771"
title = "Enrich apm instructions to emit full APM system knowledge"
state = "in_progress"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4bee5771-enrich-apm-instructions-to-emit-full-apm"
created_at = "2026-05-22T23:22:16.080767Z"
updated_at = "2026-05-23T02:58:45.656188Z"
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

- Wiring `generate()` into `build_system_prompt` — that is T3 (d8e2fa0e)
- Updating CLI help text for `apm instructions` and `apm prompt` — that is ticket bfa41899
- Rewriting `apm.worker.md` or `apm.spec-writer.md` built-in role files — those are T4 (78eeb755) and T5 (34ad9126)
- Removing `agents.md` or migrating the project's `.apm/agents/` directory — those are T7 (1fce91bd) and T8 (7c5c491d)
- Defining a canonical role catalog beyond what already exists in the workflow config
- Unknown `--role` validation beyond a best-effort warning; the fallback is full (unscoped) output

### Approach

#### 1. Create `apm-core/src/instructions.rs`

Public entry point:

```rust
pub fn generate(root: &Path, role: Option<&str>, commands: &[(String, String)]) -> Result<String>
```

- `root` — project root; `Config::load(root)` retrieves `WorkflowConfig` and `TicketConfig`
- `role` — optional role name (e.g. `"worker"`, `"spec-writer"`)
- `commands` — `(name, about)` pairs pre-extracted from the CLI by the caller; keeps `apm-core` free of a clap dependency

The function builds each section into a `String` buffer and returns the concatenation.

Register in `apm-core/src/lib.rs`: `pub mod instructions;`

#### 2. State machine section

Call `Config::load(root)`; on error fall back to a static built-in description of the standard APM state machine (the same states documented in `agents.md`).

For each `WorkflowState`, emit: state id, label, who can act (`actionable`). For each `TransitionConfig` of that state, emit `→ <to>`, trigger, and derived role. Role is derived from the transition by: (a) `profile.role` if a profile is resolved, (b) basename of `instructions` path stripped of `apm.` prefix and `.md` suffix (e.g. `.apm/agents/default/apm.spec-writer.md` → `spec-writer`), (c) default `"worker"`.

**Role filtering:** when `role.is_some()`, collect the set of states to emit by scanning all transitions across all states; include a state if any outgoing transition matches the role OR if it is the `to` target of such a transition. Emit only those states and only the matching transitions within them. If no transitions match (unknown role), emit a warning line and fall back to the unscoped full output.

#### 3. Ticket format section

From `Config.ticket.sections`, emit each section's name, type (`free` / `tasks` / `qa`), and required flag. Precede this with a hard-coded list of standard frontmatter fields (`id`, `title`, `state`, `priority`, `effort`, `risk`, `author`, `owner`, `branch`, `created_at`, `updated_at`; optional: `epic`, `target_branch`, `depends_on`). On config load failure, emit a static built-in description of the default ticket schema.

#### 4. Shell discipline section — static string

Content: the constraints from the current `agents.md` shell-discipline block: no `&&`, no `&`, no `$()` subshells, use `git -C` for worktree git commands, one command per Bash call, use Write tool for temp files instead of heredocs or `$()`. Always emitted in full.

#### 5. Session identity section — static string

Content: export `APM_AGENT_NAME=claude-MMDD-HHMM-XXXX` before running any `apm` command; hold the same name for the entire session; do not regenerate mid-session. Always emitted in full.

#### 6. Command reference section

With no role: format all `commands` entries as the current `render_compact_commands` does.

With a role: filter `commands` to those whose name appears in a hard-coded per-role allowlist:

- `"spec-writer"`: `show`, `spec`, `set`, `state`, `new`, `sync`, `list`, `next`
- `"worker"`: `show`, `start`, `state`, `spec`, `new`, `sync`, `list`, `next`
- any unrecognized role: all commands (unscoped fallback)

#### 7. Update `apm/src/cmd/instructions.rs`

Change `run(cli_cmd: clap::Command)` to `run(cli_cmd: clap::Command, root: &Path, role: Option<&str>)`. Extract the `Vec<(String, String)>` from `cli_cmd` (reuse existing non-hidden subcommand iteration). Call `apm_core::instructions::generate(root, role, &commands)` and print the result.

Move the existing unit tests (which tested internal helpers directly) to tests on `generate()` via a temp dir with no `.apm/` present, so the static fallback path is exercised.

#### 8. Update `apm/src/main.rs`

Add `role: Option<String>` field to the `Instructions` variant with `#[arg(long, value_name = "ROLE")]`. Update the handler arm to pass `&root` and `role.as_deref()` to `cmd::instructions::run`.

#### 9. Tests in `apm-core/src/instructions.rs`

Use a `tempfile::TempDir` with no `.apm/` directory (triggers static fallbacks throughout):

- `generate_no_role_contains_all_sections` — assert each of the five section headers is present
- `generate_no_ansi` — assert output contains no `\x1b`
- `generate_is_idempotent` — assert two calls with identical args return equal strings
- `generate_role_independent_sections` — call with `role = Some("worker")`; assert shell discipline and session identity headers present
- `generate_worker_scopes_commands` — call with `role = Some("worker")`; assert `"start"` appears; for the state machine fallback, assert the static text for `in_progress` is present (worker acts there) and that a spec-writer-only marker (e.g. literal `"groomed"` state description) is absent or not under a state heading — skip this assertion if the static fallback does not partition by role (acceptable for static content; only the live config path needs to filter)

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-22T23:51Z | groomed | in_design | philippepascal |
| 2026-05-22T23:58Z | in_design | specd | claude-0522-1400-b7f2 |
| 2026-05-23T02:58Z | specd | ready | philippepascal |
| 2026-05-23T02:58Z | ready | in_progress | philippepascal |
