+++
id = "2e772eab"
title = "Wrapper-contract versioning (APM_WRAPPER_VERSION + manifest.toml)"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2e772eab-wrapper-contract-versioning-apm-wrapper-"
created_at = "2026-04-30T20:05:11.077339Z"
updated_at = "2026-05-01T19:14:15.160859Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95", "2c32a282"]
+++

## Spec

### Problem

Add wrapper-contract versioning so future contract changes (new env vars, new output protocol, etc.) do not silently break user wrappers. v1 is the only contract version this APM build understands.

**Reference spec:** `docs/agent-wrappers.md` — section 'Wrapper-contract versioning'.

**Scope:**
- APM exports `APM_WRAPPER_VERSION=1` to every wrapper invocation (already stamped in ticket d3b93b95; this ticket formalizes the meaning).
- Custom wrappers declare which contract version they target via `manifest.toml.wrapper.contract_version` (already parsed in 2c32a282; this ticket adds the compatibility check).
- Compatibility check at spawn time:
  - Wrapper version == APM version → proceed.
  - Wrapper version < APM version → proceed; APM emits a warning to the worker log noting the wrapper targets an older contract and may not use newer env vars.
  - Wrapper version > APM version → refuse to spawn with a clear error: 'wrapper <name> targets contract version N but this APM build supports up to version 1; upgrade APM'.
