+++
id = "d486d183"
title = "apm help config: render config.toml schema from Config struct"
state = "in_design"
priority = 0
effort = 4
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d486d183-apm-help-config-render-config-toml-schem"
created_at = "2026-04-28T19:27:57.393396Z"
updated_at = "2026-04-28T20:22:08.772662Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
depends_on = ["bc89e0a0", "069c3403"]
+++

## Spec

### Problem

The `render_config()` function in `apm/src/cmd/help.rs` is introduced as a stub by ticket bc89e0a0. It returns a placeholder string referencing this ticket (d486d183). As a result, `apm help config` gives users no actionable information about what fields are valid in `.apm/config.toml`, their types, defaults, or purpose.

The `Config` struct in `apm-core/src/config.rs` already defines all config keys and their types, but nearly every field lacks a `/// doc comment`. Ticket 069c3403 provides `apm_core::help_schema::schema_entries::<T>()` and `render_schema::<T>()`, which convert any `JsonSchema`-derived struct into a formatted field listing including type, default, and description drawn from doc comments.

This ticket wires those two pieces together: add one-line doc comments to every user-facing field in the `Config` struct tree, then replace the `render_config()` stub to call into the help_schema infrastructure and format output grouped by TOML section header.

### Acceptance criteria

- [ ] `apm help config` exits 0 and prints non-empty output to stdout
- [ ] The placeholder string referencing ticket d486d183 no longer appears in `apm help config` output
- [ ] Output contains a recognisable header or path prefix for each top-level section: `project`, `tickets`, `worktrees`, `git_host`, `agents`, `sync`, `logging`, `workers`
- [ ] Output contains a line for `project.name` that is marked as required (no default shown)
- [ ] Output contains a line for `project.default_branch` with default `main`
- [ ] Output contains a line for `agents.max_concurrent` with default `3`
- [ ] Output contains a line for `agents.max_workers_per_epic` with default `1`
- [ ] Output contains a line for `workers.command` with default `claude`
- [ ] Every field line that has a doc comment shows a non-empty description
- [ ] `worker_profiles` appears in the output with a description explaining it is a map of named profiles

### Out of scope

- Content for `render_workflow()`, `render_ticket()`, `render_commands()` — those are sibling tickets 7ba021e8, 14214305, and 3665e017
- ANSI colour or markdown rendering in the output
- Pager integration (no `less`/`more` invocation)
- Per-user-defined `worker_profiles` key documentation — only a generic map description is shown; individual profiles are user-defined and unknowable at build time
- `LocalConfig` and `LocalWorkersOverride` structs — internal override file, not user-facing `config.toml`
- `WorkflowConfig`, `StateConfig`, `TransitionConfig`, and their nested types — covered by ticket 7ba021e8
- `TicketConfig` and `TicketSection` — covered by ticket 14214305
- Changes to the `apm help` dispatcher or topic routing — established by ticket bc89e0a0
- Publishing a JSON Schema file as a build artifact

### Approach

**Precondition:** ticket 069c3403 is merged, so `apm_core::help_schema::{schema_entries, FieldEntry}` exist and `JsonSchema` is already derived on `Config` and all nested types.

---

**1. Add `/// doc comments` to `apm-core/src/config.rs`**

Every user-facing field in the `Config` struct tree needs a one-line doc comment so schemars can surface it as a `description`. Fields that already have a comment (e.g. `load_warnings`, `actionable`, `label`, `hint`) need no change. Fields to annotate (one line each):

