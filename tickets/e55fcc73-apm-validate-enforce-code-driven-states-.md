+++
id = "e55fcc73"
title = "apm validate: enforce code-driven states are declared in workflow.toml"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e55fcc73-apm-validate-enforce-code-driven-states-"
created_at = "2026-04-28T22:42:06.291026Z"
updated_at = "2026-04-28T22:48:19.267181Z"
depends_on = ["50649e84"]
+++

## Spec

### Problem

`apm-core/src/state.rs` hard-codes `state = "merge_failed"` when an attempted merge fails during the `in_progress → implemented` transition (lines 161–184). This write bypasses the state machine entirely — `workflow.toml` is never consulted. As a result a ticket can land in a state that the project's `workflow.toml` does not declare: no transitions are defined for it, `apm state` cannot move the ticket out, and it is visible only via `apm list`.

Ticket 63f5e6d2 hit this exactly: it ended up in `merge_failed` on a project initialised before commit `a7bce26b` (the commit that added `merge_failed` to the default template). The only escape was a manual `workflow.toml` edit.

The fix has two parts:

**1. `apm validate` enforces that every state the code can write is declared in `workflow.toml`.**
A small registry — `SYSTEM_STATES` in `apm-core/src/state.rs` — lists every state value the code may write directly (currently just `"merge_failed"`). `apm validate` walks this list against the loaded config; any registered state absent from `workflow.toml` is reported as a config error. Because this is a config-level check it runs even under `--config-only`.

**2. `apm validate --fix` ports missing states from the embedded default template.**
For each missing state, the fix locates the corresponding `[[workflow.states]]` block in the default `workflow.toml` (shipped inside the binary via `include_str!`) and appends it verbatim to the project's `.apm/workflow.toml`. The operation is idempotent. If the default template itself has no block for a registered state (i.e. `SYSTEM_STATES` and the template have drifted), the fix reports an error and exits non-zero rather than silently skipping.

The existing hash-trip on config-file changes surfaces this check automatically on the next mutating command. Tying re-validation to the binary version (so a binary upgrade triggers it) is a natural follow-up but is not part of this ticket.

### Acceptance criteria

- [ ] `apm validate` on a project whose `workflow.toml` has no `merge_failed` state exits non-zero and prints an error that names `merge_failed` and suggests running `apm validate --fix`
- [ ] `apm validate --config-only` also catches the missing system-state (the check is config-level, not ticket-level)
- [ ] `apm validate` on a freshly `apm init`'d project passes the new system-states check without errors
- [ ] `apm validate --fix` on a project missing `merge_failed` appends the `[[workflow.states]]` block from the default template to `.apm/workflow.toml` and prints which states were added
- [ ] `apm validate --fix` re-run on a project that already has `merge_failed` in `workflow.toml` makes no changes (idempotent)
- [ ] `apm validate --fix` when a state in `SYSTEM_STATES` has no corresponding block in the embedded default template reports a clear error and exits non-zero (it does not silently skip the state)
- [ ] `apm validate --json` includes the system-states issue with `kind = "config"`
- [ ] `SYSTEM_STATES` in `apm-core/src/state.rs` contains exactly `["merge_failed"]` with a code comment identifying the function that writes each entry
- [ ] A unit test in `apm-core/src/validate.rs` asserts that every entry in `SYSTEM_STATES` has a matching `[[workflow.states]]` block in the embedded default `workflow.toml`; adding a new entry to `SYSTEM_STATES` without a default-template block causes this test to fail
- [ ] `docs/commands.md` `apm validate` section lists the new check under *Config checks*

### Out of scope

- Recovering ticket 63f5e6d2 specifically (operational; addressed manually)
- Worker-leak / transcript work in ticket 498febe0 (separate concern)
- Binary-version stamp in the hash-trip so a binary upgrade auto-triggers re-validation (acknowledged as a follow-up)
- A general "sync project config from default template" command covering instruction files, `ticket.toml` defaults, or other config sections beyond workflow states
- Adding `"closed"` to `SYSTEM_STATES` — `closed` is the implicit terminal state, intentionally not declared in `workflow.toml`
- Adding `"in_progress"` to `SYSTEM_STATES` — the fallback default in `start.rs` is user-reachable via `apm start`; it is not a code-driven bypass
- Modifying how `merge_failed` is written in `state.rs` (the code path stays unchanged; this ticket only adds the validation safety net)

### Approach

**Step 1 — `apm-core/src/state.rs`: add `SYSTEM_STATES`**

After the `use` block, add:

```rust
/// States the code may write directly without user action.
/// Every entry here must have a corresponding block in
/// `apm-core/src/default/workflow.toml`.
///
/// - `"merge_failed"` written by `transition()` when a merge error occurs
///   (in_progress → implemented path, ~line 164).
pub const SYSTEM_STATES: &[&str] = &["merge_failed"];
```

No other changes to this file.

**Step 2 — `apm-core/src/validate.rs`: add check + fix helper + canary test**

*Check function* — add after `validate_config`:

```rust
pub fn check_system_states(config: &Config) -> Vec<String> {
    let declared: std::collections::HashSet<&str> =
        config.workflow.states.iter().map(|s| s.id.as_str()).collect();
    crate::state::SYSTEM_STATES
        .iter()
        .filter(|&&s| !declared.contains(s))
        .map(|&s| format!(
            "system state {:?} is not declared in workflow.toml; \
             run `apm validate --fix` to add it from the default template",
            s
        ))
        .collect()
}
```

