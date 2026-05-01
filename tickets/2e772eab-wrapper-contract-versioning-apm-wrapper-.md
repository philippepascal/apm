+++
id = "2e772eab"
title = "Wrapper-contract versioning (APM_WRAPPER_VERSION + manifest.toml)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2e772eab-wrapper-contract-versioning-apm-wrapper-"
created_at = "2026-04-30T20:05:11.077339Z"
updated_at = "2026-05-01T00:30:28.342775Z"
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

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

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
