+++
id = "25c92daa"
title = "Mock and debug built-in wrappers (mock-happy, mock-sad, mock-random, debug)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/25c92daa-mock-and-debug-built-in-wrappers-mock-ha"
created_at = "2026-04-30T20:04:21.901984Z"
updated_at = "2026-04-30T21:57:43.460139Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["d3b93b95", "a1b94ea4", "6cac8518"]
+++

## Spec

### Problem

Ship three mock built-in wrappers for testing the harness without burning credits, plus a `debug` introspection helper. All four are Rust built-ins registered in the dispatcher from ticket d3b93b95.

**Reference spec:** `docs/agent-wrappers.md` — sections 'mock-happy', 'mock-sad', 'mock-random', 'Mock-happy details', 'Mock-sad / mock-random determinism', 'Detailed considerations / debug'.

**Scope:**

**`mock-happy`** built-in:
- For spec-writer profile: writes minimal valid markdown to all required spec sections via shelling out to the same `apm` binary (`apm spec --set` per section). Sets effort and risk to 1, 1.
- For impl-agent profile: emits a fake commit (creates a placeholder file in the worktree, `git add` + `git commit`).
- Picks the transition with `outcome = "success"` from the ticket's current state (using the helper from ticket a1b94ea4) and runs `apm state <id> <to-state>`. If zero or multiple success transitions exist, exit non-zero with a diagnostic.
- Emits 1–2 fake JSONL events on stdout for log realism.
- Exits 0.

**`mock-sad`** built-in:
- Writes some-but-not-all required spec sections (or content that fails validate).
- Optionally writes a question to `### Open questions`.
- Picks uniformly from transitions where `outcome ≠ "success"` valid from the current state. Seedable via `APM_OPT_SEED`.
- Runs `apm state <id> <to-state>`. If the eligible set is empty, exit non-zero with a diagnostic.
- Exits 0.

**`mock-random`** built-in:
- Picks uniformly from ALL valid transitions (any outcome, including success). Same seeding via `APM_OPT_SEED`.
- For success: behaves like mock-happy. For non-success: behaves like mock-sad.
- Exits 0.

**`debug`** built-in:
- Prints all `APM_*` env vars to stderr.
- Prints contents of `APM_SYSTEM_PROMPT_FILE` and `APM_USER_MESSAGE_FILE` to stderr.
- Emits a single canonical `tool_use` JSONL event on stdout.
- Does NOT call `apm state` (no transition).
- Exits 0.
- Useful for verifying wrapper-contract plumbing without invoking any real agent.

**Implementation notes:**
- All four live under `apm-core/src/wrapper/builtin/` (one file each: `claude.rs`, `mock_happy.rs`, `mock_sad.rs`, `mock_random.rs`, `debug.rs`).
- The mocks shell out to the host `apm` binary for state transitions and spec writes — no special internal API.
- Mocks read the workflow from `apm-core::config::Config::load(root)` to find valid transitions and their outcomes (using the helper from ticket a1b94ea4).
- Per-agent instruction files (`apm.worker.md`, `apm.spec-writer.md`) are NOT needed for mocks — the per-agent instructions resolution from ticket 7f5f73d5 should fall through gracefully when those files don't exist for a built-in. Confirm in spec phase.

**Out of scope:**
- Documenting the mocks in user-facing help/docs beyond the existing `docs/agent-wrappers.md` reference.
- Apm subcommand support (`apm agents test mock-happy` etc.) — that is a separate ticket.

**Tests:**
- For each mock: integration test that wires it up against a fixture project, runs a worker, asserts the expected state transition occurred.
- For mock-sad / mock-random: assert seed reproducibility (same seed → same chosen transition).
- For debug: assert env vars, prompt, and message all appear in the captured `.apm-worker.log`.

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
| 2026-04-30T20:04Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:57Z | groomed | in_design | philippepascal |
