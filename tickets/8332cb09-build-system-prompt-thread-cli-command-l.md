+++
id = "8332cb09"
title = "build_system_prompt: thread CLI command list into Layer 3 of worker prompt"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8332cb09-build-system-prompt-thread-cli-command-l"
created_at = "2026-05-31T02:58:36.745209Z"
updated_at = "2026-06-01T01:17:33.856639Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
depends_on = ["9c66e199"]
+++

## Spec

### Problem

`build_system_prompt` in `apm-core/src/start.rs` calls `instructions::generate(root, Some(role), ticket_id, &[])` with an empty commands slice. `command_reference_body` returns an empty string when commands is empty, so `generate` skips the `## Command Reference` block entirely. Every system prompt produced by `build_system_prompt` — covering all workers dispatched via `apm start`, the `apm work` loop, and the server dispatcher — ends at Session Identity with no command listing. Workers have no indication of which `apm` commands they are permitted to run.

The `apm instructions` CLI command produces the correct output because `apm/src/cmd/instructions.rs` extracts the clap subcommand list and passes it to `generate`. `build_system_prompt` lives in `apm-core`, which intentionally carries no clap dependency, so there is no equivalent extraction there.

After 9c66e199 unifies the worker allow-list to exactly six commands, the set is stable and fully knowable inside `apm-core`. A static const in `instructions.rs` can carry the names and descriptions; `build_system_prompt` converts it and passes it, closing the gap without adding a clap dependency.

### Acceptance criteria

- [x] `apm prompt <id> --system` output contains `## Command Reference` listing all six commands: `apm show`, `apm state`, `apm spec`, `apm set`, `apm new`, `apm instructions`
- [x] `apm instructions <id> --role coder` and `apm prompt <id> --system` produce a `## Command Reference` section containing the same six command names for the same ticket
- [x] A unit test in `apm-core/src/start.rs` asserts that `build_system_prompt` output contains `## Command Reference` with at least `apm show` and `apm instructions` present
- [x] All existing `build_system_prompt` layer-composition tests pass after updating their expected-string helpers to use `WORKER_COMMANDS` instead of `&[]`
- [ ] `cargo test --workspace` passes

### Out of scope

- Per-role allow-list changes — handled by 9c66e199
- Changes to `apm instructions` CLI output — it already works and is unchanged
- Changing the signature of `instructions::generate` or `build_system_prompt`
- Updates to `apm --help` or `--help` output on any subcommand
- Any `apm-server` route changes beyond what is fixed by `build_system_prompt` passing the const
- Documentation outside of inline code comments

### Approach

This ticket assumes 9c66e199 has landed on the target branch (`epic/9c3c4c20-…`). Rebase before implementing.

#### 1. Add `WORKER_COMMANDS` const — `apm-core/src/instructions.rs`

This ticket defines a separate const from 9c66e199's `WORKER_COMMAND_ALLOWLIST`. The two consts serve different purposes and must coexist:

- `WORKER_COMMAND_ALLOWLIST: &[&str]` (landed by 9c66e199) — used by `role_command_allowlist` to filter the CLI-extracted command list at runtime.
- `WORKER_COMMANDS: &[(&str, &str)]` (this ticket) — used by `build_system_prompt` to render the Command Reference when no CLI is present.

Both consts are `pub(crate)` in `instructions.rs`. The six command names in `WORKER_COMMANDS` must exactly match those in `WORKER_COMMAND_ALLOWLIST`. A comment in each cross-references the other. No merge into a single const is attempted because the two types (`&[&str]` vs `&[(&str, &str)]`) serve different call sites and a `const fn` bridge is not worth the complexity.

Descriptions are purpose-built for agent consumption and are **not** synced from clap `///` doc comments. `apm-core` carries no clap dependency and should not grow one. If a subcommand's fundamental purpose changes, update both `WORKER_COMMANDS` here and the clap string in `apm/src/main.rs` in the same commit. A comment in the const documents this rule.

Add `WORKER_COMMANDS` immediately after `WORKER_COMMAND_ALLOWLIST` (or after the existing `static` declarations if 9c66e199 has not yet landed):

