+++
id = "9c66e199"
title = "Unify worker command allow-list to six commands; remove per-role lists"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9c66e199-unify-worker-command-allow-list-to-six-c"
created_at = "2026-05-31T02:57:57.400665Z"
updated_at = "2026-05-31T07:36:19.800933Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
+++

## Spec

### Problem

The `role_command_allowlist` function in `apm-core/src/instructions.rs` hard-codes two named roles — `"spec-writer"` and `"worker"` — each with its own 8-command list. Any role not matching those two strings (including `"coder"`) falls through to `None` and receives the full, unfiltered command list of 30+ entries. A coder agent today sees supervisor commands like `apm sync`, `apm list`, `apm next`, and `apm start` in its Command Reference — noise at best, misleading at worst.

Beyond the coder gap, the design is fragile: adding a new role requires editing match arms in code. Per project convention, role names belong in config, not in code. Both role lists also include supervisor-tier commands (`sync`, `list`, `next`) that workers should never invoke.

The fix is to replace the per-role match arms with a single `WORKER_COMMAND_ALLOWLIST` constant returned for any supplied role. The six commands — `show`, `state`, `spec`, `set`, `new`, `instructions` — cover everything a worker needs: reading the ticket, transitioning state, editing the spec, adjusting fields, filing side-notes, and bootstrapping the session. No role-name strings belong in the filtering logic after this change.

### Acceptance criteria

- [ ] `apm instructions --role coder` Command Reference lists exactly the six commands: show, state, spec, set, new, instructions
- [ ] `apm instructions --role spec-writer` Command Reference lists the same six commands as coder
- [ ] `apm instructions --role any-unknown-role` Command Reference lists the same six commands
- [ ] `apm instructions` (no role) Command Reference remains unfiltered
- [ ] The literal strings `"spec-writer"` and `"worker"` do not appear in the command-filtering logic of `instructions.rs`
- [ ] `cargo test --workspace` passes with updated test assertions
- [ ] `apm.coder.md` (default and project copy) Permitted commands section lists: show, state, spec, set, new, instructions
- [ ] `apm.spec-writer.md` (default and project copy) Permitted commands section lists: show, state, spec, set, new, instructions

### Out of scope

- Workflow schema changes (covered by other tickets in the epic)
- The `build_system_prompt` empty-commands bug (separate ticket)
- Help text sweep for other `apm` subcommands (separate ticket)
- The hardcoded `"claude/coder"` fallback in `start.rs` (separate ticket covering `workers.default`)
- Changing which transitions appear in the State Machine section — unrelated to command filtering

### Approach

#### `apm-core/src/instructions.rs`

1. Add a constant after the existing `static` declarations near the top of the file:
   ```rust
   const WORKER_COMMAND_ALLOWLIST: &[&str] = &["show", "state", "spec", "set", "new", "instructions"];
   ```

2. Replace the body of `role_command_allowlist` — keep the signature unchanged so the call site in `command_reference_body` requires no edits:
   ```rust
   fn role_command_allowlist(_role: &str) -> Option<&'static [&'static str]> {
       Some(WORKER_COMMAND_ALLOWLIST)
   }
   ```

3. Update `sample_commands()` in the test module to include `("instructions", "Print APM system knowledge".to_string())` so assertions on that command entry are possible.

4. Rewrite the three tests whose assertions no longer hold:
   - `generate_worker_scopes_commands`: assert the six unified commands are present; assert `apm start`, `apm sync`, `apm prompt` are absent.
   - `generate_spec_writer_scopes_commands`: assert the same six commands; assert `apm start` is absent.
   - `generate_unknown_role_falls_back_to_full_commands`: rename and rewrite — unknown roles now receive the allowlist, not the full set; assert the six commands appear and `apm prompt` does not.

#### `apm-core/src/default/agents/claude/apm.coder.md`

Replace the "Permitted `apm` commands" block (currently four bullets) with:

```
**Permitted `apm` commands:**
- `apm show` — read a ticket
- `apm state` — transition ticket state
- `apm spec` — read or write spec sections
- `apm set` — set a field on a ticket
- `apm new` — file a side-note ticket
- `apm instructions` — load APM system knowledge
```

#### `apm-core/src/default/agents/claude/apm.spec-writer.md`

The file has no "Permitted `apm` commands" section. Add a `## Scope limits` section immediately before `## How to save spec sections`:

```markdown
## Scope limits

**Permitted `apm` commands:**
- `apm show` — read a ticket
- `apm state` — transition ticket state
- `apm spec` — read or write spec sections
- `apm set` — set a field on a ticket
- `apm new` — file a side-note ticket
- `apm instructions` — load APM system knowledge
```

#### `.apm/agents/claude/apm.coder.md` and `.apm/agents/claude/apm.spec-writer.md`

Apply the same edits as above to the project copies. These files mirror the defaults and must stay in sync.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-31T02:57Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
| 2026-05-31T07:33Z | groomed | in_design | philippepascal |
| 2026-05-31T07:36Z | in_design | specd | claude |
