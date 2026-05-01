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
updated_at = "2026-05-01T00:09:40.082843Z"
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

- [ ] **Dispatcher registration**
- [ ] `resolve_builtin("mock-happy")` returns `Some(_)`
- [ ] `resolve_builtin("mock-sad")` returns `Some(_)`
- [ ] `resolve_builtin("mock-random")` returns `Some(_)`
- [ ] `resolve_builtin("debug")` returns `Some(_)`

- [ ] **mock-happy — spec mode (ticket in `in_design`)**
- [ ] When run against a ticket in `in_design` state, `mock-happy` writes non-empty content to all four required spec sections: Problem, Acceptance criteria, Out of scope, Approach
- [ ] When run against a ticket in `in_design` state, `mock-happy` sets the ticket's `effort` to `1`
- [ ] When run against a ticket in `in_design` state, `mock-happy` sets the ticket's `risk` to `1`
- [ ] When run against a ticket in `in_design` state, `mock-happy` transitions the ticket to `specd`

- [ ] **mock-happy — impl mode (ticket in `in_progress`)**
- [ ] When run against a ticket in `in_progress` state, `mock-happy` creates at least one new git commit in the worktree
- [ ] When run against a ticket in `in_progress` state, `mock-happy` calls `apm state <id> implemented`

- [ ] **mock-happy — JSONL output**
- [ ] `mock-happy` emits at least one JSONL line on stdout; each emitted line is a valid JSON object with `"type": "tool_use"`

- [ ] **mock-happy — error cases and exit**
- [ ] When the current state has zero `outcome = "success"` transitions, `mock-happy` exits non-zero and writes a diagnostic message to stderr
- [ ] When the current state has two or more `outcome = "success"` transitions, `mock-happy` exits non-zero and writes a diagnostic message to stderr
- [ ] `mock-happy` exits 0 when it completes without error

- [ ] **mock-sad — behaviour**
- [ ] `mock-sad` writes content to at least one but fewer than all four required spec sections (Problem, Acceptance criteria, Out of scope, Approach)
- [ ] `mock-sad` transitions the ticket to a state reachable via a transition whose `resolve_outcome` result is not `"success"`
- [ ] `mock-sad` exits 0 after completing its run

- [ ] **mock-sad — seeding**
- [ ] Given the same `APM_OPT_SEED` value, two successive `mock-sad` spawns against the same ticket in the same state choose the same target state

- [ ] **mock-sad — error case**
- [ ] When no non-success transitions are available from the current state, `mock-sad` exits non-zero and writes a diagnostic to stderr

- [ ] **mock-random — behaviour**
- [ ] `mock-random` transitions the ticket to a state reachable by any valid transition from the current state (including success-outcome transitions)
- [ ] When `mock-random` picks a success-outcome transition, it writes all four spec sections and sets effort/risk (spec mode) or creates a commit (impl mode), matching `mock-happy`'s behaviour
- [ ] When `mock-random` picks a non-success-outcome transition, it writes only partial spec content (matching `mock-sad`'s behaviour)
- [ ] `mock-random` exits 0 after completing its run

- [ ] **mock-random — seeding**
- [ ] Given the same `APM_OPT_SEED` value, two successive `mock-random` spawns against the same ticket in the same state choose the same target state

- [ ] **debug — output**
- [ ] `debug` writes the name and value of every `APM_*` environment variable to stderr
- [ ] `debug` writes the full contents of the file at `APM_SYSTEM_PROMPT_FILE` to stderr
- [ ] `debug` writes the full contents of the file at `APM_USER_MESSAGE_FILE` to stderr
- [ ] `debug` emits exactly one JSONL line on stdout: a valid JSON object with `"type": "tool_use"`

- [ ] **debug — behaviour**
- [ ] `debug` does not call `apm state`; the ticket's state is unchanged after `debug` runs
- [ ] `debug` exits 0

- [ ] **workflow.toml**
- [ ] `in_design → specd` carries `outcome = "success"` in the default workflow (`apm-core/src/default/workflow.toml`)
- [ ] `ammend → specd` carries `outcome = "success"` in the default workflow
- [ ] The project's `.apm/workflow.toml` carries the same two annotations

### Out of scope

- User-facing documentation for mock wrappers beyond the existing `docs/agent-wrappers.md` reference
- The `apm agents` subcommand family (`apm agents test`, `apm agents list`, etc.) — ticket 71d80e40
- Custom wrapper resolution from `.apm/agents/<name>/` — ticket 2c32a282
- Per-ticket `frontmatter.agent` / `agent_overrides` override — ticket 0ca3e019
- Built-in wrappers for third-party agents (`codex`, `aider`, etc.)
- Wrapper-contract version compatibility checks (`manifest.toml`, `APM_WRAPPER_VERSION` ceiling) — ticket 2e772eab
- Per-agent instruction file resolution under `.apm/agents/<name>/apm.*.md` (ticket 7f5f73d5); mocks do not invoke a real agent so instruction files are irrelevant to their operation
- Windows or non-Unix platform support; mocks shell out to `/bin/sh`
- Automated handling of the `pr_or_epic_merge` completion strategy in integration tests; mock-happy creates a real commit and calls `apm state <id> implemented`, then APM's orchestration layer (running separately) handles the merge — no in-test merge attempt
- Validating that `apm validate` warns when a workflow has no reachable success outcome — that check lives in ticket a1b94ea4

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
| 2026-05-01T00:08Z | in_design | ammend | philippepascal |
| 2026-05-01T00:09Z | ammend | in_design | philippepascal |