```rust
/// Name + description tuples for the six worker-permitted `apm` commands.
/// Names must stay in sync with WORKER_COMMAND_ALLOWLIST (ticket 9c66e199).
/// Descriptions are purpose-built for agent consumption; they are NOT copied
/// from clap `///` doc comments. If a subcommand's fundamental purpose changes,
/// update both this const and the clap string in apm/src/main.rs in the same commit.
pub(crate) const WORKER_COMMANDS: &[(&str, &str)] = &[
    ("instructions", "Output APM system knowledge for agents: state machine, ticket format, shell discipline, session identity, and command reference"),
    ("new",          "Create a new ticket"),
    ("set",          "Set a field on a ticket"),
    ("show",         "Show a ticket"),
    ("spec",         "Read or write individual spec sections of a ticket"),
    ("state",        "Transition a ticket's state"),
];
```

Alphabetical order matches `extract_commands`'s sort so the CLI and worker-spawn paths produce sections with identical ordering.

#### 2. Pass the const in `build_system_prompt` — `apm-core/src/start.rs`

At line 974, replace:

```rust
let instructions_layer = crate::instructions::generate(root, Some(role), ticket_id, &[])?;
```

with:

```rust
let cmds: Vec<(String, String)> = crate::instructions::WORKER_COMMANDS
    .iter()
    .map(|(n, a)| (n.to_string(), a.to_string()))
    .collect();
let instructions_layer = crate::instructions::generate(root, Some(role), ticket_id, &cmds)?;
```

No signature change to `build_system_prompt` or `generate` is required. All production call sites reach this path (lines 488, 656, 829 in `start.rs` and lines 104, 147, 207 in `prompt.rs`), so fixing this one call site fixes all of them.

#### 3. Update layer-composition tests — `apm-core/src/start.rs`

Five tests call `crate::instructions::generate(p, Some("coder"), None, &[])` to build an expected string that is compared against `build_system_prompt` output. After step 2 the production call passes `WORKER_COMMANDS` while these tests pass `&[]`, causing them to diverge.

Affected call sites (all in the `start.rs` test module):
- Line 1223 — `agents_instructions_prepended_with_blank_line`
- Line 1238 — `agents_instructions_none_is_no_op`
- Line 1253 — `agents_instructions_empty_path_is_no_op`
- Line 1284 — `agents_instructions_trailing_whitespace_trimmed`
- Line 1304 — `project_file_in_layer2`

For each, replace `&[]` with:

```rust
let cmds: Vec<(String, String)> = crate::instructions::WORKER_COMMANDS
    .iter()
    .map(|(n, a)| (n.to_string(), a.to_string()))
    .collect();
// pass &cmds where &[] was used
```

Three call sites in `instructions.rs` tests (lines 467, 506, 685) pass `&[]` intentionally — they test ID substitution, state-machine table format, and live-config filtering respectively, none of which depend on the command list. Leave them unchanged.

#### 4. Add a new test — `apm-core/src/start.rs`

```rust
#[test]
fn build_system_prompt_contains_command_reference() {
    let dir = tempfile::tempdir().unwrap();
    let result = build_system_prompt(dir.path(), None, "claude", "coder", None).unwrap();
    let cr_pos = result.find("## Command Reference")
        .expect("## Command Reference section missing from worker prompt");
    let cr = &result[cr_pos..];
    assert!(cr.contains("apm show"), "apm show missing from Command Reference");
    assert!(cr.contains("apm instructions"), "apm instructions missing from Command Reference");
}
```

#### 5. Verify no changes needed to the CLI path

`apm/src/cmd/instructions.rs` already passes a non-empty `commands` vec extracted from clap. After 9c66e199 lands, `role_command_allowlist` filters that list to the same six commands whose names appear in `WORKER_COMMANDS`. No edits required.

#### 6. `instructions.rs` test hygiene

If 9c66e199's `sample_commands()` update (adding `"instructions"`) has not yet landed, add it here so assertions on the six-command set pass:

```rust
("instructions".to_string(), "Output APM system knowledge…".to_string()),
```

### Open questions


### Amendment requests

- [x] Reconcile const naming and shape with 9c66e199. The two specs currently propose different shapes — 9c66e199 defines an allow-list of command names; this ticket needs name plus description tuples for the Command Reference section. Clarify whether this ticket reuses 9c66e199's const, defines its own const alongside it, or extends 9c66e199's const with descriptions. Pick one approach and document it.
- [x] Pin the description source for the unified command list. Either mirror the clap long_about strings from main.rs (single source of truth, but the apm-core const must be kept in sync manually), or maintain shorter purpose-built descriptions in apm-core (decoupled but two sources). Pick one and document the synchronisation strategy.
- [x] Grep for every call site of build_system_prompt and of instructions::generate with an empty commands slice. The spec mentions four tests that pass empty slices but should verify the list is exhaustive; missing call sites would silently produce empty command references for those code paths.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:58Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:41Z | groomed | in_design | philippepascal |
| 2026-05-31T07:46Z | in_design | specd | claude |
| 2026-05-31T19:36Z | specd | ammend | philippepascal |
| 2026-05-31T20:07Z | ammend | in_design | philippepascal |
| 2026-05-31T20:11Z | in_design | specd | claude |
| 2026-05-31T21:04Z | specd | ready | philippepascal |
| 2026-06-01T01:17Z | ready | in_progress | philippepascal |