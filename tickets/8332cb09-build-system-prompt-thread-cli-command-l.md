+++
id = "8332cb09"
title = "build_system_prompt: thread CLI command list into Layer 3 of worker prompt"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8332cb09-build-system-prompt-thread-cli-command-l"
created_at = "2026-05-31T02:58:36.745209Z"
updated_at = "2026-05-31T07:41:47.983826Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:58Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:41Z | groomed | in_design | philippepascal |