- `Config`: no field-level comments needed; struct-level doc is optional
- `ProjectConfig`: `name`, `description`, `default_branch`, `collaborators`
- `TicketsConfig`: `dir`, `archive_dir` (skip `sections` — internal use)
- `AgentsConfig`: `max_concurrent`, `max_workers_per_epic`, `max_workers_on_default`, `instructions`, `side_tickets`, `skip_permissions`
- `WorktreesConfig`: `dir`, `agent_dirs`
- `SyncConfig`: `aggressive`
- `LoggingConfig`: `enabled`, `file`
- `GitHostConfig`: `provider`, `repo`, `token_env`
- `WorkersConfig`: `container`, `command`, `args`, `model`, `env`, `keychain`
- `WorkerProfileConfig`: `command`, `args`, `model`, `env`, `container`, `instructions`, `role_prefix`
- `WorkConfig`: `epic`
- `ServerConfig`: `origin`, `url`
- `ContextConfig`: `epic_sibling_cap`, `epic_byte_cap`
- `PrioritizationConfig`: `priority_weight`, `effort_weight`, `risk_weight`

Doc comments must be `/// one sentence.` placed immediately above the field. Do not change field types, serde attributes, or defaults.

---

**2. Replace the stub in `apm/src/cmd/help.rs`**

Replace `render_config()` (currently returns a placeholder string) with a function that:

1. Calls `apm_core::help_schema::schema_entries::<apm_core::config::Config>()` to get `Vec<FieldEntry>`.
2. Groups entries by their first path segment (the part before the first `.` or `[`). Preserve the order in which each new prefix is first encountered (struct field order flows through schemars).
3. For each group, print a `[section]` header line (e.g. `[project]`), then one line per entry using the same column-aligned format as `render_schema` from 069c3403:
   ```
   <toml_path>  <type>  [default: <val>]  # <description>
   ```
   Omit `[default: ...]` when `entry.default` is `None`; omit `# ...` when `entry.description` is `None`.
4. For `worker_profiles`, emit the section header `[worker_profiles.<name>]` and a single descriptive note explaining that each key is a user-defined named profile whose fields mirror `[workers]`, followed by the fields from `WorkerProfileConfig` with paths like `worker_profiles[].command`.
5. Return the resulting `String`; the caller in `run()` prints it to stdout.

Column widths: compute the maximum width of `toml_path` and `type` across all entries in the group, then pad with spaces. This keeps output readable without requiring a terminal-width query.

---

**3. Imports and crate wiring**

`apm/src/cmd/help.rs` already imports `apm_core` (it will after bc89e0a0). Add:
```rust
use apm_core::help_schema::schema_entries;
use apm_core::config::Config;
```

No `Cargo.toml` changes needed — `apm-core` is already a workspace dependency of `apm`.

---

**Implementation order:**

1. Add doc comments to `apm-core/src/config.rs` (bulk of the work; safe — comments only)
2. Replace `render_config()` stub in `apm/src/cmd/help.rs`
3. `cargo build -p apm` — confirm it compiles
4. `apm help config` — manually verify output contains section headers and all AC field lines

### Open questions


### Amendment requests

- [ ] `worker_profiles` rendering is contradictory between AC and Approach. AC describes it as "a map of named profiles" but Approach uses array notation `worker_profiles[].command`. The actual struct is `HashMap<String, WorkerProfileConfig>`, which is a map, not an array. Reconcile to map notation: render as `worker_profiles.<name>.command` (matching the TOML form `[worker_profiles.spec_agent]`). Update the AC and Approach together so they agree.
- [ ] Field name verification not done. The spec lists ~30 fields to add doc comments to, but doesn't reference current line numbers in `apm-core/src/config.rs`. Before implementing: pull the current struct definitions, walk every field listed in the spec, and confirm the field exists with the stated type and default. Some specifics that may have drifted: `workers.command` defaulting to `claude`, the exact field set on `AgentsConfig`, and any fields added after this spec was written (e.g., `max_workers_on_default` from ticket 07d51d55).
- [ ] After field verification, update the AC list to reference the verified field names. Remove any ACs that point at fields that don't exist in the current struct, and add ACs for any fields that exist but weren't in the original list.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T19:27Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:49Z | groomed | in_design | philippepascal |
| 2026-04-28T19:52Z | in_design | specd | claude-0428-1949-a538 |
| 2026-04-28T20:17Z | specd | ammend | philippepascal |
| 2026-04-28T20:22Z | ammend | in_design | philippepascal |
