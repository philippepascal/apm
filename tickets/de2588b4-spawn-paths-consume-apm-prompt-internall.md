+++
id = "de2588b4"
title = "Spawn paths consume apm prompt internally"
state = "in_design"
priority = 0
effort = 2
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/de2588b4-spawn-paths-consume-apm-prompt-internall"
created_at = "2026-05-14T21:14:34.141790Z"
updated_at = "2026-05-15T21:49:41.304742Z"
depends_on = ["ba121f45"]
+++

## Spec

### Problem

ba121f45 renames `resolve_system_prompt` to `build_system_prompt` (Step 1), exposes `apm prompt <id>` as a CLI that calls the same function via the same transition-lookup path used by the spawn paths (Step 2), and updates the three call sites in `start.rs` and any existing test references (Step 3). After ba121f45 lands, the function rename and call-site substitutions are complete.\n\nThe gap that remains is automated verification: nothing asserts that `apm prompt <id>` and the three spawn paths produce identical system-prompt strings for the same ticket. A future refactor to argument-construction logic in any of the four code paths could silently break parity. This ticket delivers the unit tests that close that gap.

### Acceptance criteria

- [ ] A unit test verifies that `prompt::run()` and the argument-construction path used by `run()` produce the same `build_system_prompt` result for a fixture ticket\n- [ ] Equivalent parity tests exist for `run_next()` and `spawn_next_worker()`\n- [ ] A unit test verifies that when `build_system_prompt` returns an error (e.g. instructions file missing), each spawn path propagates it unchanged\n- [ ] All new tests pass against the post-ba121f45 codebase

### Out of scope

- Renaming `resolve_system_prompt` to `build_system_prompt` (ba121f45 Step 1)\n- Adding the `apm prompt` CLI command (ba121f45 Step 2)\n- Updating the three call sites in `start.rs` (ba121f45 Step 3)\n- Renaming existing test functions from `resolve_system_prompt` to `build_system_prompt` (ba121f45)\n- Changing the priority cascade or per-agent file Level 0 logic (ba121f45)\n- Shelling out to `apm prompt` as a subprocess

### Approach

**Scope**\n\nba121f45 Step 3 owns all three call-site substitutions and test renames in `start.rs`. This ticket adds parity unit tests that verify the four paths (`run()`, `run_next()`, `spawn_next_worker()`, and `prompt::run()`) produce identical `build_system_prompt` output for the same ticket.\n\n**Test fixture**\n\nAdd a helper in the `apm-core/src/start.rs` test module that builds a minimal temp directory:\n\n- `.apm/config.toml` — minimal config with a workers block\n- `.apm/workflow.toml` — one state with one spawnable transition; `instructions` points to a temp file containing arbitrary content (e.g. `"test instructions"`)\n- `tickets/<id>.md` — ticket in the spawnable state\n\nThe instructions file content is arbitrary; the parity tests only assert that all four paths return the same string, not its value.\n\n**Parity tests**\n\nThree tests (or one parameterized test), one per spawn path:\n\n1. Invoke the argument-construction logic that the spawn path uses and call `build_system_prompt()` directly, capturing the result.\n2. Call `prompt::run()` for the same fixture ticket, capturing its return value (or redirect stdout to a buffer if `prompt::run()` writes directly).\n3. Assert both results are equal.\n\nIf the argument-construction logic is shared as ba121f45 designs, these tests will pass trivially and serve as regression guards against future drift.\n\n**Error path test**\n\nConfigure the fixture so `transition.instructions` points to a non-existent file. Assert that the spawn path returns an `Err` and that the error message string is identical to what `build_system_prompt()` returns directly. One test covering `run()` is sufficient; the error propagation path is the same across all three spawn paths.

### Open questions


### Amendment requests

- [ ] The Approach presents two alternative splits with ba121f45 and instructs the implementer to 'confirm with the ba121f45 implementer'. Drop the alternatives. ba121f45 Step 3 already commits to replacing resolve_system_prompt with build_system_prompt at all three call sites in start.rs. This ticket must commit to a single, post-ba121f45 scope.
- [ ] With ba121f45 owning the call-site substitution, this ticket is materially redundant. Either close it as folded into ba121f45, or re-scope it strictly to parity testing — i.e. its sole deliverable becomes the parity unit test (assembled prompt from run()/run_next()/spawn_next_worker() equals apm prompt stdout). The Approach currently says 'No new automated test infrastructure is required' which contradicts the ACs that assert equality of the assembled prompt strings.
- [ ] Approach also lists call-site line numbers (~363, ~566, ~770) as one-word substitutions. If ba121f45 owns those substitutions, remove this section; if this ticket owns them after re-scope, keep them and remove the parallel claim in ba121f45 Step 3. Avoid the double-write.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-14T21:14Z | — | new | philippe|philippepascal |
| 2026-05-14T21:22Z | new | groomed | philippe |
| 2026-05-15T01:39Z | groomed | in_design | philippe |
| 2026-05-15T01:45Z | in_design | specd | default-0515-0139-de08 |
| 2026-05-15T19:56Z | specd | ammend | philippe |
| 2026-05-15T21:49Z | ammend | in_design | philippe |