- Built-in wrappers always target the current APM build's contract version (no manifest needed).
- `apm agents test` (from ticket 25c92daa wait — that's mocks; from ticket 9 / apm agents subcommand) integrates the version check into the smoke test result.

**Centralized version constant:**
- New const `apm_core::wrapper::CONTRACT_VERSION: u32 = 1;`. Bumped (in code) when the contract changes. This ticket establishes 1 as the value; future contract changes increment.

**Out of scope:**
- Defining what a contract bump means (what changes constitute a major version bump). Document at bump time.
- Backporting v1 → v2 helpers. Future concern.
- A registry of wrapper versions across the apm-agents ecosystem. Future concern.

**Tests:**
- Wrapper with contract_version = 1 → spawn succeeds, no warnings.
- Wrapper with no manifest → assumed v1, spawn succeeds.
- Wrapper with contract_version = 2 → spawn refuses with the upgrade message.
- Hypothetical: simulate APM v2 by setting `CONTRACT_VERSION = 2`, wrapper still v1 → spawn succeeds with the older-version warning.

### Acceptance criteria

- [ ] `pub const CONTRACT_VERSION: u32 = 1` is defined in `apm_core::wrapper` and accessible from outside the module
- [ ] `APM_WRAPPER_VERSION` env var is set to `CONTRACT_VERSION.to_string()` (not a hardcoded `"1"`) in both `ClaudeWrapper::spawn` and `CustomWrapper::spawn`
- [ ] Spawning a custom wrapper whose manifest declares `contract_version = 1` (equal to `CONTRACT_VERSION`) succeeds and no version-warning line is written to the worker log
- [ ] Spawning a custom wrapper with no manifest present defaults to `contract_version = 1`, spawn succeeds, no warning written
- [ ] Spawning a custom wrapper whose manifest declares `contract_version > CONTRACT_VERSION` returns `Err` and does not produce a child process
- [ ] The error for `contract_version > CONTRACT_VERSION` includes the wrapper name, the declared version number, the APM max-supported version, and the string `"upgrade APM"`
- [ ] Spawning a custom wrapper whose manifest declares `contract_version < CONTRACT_VERSION` succeeds (returns `Ok(child)`, no error)
- [ ] When declared version is less than `CONTRACT_VERSION`, a warning line is appended to the worker log file before spawn proceeds
- [ ] The version-comparison logic is extracted into a private helper `check_contract_version(declared: u32, apm_version: u32, log_path: &Path)` so the older-version warning path can be exercised in a unit test without modifying the compile-time constant

### Out of scope

- Defining what changes to the contract constitute a version bump — documented at the time of the bump, not here
- Backporting compatibility shims for wrappers targeting future contract versions
- A wrapper-version registry or cross-ecosystem compatibility matrix
- `apm validate` output for version mismatches — that check is already part of ticket 2c32a282's `validate_agents` helper
- Surfacing `CONTRACT_VERSION` in any CLI output (`apm version`, `apm validate --verbose`, etc.)
- Version-checking built-in wrappers — they always target the current build's version by definition and carry no manifest

### Approach

**Files changed:** `apm-core/src/wrapper/mod.rs`, `apm-core/src/wrapper/claude.rs`, `apm-core/src/wrapper/custom.rs`

---

**1. Define the constant — `wrapper/mod.rs`**

Add at module scope (after imports, before the trait definition):

```rust
pub const CONTRACT_VERSION: u32 = 1;
```

This is the single source of truth. Every comparison in the spawn path and every `APM_WRAPPER_VERSION` env var value derives from it.

---

**2. Use the constant in `ClaudeWrapper` — `wrapper/claude.rs`**

Add `use super::CONTRACT_VERSION;`.

In `ClaudeWrapper::spawn`, replace the hardcoded string literal `"1"` for the `APM_WRAPPER_VERSION` env key with `CONTRACT_VERSION.to_string()`. Both the local-path and container-path branches need this change.

---

**3. Extend `CustomWrapper` — `wrapper/custom.rs`**

Add `use super::CONTRACT_VERSION;`.

**Extract a private helper** (keeps the version logic independently testable):

```rust
fn check_contract_version(
    declared: u32,
    apm_version: u32,
    log_path: &Path,
) -> anyhow::Result<()> {
    match declared.cmp(&apm_version) {
        std::cmp::Ordering::Greater => anyhow::bail!(
            "wrapper targets contract version {} but this APM build supports up to \
             version {}; upgrade APM",
            declared,
            apm_version,
        ),
        std::cmp::Ordering::Less => {
            // Non-fatal: append a warning to the log and continue.
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(log_path)
            {
                let _ = writeln!(
                    f,
                    "[apm] warning: wrapper targets contract version {} but this APM \
                     build is version {}; the wrapper may not use newer env vars",
                    declared, apm_version,
                );
            }
        }
        std::cmp::Ordering::Equal => {}
    }
    Ok(())
}
```

**Update `CustomWrapper::spawn`:**

- Derive `declared`: `let declared = self.manifest.as_ref().map_or(1, |m| m.contract_version);`
- Replace the existing `if contract_version > 1 { bail!(...) }` block introduced by 2c32a282 with a single call: `check_contract_version(declared, CONTRACT_VERSION, &ctx.log_path)?;`
- Replace the hardcoded `"1"` for `APM_WRAPPER_VERSION` with `CONTRACT_VERSION.to_string()`.

The helper subsumes the old bail — no net behaviour change for the `> 1` case, and it adds the new `< CONTRACT_VERSION` warning path.

---

**4. Tests — `wrapper/custom.rs` under `#[cfg(test)]`**

Four unit tests against the helper (no subprocess needed):

- `check_version_equal` — `check_contract_version(1, 1, &log)` returns `Ok(())`, log is empty.
- `check_version_older_writes_warning` — `check_contract_version(1, 2, &log)` returns `Ok(())`, log file contains the word `"warning"` and both version numbers.
- `check_version_too_high_returns_err` — `check_contract_version(2, 1, &log)` returns `Err`, error string contains `"upgrade APM"`, `"2"`, and `"1"`.
- `check_version_no_manifest_defaults_to_1` — set `declared = None::<Manifest>.map_or(1, |m| m.contract_version)`, assert it equals 1 (documents the default assumption without spawning).

Two integration tests using a real subprocess (same fixture pattern as 2c32a282's `integration_echo_test_wrapper`):

- `spawn_matching_contract_succeeds` — fixture wrapper with `manifest.toml` declaring `contract_version = 1`; assert spawn returns `Ok`, exit code 0, log contains no `"warning"` line.
- `spawn_future_contract_rejected` — fixture wrapper with `manifest.toml` declaring `contract_version = 2`; assert `CustomWrapper::spawn` returns `Err` before a child is created.

The `check_version_older_writes_warning` unit test covers the older-version path by calling the helper directly with `apm_version = 2`; no integration test is needed for this branch since `CONTRACT_VERSION = 1` makes it currently unreachable in production.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:05Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T22:03Z | groomed | in_design | philippepascal |
| 2026-05-01T00:09Z | in_design | ammend | philippepascal |
| 2026-05-01T00:30Z | ammend | in_design | philippepascal |
| 2026-05-01T00:33Z | in_design | specd | claude-0501-0030-e588 |
| 2026-05-01T17:38Z | specd | ready | philippepascal |
| 2026-05-01T19:14Z | ready | in_progress | philippepascal |