*Fix helper* — add after `check_system_states`:

```rust
/// Appends the `[[workflow.states]]` block for `state_id` from the embedded
/// default template to `workflow_path`.
/// Returns `Ok(true)` if the block was appended, `Ok(false)` if already
/// present, or an `Err` if `state_id` has no block in the default template.
pub fn port_missing_state(
    workflow_path: &std::path::Path,
    state_id: &str,
    config: &Config,
) -> anyhow::Result<bool> {
    if config.workflow.states.iter().any(|s| s.id == state_id) {
        return Ok(false);
    }
    let default_template = include_str!("default/workflow.toml");
    let block = extract_state_block(default_template, state_id)
        .ok_or_else(|| anyhow::anyhow!(
            "cannot fix: state {:?} is registered in SYSTEM_STATES but has \
             no block in the default workflow.toml template",
            state_id
        ))?;
    let mut existing = std::fs::read_to_string(workflow_path)?;
    if !existing.ends_with('\n') { existing.push('\n'); }
    existing.push('\n');
    existing.push_str(&block);
    std::fs::write(workflow_path, existing)?;
    Ok(true)
}
```

*Block extractor* — private helper in the same file. Walk the default template lines by index (not a streaming iterator, so look-ahead is trivial). Start a capture when a `[[workflow.states]]` header is found whose immediately-following lines contain `id = "merge_failed"` (or whichever state_id). Stop the capture when the next `[[workflow.states]]` header appears or the file ends. Strip trailing blank lines; append a trailing newline.

```rust
fn extract_state_block(toml_text: &str, state_id: &str) -> Option<String> {
    let lines: Vec<&str> = toml_text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() == "[[workflow.states]]" {
            // scan ahead (up to ~5 lines) for the id field
            let header = i;
            let mut found = false;
            for j in (i + 1)..lines.len().min(i + 6) {
                let t = lines[j].trim();
                if t == format!("id = {:?}", state_id)
                    || t == format!("id         = {:?}", state_id)
                {
                    found = true;
                    break;
                }
                if t.starts_with("[[") { break; }
            }
            if found {
                // collect until next [[workflow.states]] or EOF
                let mut end = header + 1;
                while end < lines.len() {
                    if lines[end].trim() == "[[workflow.states]]" { break; }
                    end += 1;
                }
                // strip trailing blank lines
                while end > header && lines[end - 1].trim().is_empty() {
                    end -= 1;
                }
                return Some(lines[header..end].join("\n") + "\n");
            }
        }
        i += 1;
    }
    None
}
```

*Canary test* — add to the `#[cfg(test)]` block:

```rust
#[test]
fn system_states_have_default_blocks() {
    let template = include_str!("default/workflow.toml");
    for &state_id in crate::state::SYSTEM_STATES {
        assert!(
            extract_state_block(template, state_id).is_some(),
            "SYSTEM_STATES contains {:?} but no matching block found in \
             default/workflow.toml — add the block or remove the entry",
            state_id
        );
    }
}
```

**Step 3 — `apm/src/cmd/validate.rs`: wire up check and fix**

*Config-checks block* (after the existing `validate_config` call, not guarded by `!config_only`):

```rust
for msg in apm_core::validate::check_system_states(&ctx.config) {
    issues.push(Issue { kind: "config".into(), subject: "workflow".into(), message: msg });
}
```

*Fix block* (alongside `apply_branch_fixes` / `ensure_gitignore`, also reachable under `--config-only`):

```rust
if fix {
    let workflow_path = root.join(".apm/workflow.toml");
    for &state_id in apm_core::state::SYSTEM_STATES {
        match apm_core::validate::port_missing_state(&workflow_path, state_id, &ctx.config) {
            Ok(true)  => eprintln!("fix: added state {:?} from default template", state_id),
            Ok(false) => {}
            Err(e)    => { eprintln!("error: {e:#}"); /* propagate exit-non-zero */ }
        }
    }
}
```

Use `ctx.config` for the idempotency check inside `port_missing_state` (already loaded; no reload needed).

**Step 4 — `docs/commands.md`**

In the `apm validate` *Config checks* bullet group, append:

> - **System-state declarations** — every state the binary may write directly (currently `merge_failed`) must be declared in `workflow.toml`; `--fix` appends the missing block(s) verbatim from the embedded default template.

Update the `--fix` description to mention: "Adds missing system-state blocks to `workflow.toml`."

**Constraints**

- `port_missing_state` appends only — it must not modify any existing content in `workflow.toml`.
- `port_missing_state` must return an `Err` (not `Ok(false)`) when the default template has no block for the requested state; silently skipping would hide `SYSTEM_STATES`/template drift.
- The fix writes the file but does NOT commit it. The existing hash-trip detects the change on the next mutating command and re-runs validate.
- No behaviour changes to `--json`, `--no-aggressive`, existing validate checks, or the `--config-only` skip of ticket-level checks.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T22:42Z | — | new | philippepascal |
| 2026-04-28T22:44Z | new | groomed | philippepascal |
| 2026-04-28T22:48Z | groomed | in_design | philippepascal |