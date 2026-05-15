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

- Adding or changing the `build_system_prompt` function itself (ba121f45)
- Adding the `apm prompt` CLI command (ba121f45)
- Changing the priority cascade or per-agent file Level 0 logic (ba121f45)
- Shelling out to `apm prompt` as a subprocess — the spawn paths call `build_system_prompt` directly as a library function
- Changes to argument-construction code in the spawn paths beyond the function-name substitution
- Modifying any spawn-path behavior other than the system-prompt call

### Approach

**Design decision — direct call, not shell-out.** The spawn paths call build_system_prompt() as a library function. Shelling out to apm prompt would add subprocess overhead and complex error handling with no benefit: ba121f45 already designed apm prompt to use the same lookup as run(), so the parity guarantee is structural, not process-boundary-dependent.

**Coordination with ba121f45**

ba121f45 Step 3 states it will replace resolve_system_prompt with build_system_prompt at the same three call sites. Two valid splits exist:

1. ba121f45 keeps resolve_system_prompt as a deprecated alias calling through to build_system_prompt, leaving the spawn-path call sites untouched. de2588b4 then removes the alias and updates all three sites.
2. ba121f45 renames the function and updates all usages including the three spawn sites. de2588b4 becomes a verification-and-test ticket only.

Either split is acceptable. The implementer should confirm with the ba121f45 implementer which path they took before starting work. If ba121f45 already updated the three call sites, this ticket closes by verifying the parity ACs and renaming any remaining test references.

**Call site changes (apm-core/src/start.rs)**

Three locations, each a one-word substitution — no surrounding argument-construction code changes:

- run() ~line 363: resolve_system_prompt(root, tr_instructions, profile, &config.workers, &params.agent, role)? becomes build_system_prompt with the same args
- run_next() ~line 566: resolve_system_prompt(root, tr_instructions2, profile2, &config.workers, &params.agent, role2)? becomes build_system_prompt with the same args
- spawn_next_worker() ~line 770: resolve_system_prompt(root, tr_instructions_snw, profile2, &config.workers, &params.agent, role2)? becomes build_system_prompt with the same args

**Test updates (apm-core/src/start.rs test module)**

The use super:: import at ~line 960 lists resolve_system_prompt by name. Update it to build_system_prompt. Rename the ~8 test functions that include resolve_system_prompt in their name to use build_system_prompt. No logic changes to test bodies — assertions remain the same.

**Parity verification**

After the substitution, verify the parity ACs by running apm prompt <id> and comparing its stdout against the system-prompt temp file written by a spawn (visible via a test fixture or debug log). No new automated test infrastructure is required beyond the unit-test renames above.

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