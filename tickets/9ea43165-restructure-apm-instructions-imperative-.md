+++
id = "9ea43165"
title = "Restructure apm instructions: imperative format, role filtering, ticket-id substitution, layer reorder"
state = "ready"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9ea43165-restructure-apm-instructions-imperative-"
created_at = "2026-05-30T16:49:25.808040Z"
updated_at = "2026-05-30T18:09:17.269074Z"
depends_on = ["48d3932b"]
+++

## Spec

### Problem

GOAL: make apm instructions output dramatically easier for an LLM worker to consume by (a) replacing descriptive prose with imperative tables and concrete commands, (b) filtering to only what the role can act on, (c) substituting concrete ticket IDs where currently the output uses <id> placeholders, and (d) reordering the three prompt layers so role-specific rules sit at the highest-attention position.

This ticket depends on a3c34ddc (shell-discipline relocation). It assumes a3c34ddc has landed so the apm instructions output is already shorter and agent-agnostic before this restructure begins.

PROBLEM: the current apm instructions output is descriptive, role-undifferentiated, and uses placeholder strings the worker cannot copy-paste. Example today:

  ### Ready (ready)
  Actionable by: agent
    → in_progress, trigger: command:start, role: coder

A worker has to mentally parse the state-machine grammar and re-substitute the placeholders. The actionable form is much shorter and more directly usable:

  Ready → in_progress     apm state abc12345 in_progress

REQUIREMENTS:

1) IMPERATIVE FORMAT for the state machine section. Replace the verbose per-state block with a compact transitions table: from-state → to-state, then the exact apm command. One row per allowed transition. Drop the trigger/role columns (they are not actionable by the worker). Keep enough information that the worker knows WHY each transition exists (a one-line gloss is fine) but the column layout should make the apm command unmistakable.

2) ROLE FILTERING. When apm instructions is invoked with a role (the dispatcher always knows the role), only emit transitions actionable by that role and only commands the role is permitted to invoke. A coder worker should not see spec-writer transitions, and vice versa. Same treatment for the command-reference section: filter to the role's permitted command set. The role is already passed to the helpers; this is a render-time filter.

3) TICKET-ID SUBSTITUTION. apm instructions accepts an optional ticket id argument (e.g. apm instructions abc12345 --role coder). When present, every occurrence of the literal placeholder <id> in the rendered output is substituted with the actual ticket id, so commands like apm state <id> in_progress render as apm state abc12345 in_progress. The dispatch path in apm-core/src/start.rs (build_system_prompt and its call sites) must be updated to pass the ticket id when invoking the instructions helper so workers receive the substituted form. Without a ticket id, the output keeps the <id> literal so humans inspecting the instructions still see a usable template.

4) NO-ROLE LISTING. apm instructions without a role argument prints a short index of available roles, each with a one-line description, mirroring what apm prompt already surfaces. Same shared helper underneath if possible.

