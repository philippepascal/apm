+++
id = "33927683"
title = "Pre-existing test failure: mock_happy_spec_mode_transitions_to_specd"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/33927683-pre-existing-test-failure-mock-happy-spe"
created_at = "2026-05-04T03:33:27.432606Z"
updated_at = "2026-05-04T04:35:40.291779Z"
+++

## Spec

### Problem

The integration test `start::tests::mock_happy_spec_mode_transitions_to_specd` (and two sibling tests that share the same helper) fails because `find_apm_bin()` — a test-only helper in `apm-core/src/start.rs` — resolves to the wrong binary.\n\n`find_apm_bin()` first checks `APM_BIN`, then falls back to `which apm`. On any machine with Atom Package Manager installed via Homebrew, `which apm` returns `/opt/homebrew/bin/apm`, which does not recognise the `spec`, `state`, or `set` subcommands. The mock-happy shell script calls `"$APM" spec "$ID" ...`, which fails immediately, so the ticket never reaches `specd` state and the assertion fires.\n\nThe correct binary is the one this workspace builds: `target/debug/apm` (or `target/release/apm`). The fix is to replace the `which apm` fallback with a lookup that derives the target directory from `std::env::current_exe()`, which always points at the actual Cargo test binary regardless of PATH.\n\nThe failure pre-dates ticket f8cbd68c and is not a regression introduced by that branch.

### Acceptance criteria

- [ ] `start::tests::mock_happy_spec_mode_transitions_to_specd` passes when `target/debug/apm` exists (project has been built)\n- [ ] `start::tests::mock_sad_transitions_to_non_success_state` passes when `target/debug/apm` exists\n- [ ] `start::tests::mock_sad_seed_reproducibility` passes when `target/debug/apm` exists\n- [ ] All three tests skip (return without panic or assertion failure) when `APM_BIN` is unset and no cargo-built binary is found at the derived path\n- [ ] Setting `APM_BIN` to a valid path still takes priority over the cargo-relative lookup\n- [ ] `which apm` is no longer invoked by `find_apm_bin()`

### Out of scope

- Fixing test failures unrelated to `find_apm_bin()`\n- Changing how `APM_BIN` is propagated to scripts at runtime (production path in `wrapper/builtin/mod.rs`)\n- Handling Windows paths or non-Unix test environments\n- Adding a CI step to pre-build `apm` before running `apm-core` tests (CI config concern, not a code concern)

### Approach

**File:** `apm-core/src/start.rs` — `find_apm_bin()` (inside the `#[cfg(test)]` block, ~line 1739).\n\nReplace the `which apm` branch with a cargo-relative lookup. The resulting function:\n\n```rust\nfn find_apm_bin() -> Option<String> {\n    // 1. Explicit override wins\n    if let Ok(v) = std::env::var("APM_BIN") {\n        if !v.is_empty() && std::path::Path::new(&v).exists() {\n            return Some(v);\n        }\n    }\n    // 2. Derive from the test binary path.\n    //    current_exe() -> <workspace>/target/{profile}/deps/apm_core-<hash>\n    //    two parents up -> <workspace>/target/{profile}/\n    //    sibling "apm"  -> <workspace>/target/{profile}/apm\n    if let Ok(exe) = std::env::current_exe() {\n        if let Some(target_dir) = exe.parent().and_then(|p| p.parent()) {\n            let candidate = target_dir.join("apm");\n            if candidate.is_file() {\n                return Some(candidate.to_string_lossy().into_owned());\n            }\n        }\n    }\n    None\n}\n```\n\n`is_file()` (not `exists()`) rules out a directory accidentally named `apm`.\n\nNo other files change. The three affected tests already call `find_apm_bin()` and return early on `None`, so their guard logic is correct as-is.\n\n**Verify:** `cargo build -p apm && cargo test -p apm-core start::tests::mock_happy_spec_mode_transitions_to_specd` should pass. Without building first, the test skips silently.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-04T03:33Z | — | new | claude-0503-1430-f8cb|philippepascal |
| 2026-05-04T04:35Z | new | groomed | philippepascal |
| 2026-05-04T04:35Z | groomed | in_design | philippepascal |