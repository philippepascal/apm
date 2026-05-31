+++
id = "8332cb09"
title = "build_system_prompt: thread CLI command list into Layer 3 of worker prompt"
state = "specd"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8332cb09-build-system-prompt-thread-cli-command-l"
created_at = "2026-05-31T02:58:36.745209Z"
updated_at = "2026-05-31T07:46:11.277683Z"
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

- [ ] `apm prompt <id> --system` output contains `## Command Reference` listing all six commands: `apm show`, `apm state`, `apm spec`, `apm set`, `apm new`, `apm instructions`
- [ ] `apm instructions <id> --role coder` and `apm prompt <id> --system` produce a `## Command Reference` section containing the same six command names for the same ticket
- [ ] A unit test in `apm-core/src/start.rs` asserts that `build_system_prompt` output contains `## Command Reference` with at least `apm show` and `apm instructions` present
- [ ] All existing `build_system_prompt` layer-composition tests pass after updating their expected-string helpers to use `WORKER_COMMANDS` instead of `&[]`
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

Add a `pub(crate)` const immediately after the existing `static` declarations (around line 88):

```rust
pub(crate) const WORKER_COMMANDS: &[(&str, &str)] = &[
    ("instructions", "Output APM system knowledge for agents: state machine, ticket format, shell discipline, session identity, and command reference"),
    ("new",          "Create a new ticket"),
    ("set",          "Set a field on a ticket"),
    ("show",         "Show a ticket"),
    ("spec",         "Read or write individual spec sections of a ticket"),
    ("state",        "Transition a ticket's state"),
];
```

The descriptions are sourced verbatim from the clap `///` doc comments in `apm/src/main.rs` for the six matching subcommands. Alphabetical order matches `extract_commands`'s sort, so the CLI and worker-spawn paths produce identical sections.

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

No signature change to `build_system_prompt` or `generate` is required.

#### 3. Update layer-composition tests — `apm-core/src/start.rs`

Four tests construct an expected string by calling `crate::instructions::generate(p, Some("coder"), None, &[])` and asserting `result == expected`. After step 2 they will diverge. Fix each by replacing `&[]` with the converted const:

```rust
let cmds: Vec<(String, String)> = crate::instructions::WORKER_COMMANDS
    .iter()
    .map(|(n, a)| (n.to_string(), a.to_string()))
    .collect();
let instructions_layer = crate::instructions::generate(p, Some("coder"), None, &cmds).unwrap();
```

Affected tests: `agents_instructions_prepended_with_blank_line`, `agents_instructions_none_is_no_op`, `agents_instructions_empty_path_is_no_op`, `agents_instructions_trailing_whitespace_trimmed`.

Also check `project_file_in_layer2` and any other tests that call `generate` with `&[]` inside a `build_system_prompt` comparison; apply the same fix.

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

`apm/src/cmd/instructions.rs` extracts all clap commands, `role_command_allowlist` (after 9c66e199) filters to the six-command list. The output will match what `build_system_prompt` now produces via `WORKER_COMMANDS`. No edits required.

#### 6. `instructions.rs` test hygiene

If 9c66e199's approach for `sample_commands()` (adding `"instructions"`) has not yet landed, add it here so tests that assert on the six-command output pass:

```rust
("instructions".to_string(), "Output APM system knowledge…".to_string()),
```

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:58Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:41Z | groomed | in_design | philippepascal |
| 2026-05-31T07:46Z | in_design | specd | claude |