5) LAYER REORDER. In apm-core/src/start.rs::build_system_prompt, swap the layer order from L1 (apm instructions) -> L2 (.apm/project.md) -> L3 (role file) to L3 -> L2 -> L1. Rationale: after a3c34ddc, the role file carries the rules a worker needs on every tool call (shell discipline, the don'ts), while apm instructions becomes purely reference material (state machine, command reference). Putting the role file first places the highest-frequency rules in the highest-attention position of the prompt. apm prompt --explain output should reflect the new order.

OUT OF SCOPE:
- The shell-discipline content (covered by a3c34ddc).
- The cascade that resolves the role file (unchanged).
- apm-server / apm-ui display of instructions content.
- New placeholder substitutions beyond <id> (the worker mostly needs the ticket id; other dynamic values can be added later if a need surfaces).
- Per-ticket personalization beyond <id> substitution (no insertion of frontmatter fields, history, etc.).

TESTS:
- apm instructions --role coder against the default workflow produces a transitions table whose rows are ONLY transitions a coder can act on; spec-writer transitions are absent.
- apm instructions abc12345 --role coder produces output where no occurrence of the literal <id> remains; every command line contains abc12345 instead.
- apm instructions (no args) prints a roles index that includes coder, spec-writer, main-agent and a one-line gloss for each.
- A snapshot or stable-assertion test for the imperative format output to guard against accidental reversion to the descriptive form.
- The build_system_prompt unit tests assert the new order (role file content appears before project content which appears before apm instructions content).
- apm prompt --explain reflects the new layer order in its labelling (layer 1 → role file, layer 2 → project, layer 3 → apm instructions). Update any tests asserting layer labels.
- A dispatch integration test that spawns a worker (or stops short of spawning) and asserts the ticket id appears substituted in the system prompt sent to the worker.

### Acceptance criteria

- [ ] `apm instructions --role coder` emits a Markdown table under `## State Machine` with rows only for transitions the coder role can act on; rows for `in_design`, `groomed`, and `specd` are absent from the table.
- [ ] `apm instructions --role spec-writer` emits a Markdown table with no rows for `in_progress` or `implemented`.
- [ ] The state machine table produced for any role has a header row with `From`, `To`, and `Command` columns; each data row contains the exact runnable apm command (`apm start <id>` for `command:start` transitions, `apm state <id> <to>` for all others).
- [ ] `apm instructions abc12345 --role coder` produces output containing no occurrence of the literal string `<id>`; every `Command` cell contains `abc12345`.
- [ ] `apm instructions` (no arguments) prints a role index that includes `coder`, `spec-writer`, and `main-agent`, each with a one-line description; the output does not contain a `## State Machine` section.
- [ ] `build_system_prompt` called with `ticket_id = Some("abc12345")` returns a string where no `<id>` literal appears in the instructions layer and `abc12345` appears in every former `<id>` position.
- [ ] `build_system_prompt` called with both a project file and a ticket id returns output in which the role-file content appears at an earlier character position than the project content, which appears before the instructions content.
- [ ] `apm prompt --explain` shows "layer 1:" identifying the role file and "layer 3:" identifying `apm instructions`; these labels are the reverse of the previous order.
- [ ] `cargo test --workspace` passes after all test additions and updates.

### Out of scope

- Shell-discipline content movement (covered by a3c34ddc).
- The cascade that resolves which role file to use (unchanged).
- apm-server / apm-ui display of instructions content.
- New placeholder substitutions beyond `<id>` (e.g., no `<branch>`, no frontmatter field insertion).
- Per-ticket personalization beyond `<id>` substitution (no history insertion, no dependency injection into the instructions layer).
- Changing the command-reference section format (it remains a flat aligned list, not a table).
- The `apm prompt` command itself beyond updating `--explain` layer labels and passing `ticket_id` through `build_system_prompt`.

### Approach

All five requirements are implemented in parallel (no ordering dependency between them). Six files change; call sites in `prompt.rs` and `start.rs` propagate the new `ticket_id` param.

#### 1. Imperative transitions table (`apm-core/src/instructions.rs`)

Replace `format_live_state_machine()` body with a Markdown table renderer:

- Emit a `| From | To | Command |` header and separator row, then iterate over every `(state, transition)` pair.
- For each transition: if `role = Some(r)`, skip unless `derive_transition_role(t) == r`.
- Command cell: emit `apm start <id>` when `t.trigger == "command:start"`, else `apm state <id> <t.to>`.
- Drop all per-state prose (heading, description, "Actionable by:" lines). The table is the only output.
- Remove the `filter` `HashSet` that was used to include target states in the block output — with the table format, only source-state rows are emitted; target-state visibility is automatic from the `To` column.
- Update `STATIC_STATE_MACHINE` constant to the same three-column table format using the same role-filtering rules.

#### 2. Ticket-id substitution (`instructions.rs`, `start.rs`, `cmd/instructions.rs`, `main.rs`)

- Add `ticket_id: Option<&str>` after `role` in `generate()`. After building `out`, if `ticket_id.is_some()` perform `out = out.replace("<id>", ticket_id.unwrap())`.
- Add `ticket_id: Option<&str>` to `build_system_prompt(root, project_file, agent, role, ticket_id)`; pass it through to `generate()`.
- In `start.rs`: every call to `build_system_prompt` (in `run()`, `run_next()`, `spawn_next_worker()`) passes `Some(&id)`.
- In `prompt.rs`: calls to `build_system_prompt` pass `Some(&ticket_id)` when the ticket id is known (the `run`, `run_full`, `run_message` functions), and `None` for `run_without_ticket` and `explain_without_ticket`.
- In `main.rs::Instructions`: add `ticket_id: Option<String>` as a positional argument before `--role`. Pass `ticket_id.as_deref()` to `cmd::instructions::run()`.
- In `cmd/instructions.rs::run()`: add `ticket_id: Option<&str>` param; pass to `generate()`.

#### 3. No-role listing (`instructions.rs`)

Add a guard at the top of `generate()`: when `role.is_none()`, return `role_index_body(root, config.as_ref())` immediately (skip all other sections).

New function `role_index_body(root: &Path, config: Option<&Config>) -> String`:
- Emits a `## Available Roles` section with a two-column list.
- Hardcoded entries: `coder` → "Implements tickets in a git worktree", `spec-writer` → "Writes and revises ticket specs", `main-agent` → "Project management companion for the supervisor".
- Scan `.apm/agents/` for `apm.<role>.md` files; add any role names not already in the hardcoded set with description "(custom role)".

#### 4. Layer reorder (`start.rs`, `prompt.rs`)

In `build_system_prompt()`:

- Rename locals for clarity: `instructions_layer` (was `layer1`), `project_layer` (was `layer2`), `role_layer` (was `layer3`).
- New compose order: `role_layer` → `project_layer` → `instructions_layer`.

In `prompt.rs::format_provenance()`:

- "layer 1:" now labels the role file: emit `prov.winner.source` and its level/label.
- "layer 3:" now labels `apm instructions (dynamic, role: ...)`.
- Rename `PromptProvenance.layer1_role` → `PromptProvenance.instructions_role`; update `explain_system_prompt()` and `format_provenance()` accordingly.

#### 5. Tests

**`apm-core/src/instructions.rs`** — add `ticket_id` (as `None`) to all existing `generate()` calls:
- Replace `generate_no_role_contains_all_sections` → `generate_no_role_lists_roles`: `generate(tmp, None, None, &[])` asserts output contains "coder" and "spec-writer", does not contain `## State Machine`.
- Replace `generate_no_role_sections_in_order` → `generate_role_table_precedes_command_reference`: `generate(tmp, Some("worker"), None, &sample_commands())` asserts `## State Machine` position < `## Command Reference` position.
- Update `generate_role_independent_sections`: remove Shell Discipline assertions (handled by a3c34ddc); assert `| From | To | Command |` present.
- Update `live_ticket_format_from_config`: use `role = Some("worker")` (no-role now returns role index).
- Add `generate_with_id_no_placeholder_remains`: `generate(tmp, Some("worker"), Some("abc12345"), &[])` asserts `!out.contains("<id>")` and `out.contains("abc12345")`.
- Add `imperative_table_format_header`: live config + `role = Some("coder")` asserts `## State Machine` section contains `| From | To | Command |`.

**`apm-core/src/start.rs`** — add `ticket_id = None` to all existing `build_system_prompt()` calls; update `expected` strings from `L1 + L2 + L3` to `L3 + L2 + L1` in tests that assert exact output:
- Add `build_system_prompt_layer_order`: write a role file ("ROLE CONTENT") and project file ("PROJECT CONTENT"), call with `ticket_id = None`; assert `role_pos < project_pos < instructions_pos` by `str::find`.
- Add `build_system_prompt_ticket_id_substituted`: call with `ticket_id = Some("abc12345")`, no project file; assert `out.contains("abc12345")` and `!out.contains("<id>")`.

**`apm-core/src/prompt.rs`** — add `ticket_id` to `build_system_prompt` call sites:
- Add `explain_role_file_is_layer1`: call `explain()`, assert the line starting "layer 1:" contains the role file path, not "apm instructions".
- Add `explain_instructions_is_layer3`: assert the line starting "layer 3:" contains "apm instructions".
- Update `parity_build_system_prompt_matches_prompt_run` to pass `ticket_id`.

**`apm/src/cmd/instructions.rs`** — update `run()` signature; update tests:
- `generate_contains_all_sections`: rewrite to assert role index is returned when `role = None`.
- Add `ticket_id = None` to all `generate()` calls in existing tests.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-30T16:49Z | — | new | philippepascal |
| 2026-05-30T17:08Z | new | groomed | philippepascal |
| 2026-05-30T17:21Z | groomed | in_design | philippepascal |
| 2026-05-30T17:29Z | in_design | specd | claude |
| 2026-05-30T18:09Z | specd | ready | philippepascal |
