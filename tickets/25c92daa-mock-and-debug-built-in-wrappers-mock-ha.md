+++
id = "25c92daa"
title = "Mock and debug built-in wrappers (mock-happy, mock-sad, mock-random, debug)"
state = "in_design"
priority = 0
effort = 6
risk = 4
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/25c92daa-mock-and-debug-built-in-wrappers-mock-ha"
created_at = "2026-04-30T20:04:21.901984Z"
updated_at = "2026-05-01T01:43:18.690653Z"
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

- [ ] **mock-random — error case**
- [ ] When no valid transitions are available from the current state, `mock-random` exits non-zero and writes a diagnostic to stderr

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

- [ ] **per-agent instruction file stubs**
- [ ] Each of the four built-in wrappers (`mock-happy`, `mock-sad`, `mock-random`, `debug`) has both `apm.worker.md` and `apm.spec-writer.md` stub files under `apm-core/src/default/agents/<name>/`

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

Four built-in wrappers added to `apm-core/src/wrapper/`, each implemented as a generated shell script that the Rust `spawn()` method writes to the worktree and executes via `/bin/sh`.

**Prerequisite: workflow.toml annotation**

The default workflow's `in_design → specd` and `ammend → specd` transitions have no `completion` strategy, so `resolve_outcome` (from ticket a1b94ea4) infers `"needs_input"` for them. That means mock-happy would find zero success transitions for spec-writer states and always exit non-zero.

Fix: this ticket explicitly adds `outcome = "success"` to both transitions in `apm-core/src/default/workflow.toml` and `.apm/workflow.toml`. These override the implicit default — completing a spec IS the spec-writer's success path even without a git merge.

Note: ticket a1b94ea4 also annotates workflow.toml. The two changes are compatible (a1b94ea4 annotates transitions that match their implicit default; this ticket annotates two that override the default). Implementers must merge cleanly.

**1. WrapperContext extensions (`wrapper/mod.rs`)**

Add two fields to `WrapperContext` (both set by callers in `start.rs`):

- `root_path: PathBuf` — project root (not the worktree). Set to `root.to_path_buf()`.
- `current_state: String` — the ticket's state at spawn time. Set from `ticket.frontmatter.state.clone()`.

Also add `APM_PROJECT_ROOT` to the env vars set by `ClaudeWrapper::spawn()` (both local and container paths) for consistency. All four new wrappers set it too.

**2. File layout**

Create `apm-core/src/wrapper/builtin/` and move `claude.rs` there if d3b93b95 placed it at `wrapper/claude.rs` (update all `use` paths). Add `pub mod builtin;` to `wrapper/mod.rs`.

New files: `wrapper/builtin/mock_happy.rs`, `wrapper/builtin/mock_sad.rs`, `wrapper/builtin/mock_random.rs`, `wrapper/builtin/debug.rs`.

New `wrapper/builtin/mod.rs` contains shared helpers (see below) and re-exports all four structs.

Extend `resolve_builtin()` in `wrapper/mod.rs`:
```
"mock-happy"  => Some(Box::new(MockHappyWrapper)),
"mock-sad"    => Some(Box::new(MockSadWrapper)),
"mock-random" => Some(Box::new(MockRandomWrapper)),
"debug"       => Some(Box::new(DebugWrapper)),
```

**3. Shared helpers in `wrapper/builtin/mod.rs`**

`load_transitions_with_outcomes(ctx: &WrapperContext) -> anyhow::Result<Vec<(TransitionConfig, StateConfig)>>`:
1. `Config::load(&ctx.root_path)`.
2. Find `StateConfig` matching `ctx.current_state` in the workflow; bail if not found.
3. Build a `HashMap<&str, &StateConfig>` for target lookup.
4. For each transition in the current state, clone `(TransitionConfig, target StateConfig)` and return the vec.

`is_impl_mode(transitions: &[(TransitionConfig, StateConfig)]) -> bool`:
Returns true when any transition has `completion != CompletionStrategy::None`. True for `in_progress` (has `pr_or_epic_merge`); false for `in_design`.

`happy_script(apm: &str, id: &str, target: &str, impl_mode: bool) -> String` and `sad_script(apm: &str, id: &str, target: &str) -> String`:
Private functions that return the shell script strings (see below). Called by all three mocks to avoid duplication.

`write_and_spawn_script(name: &str, script: &str, ctx: &WrapperContext) -> anyhow::Result<Child>`:
1. Write script to `<worktree>/.apm-mock-<name>-<rand_u16()>.sh`, chmod 0o755.
2. `Command::new("/bin/sh")` with the script path as arg.
3. Set all APM contract env vars (same set as `ClaudeWrapper`, including `APM_PROJECT_ROOT`).
4. `.current_dir(&ctx.worktree_path)`, `.process_group(0)`.
5. Redirect stdout + stderr to `File::create(&ctx.log_path)?` / `try_clone()`.
6. `.spawn()` and return `Child`. The script ends with `rm -f "$0"` (self-cleanup; no separate thread needed).

`apm_bin() -> anyhow::Result<String>`: returns `std::env::current_exe()?.to_str()…?`. Used so scripts shell out to the same `apm` binary that spawned them.

`apm_bin_from_ctx(ctx: &WrapperContext) -> anyhow::Result<String>`: checks `ctx.options.get("apm_bin")` first (allows test override), then falls back to `apm_bin()`.

`seed_from_ctx(ctx: &WrapperContext) -> u64`: reads `ctx.options.get("seed").and_then(|s| s.parse().ok())`, falls back to `rand::thread_rng().gen::<u64>()`.

**4. MockHappyWrapper spawn() steps**

1. `load_transitions_with_outcomes(ctx)`.
2. Filter to success: `resolve_outcome(t, s) == "success"`.
3. Match count: 0 → bail with diagnostic naming the current state; 2+ → bail with count.
4. `target = success[0].0.to.clone()`, `impl_mode = is_impl_mode(&all)`.
5. `apm = apm_bin_from_ctx(ctx)?`.
6. `script = happy_script(&apm, &ctx.ticket_id, &target, impl_mode)`.
7. `write_and_spawn_script("happy", &script, ctx)`.

`happy_script` spec mode (not impl_mode):
```sh
#!/bin/sh
set -e
APM="<apm_bin>"
ID="<ticket_id>"
"$APM" spec "$ID" --section "Problem" --set "Mock spec — no real problem analyzed."
printf '- [ ] Mock criterion 1\n- [ ] Mock criterion 2\n' > ".apm-mock-ac-$$.txt"
"$APM" spec "$ID" --section "Acceptance criteria" --set-file ".apm-mock-ac-$$.txt"
rm -f ".apm-mock-ac-$$.txt"
"$APM" spec "$ID" --section "Out of scope" --set "- Nothing in scope for this mock run"
"$APM" spec "$ID" --section "Approach" --set "Mock approach — no real implementation analyzed."
"$APM" set "$ID" effort 1
"$APM" set "$ID" risk 1
printf '{"type":"tool_use","id":"mock-1","name":"write_spec","input":{}}\n'
printf '{"type":"tool_use","id":"mock-2","name":"apm_state","input":{}}\n'
"$APM" state "$ID" <target>
rm -f "$0"
```

`happy_script` impl mode:
```sh
#!/bin/sh
set -e
APM="<apm_bin>"
ID="<ticket_id>"
printf 'mock: placeholder implementation for ticket %s\n' "$ID" > mock-implementation.txt
git add mock-implementation.txt
git commit -m "mock: placeholder commit for ticket $ID"
printf '{"type":"tool_use","id":"mock-1","name":"git_commit","input":{}}\n'
printf '{"type":"tool_use","id":"mock-2","name":"apm_state","input":{}}\n'
"$APM" state "$ID" <target>
rm -f "$0"
```

**5. MockSadWrapper spawn() steps**

1. `load_transitions_with_outcomes(ctx)`.
2. Filter to non-success: `resolve_outcome(t, s) != "success"`.
3. Empty → bail with diagnostic.
4. `seed = seed_from_ctx(ctx)`, pick `idx = seed as usize % eligible.len()`, `target = eligible[idx].0.to.clone()`.
5. `script = sad_script(&apm, &ctx.ticket_id, &target)`.
6. `write_and_spawn_script("sad", &script, ctx)`.

`sad_script`:
```sh
#!/bin/sh
set -e
APM="<apm_bin>"
ID="<ticket_id>"
"$APM" spec "$ID" --section "Problem" --set "Mock sad run — spec intentionally incomplete."
printf '{"type":"tool_use","id":"mock-1","name":"write_partial_spec","input":{}}\n'
"$APM" state "$ID" <target>
rm -f "$0"
```

**6. MockRandomWrapper spawn() steps**

1. `load_transitions_with_outcomes(ctx)`.
2. Empty → bail.
3. `seed = seed_from_ctx(ctx)`, pick `idx = seed as usize % all.len()`, chosen = `all[idx]`.
4. `outcome = resolve_outcome(&chosen.0, &chosen.1)`.
5. If `outcome == "success"` → `happy_script(...)`, else → `sad_script(...)`.
6. `write_and_spawn_script("random", &script, ctx)`.

**7. DebugWrapper spawn() steps**

No config loading. Script:
```sh
#!/bin/sh
env | grep '^APM_' >&2
printf '\n=== SYSTEM PROMPT ===\n' >&2
cat "$APM_SYSTEM_PROMPT_FILE" >&2
printf '\n=== USER MESSAGE ===\n' >&2
cat "$APM_USER_MESSAGE_FILE" >&2
printf '{"type":"tool_use","id":"debug-1","name":"noop","input":{}}\n'
rm -f "$0"
```
No `apm state` call. Ticket state is unchanged.

**8. Tests**

Unit tests (in `wrapper/mod.rs` or `wrapper/builtin/mod.rs`):
- `resolve_builtin_mock_happy_returns_some`, `_mock_sad_`, `_mock_random_`, `_debug_` — each asserts `resolve_builtin("<name>").is_some()`.

Integration tests (in `start.rs` `#[cfg(test)]` or `tests/mock_wrappers.rs`):

Each test uses a fixture helper that: creates a `tempfile::TempDir` as project root, writes minimal `.apm/config.toml` (with the wrapper under test as `agent`), writes the default workflow.toml (with the two new `outcome = "success"` annotations), creates a ticket file in the correct state, runs `git init` + initial commit (required for state and spec ops), then builds a `WrapperContext` with `apm_bin` option set to the `apm` binary found via `which apm` or passed as a test env var.

- `mock_happy_spec_mode_transitions_to_specd` — ticket in `in_design`; after `spawn_worker(ctx); child.wait()`: assert ticket state = `specd`, all four spec section headers present in ticket file, effort = 1, risk = 1.
- `mock_happy_impl_mode_creates_commit` — ticket in `in_progress`; assert state = `implemented`; assert `git log --oneline` in worktree contains a commit with "mock" in the message.
- `mock_happy_zero_success_exits_nonzero` — custom single-state workflow (all transitions non-success); assert `child.wait().status.success() == false`; read log and assert it contains "no success-outcome transition".
- `mock_sad_transitions_to_non_success_state` — ticket in `in_design`; assert resulting state is not `specd`; assert ticket file does NOT have all four spec sections populated.
- `mock_sad_seed_reproducibility` — two fresh fixture tickets in the same state, both spawned with `APM_OPT_SEED=42`; assert both end in the same target state.
- `mock_random_seed_reproducibility` — same pattern with mock-random.
- `debug_does_not_change_state` — ticket in `in_design`; after `spawn_worker; wait`: assert state still `in_design`; read log file and assert it contains `APM_TICKET_ID`, the system prompt text, and a line matching `{"type":"tool_use"`.

### Background constraint: workflow.toml annotation gap

The default workflow's `in_design → specd` and `ammend → specd` transitions have no `completion` strategy, so `resolve_outcome` (ticket a1b94ea4) infers `"needs_input"` for them by default. That makes `mock-happy` unable to find any success transition when running in the spec-writer state — it would always exit non-zero.

Fix: this ticket explicitly annotates both transitions with `outcome = "success"` in `apm-core/src/default/workflow.toml` and `.apm/workflow.toml`. This override is intentional — completing a spec IS the success outcome for the spec-writer agent, even though it involves no git merge.

### 1. WrapperContext extensions (`wrapper/mod.rs`)

Add two fields to `WrapperContext`:

```rust
pub root_path: PathBuf,       // project root (not the worktree)
pub current_state: String,    // ticket's state at spawn time
```

Callers in `start.rs` set `root_path = root.to_path_buf()` and `current_state = ticket.frontmatter.state.clone()`.

Add `APM_PROJECT_ROOT` to the wrapper contract env vars set by `ClaudeWrapper::spawn()` (both local and container paths). All four new wrappers also set it.

### 2. workflow.toml annotation (`apm-core/src/default/workflow.toml` and `.apm/workflow.toml`)

In each file, find the two transition blocks and add the `outcome` field:

```toml
# under [[workflow.states]] id = "in_design"
[[workflow.states.transitions]]
to      = "specd"
trigger = "manual"
outcome = "success"     # explicit override — spec completion is a success

# under [[workflow.states]] id = "ammend"
[[workflow.states.transitions]]
to      = "specd"
trigger = "manual"
outcome = "success"     # explicit override — amendment completion is a success
```

Both workflow files get the same change. Note: ticket a1b94ea4 also modifies workflow.toml. Implementers must merge cleanly; there is no logical conflict (a1b94ea4 adds `outcome` to transitions that match their implicit defaults; this ticket adds `outcome = "success"` to the two that override the default).

### 3. Module layout

Create `apm-core/src/wrapper/builtin/` with:

```
wrapper/builtin/mod.rs         — shared helpers, pub use for all built-ins
wrapper/builtin/mock_happy.rs  — MockHappyWrapper
wrapper/builtin/mock_sad.rs    — MockSadWrapper
wrapper/builtin/mock_random.rs — MockRandomWrapper
wrapper/builtin/debug.rs       — DebugWrapper
```

If d3b93b95's implementation placed `claude.rs` at `wrapper/claude.rs`, move it to `wrapper/builtin/claude.rs` and update all `use` paths. If d3b93b95 already used `wrapper/builtin/`, no move is needed.

Add `pub mod builtin;` to `wrapper/mod.rs`.

Update `resolve_builtin()` in `wrapper/mod.rs`:
```rust
"mock-happy"  => Some(Box::new(MockHappyWrapper)),
"mock-sad"    => Some(Box::new(MockSadWrapper)),
"mock-random" => Some(Box::new(MockRandomWrapper)),
"debug"       => Some(Box::new(DebugWrapper)),
```

### 4. Shared helpers in `wrapper/builtin/mod.rs`

#### `load_transitions_with_outcomes`

```rust
fn load_transitions_with_outcomes(
    ctx: &WrapperContext,
) -> anyhow::Result<Vec<(TransitionConfig, StateConfig)>>
```

1. `Config::load(&ctx.root_path)` — loads project config + workflow
2. Find the `StateConfig` matching `ctx.current_state` in the workflow; return `Err` if not found
3. Build a `HashMap<&str, &StateConfig>` keyed by state id for O(1) target lookup
4. For each transition in the current state, clone the pair `(TransitionConfig, target StateConfig)` and return the vec

#### `is_impl_mode`

```rust
fn is_impl_mode(transitions: &[(TransitionConfig, StateConfig)]) -> bool {
    transitions.iter().any(|(t, _)| t.completion != CompletionStrategy::None)
}
```

Returns `true` when at least one transition has a completion strategy (indicates the current state expects git work).

#### `write_and_spawn_script`

```rust
fn write_and_spawn_script(
    name: &str,
    script: &str,
    ctx: &WrapperContext,
) -> anyhow::Result<std::process::Child>
```

1. Write `script` to `<worktree>/.apm-mock-<name>-<rand_u16()>.sh` (using `rand_u16()` from d3b93b95)
2. Set permissions to 0o755 (`std::fs::set_permissions(..., Permissions::from_mode(0o755))`)
3. Build `Command::new("/bin/sh")`, arg = script path
4. Set all APM contract env vars (same set as ClaudeWrapper), including `APM_PROJECT_ROOT`
5. `.current_dir(&ctx.worktree_path)`, `.process_group(0)`
6. Redirect stdout + stderr to `File::create(&ctx.log_path)?` / `try_clone()`
7. `.spawn()`; return `Child`
8. The script's last line is `rm -f "$0"` (self-cleanup); no separate cleanup thread needed for the script file

#### `apm_bin`

```rust
fn apm_bin() -> anyhow::Result<String>
```

Returns `std::env::current_exe()?.to_str().ok_or_else(...)?.to_string()`. Used so scripts shell out to the same `apm` binary that spawned them.

#### `seed_from_ctx`

```rust
fn seed_from_ctx(ctx: &WrapperContext) -> u64
```

Reads `ctx.options.get("seed")` → parse as `u64`; on failure falls back to a random `u64` from `rand::thread_rng()`.

### 5. MockHappyWrapper (`mock_happy.rs`)

`spawn()` steps:
1. Call `load_transitions_with_outcomes(ctx)`.
2. Filter to success outcomes: `transitions.iter().filter(|(t, s)| resolve_outcome(t, s) == "success")`.
3. Match on count: 0 → `anyhow::bail!("mock-happy: no success-outcome transition from state '{}'", ctx.current_state)`, 2+ → `anyhow::bail!("mock-happy: {} success-outcome transitions; expected 1", n)`.
4. Extract `target_state = success_transitions[0].0.to.clone()`.
5. `let impl_mode = is_impl_mode(&transitions)`.
6. `let apm = apm_bin()?`.
7. Generate `script`:

   **Spec mode** (not impl mode):
   ```sh
   #!/bin/sh
   set -e
   APM="<apm_bin>"
   ID="<ticket_id>"
   "$APM" spec "$ID" --section "Problem" \
     --set "Mock spec — no real problem analyzed."
   printf '- [ ] Mock criterion 1\n- [ ] Mock criterion 2\n' \
     > ".apm-mock-ac-$$.txt"
   "$APM" spec "$ID" --section "Acceptance criteria" \
     --set-file ".apm-mock-ac-$$.txt"
   rm -f ".apm-mock-ac-$$.txt"
   "$APM" spec "$ID" --section "Out of scope" \
     --set "- Nothing in scope for this mock run"
   "$APM" spec "$ID" --section "Approach" \
     --set "Mock approach — no real implementation analyzed."
   "$APM" set "$ID" effort 1
   "$APM" set "$ID" risk 1
   printf '{"type":"tool_use","id":"mock-1","name":"write_spec","input":{}}\n'
   printf '{"type":"tool_use","id":"mock-2","name":"apm_state","input":{}}\n'
   "$APM" state "$ID" <target_state>
   rm -f "$0"
   ```

   **Impl mode** (is_impl_mode is true):
   ```sh
   #!/bin/sh
   set -e
   APM="<apm_bin>"
   ID="<ticket_id>"
   printf 'mock: placeholder implementation for ticket %s\n' "$ID" \
     > mock-implementation.txt
   git add mock-implementation.txt
   git commit -m "mock: placeholder commit for ticket $ID"
   printf '{"type":"tool_use","id":"mock-1","name":"git_commit","input":{}}\n'
   printf '{"type":"tool_use","id":"mock-2","name":"apm_state","input":{}}\n'
   "$APM" state "$ID" <target_state>
   rm -f "$0"
   ```

8. Call `write_and_spawn_script("happy", &script, ctx)`.

### 6. MockSadWrapper (`mock_sad.rs`)

`spawn()` steps:
1. `load_transitions_with_outcomes(ctx)`.
2. Filter to non-success: `resolve_outcome(t, s) != "success"`.
3. If empty: `anyhow::bail!("mock-sad: no non-success transitions from state '{}'", ctx.current_state)`.
4. `let seed = seed_from_ctx(ctx)`.
5. `let rng = rand::rngs::StdRng::seed_from_u64(seed)`.
6. Pick index = `(seed as usize) % eligible.len()` (no need for a shuffle; modulo gives deterministic pick for a given seed and list length).
7. `target_state = eligible[idx].0.to.clone()`.
8. Generate script (writes only Problem section, adds an open question, emits one JSONL event, calls `apm state`):

   ```sh
   #!/bin/sh
   set -e
   APM="<apm_bin>"
   ID="<ticket_id>"
   "$APM" spec "$ID" --section "Problem" \
     --set "Mock sad run — spec intentionally incomplete."
   "$APM" spec "$ID" --section "Open questions" \
     --set "- [ ] Mock open question — why did this fail?"
   printf '{"type":"tool_use","id":"mock-1","name":"write_partial_spec","input":{}}\n'
   "$APM" state "$ID" <target_state>
   rm -f "$0"
   ```

9. `write_and_spawn_script("sad", &script, ctx)`.

Note: `apm spec --section "Open questions"` must be a valid section name for `apm spec --set`. Verify the exact section name against the `apm spec` command's accepted sections; if "Open questions" isn't a named section, write to it via the ticket file directly or skip the question step.

### 7. MockRandomWrapper (`mock_random.rs`)

`spawn()` steps:
1. `load_transitions_with_outcomes(ctx)`.
2. If empty: `anyhow::bail!("mock-random: no valid transitions from state '{}'", ctx.current_state)`.
3. `seed_from_ctx(ctx)`.
4. Pick index via `seed as usize % all.len()`.
5. Inspect chosen transition's `resolve_outcome`:
   - `"success"` → generate the mock-happy script for the chosen `target_state` (spec or impl mode determined by `is_impl_mode(&all)`)
   - anything else → generate the mock-sad script for the chosen `target_state`
6. `write_and_spawn_script("random", &script, ctx)`.

Rather than duplicating script generation, extract private functions `happy_script(apm: &str, id: &str, target: &str, impl_mode: bool) -> String` and `sad_script(apm: &str, id: &str, target: &str) -> String` into `builtin/mod.rs` and call them from all three wrappers.

### 8. DebugWrapper (`debug.rs`)

`spawn()` steps:
1. No config loading. No transition resolution.
2. `apm_bin()`.
3. Script:

   ```sh
   #!/bin/sh
   env | grep '^APM_' >&2
   printf '\n=== SYSTEM PROMPT ===\n' >&2
   cat "$APM_SYSTEM_PROMPT_FILE" >&2
   printf '\n=== USER MESSAGE ===\n' >&2
   cat "$APM_USER_MESSAGE_FILE" >&2
   printf '{"type":"tool_use","id":"debug-1","name":"noop","input":{}}\n'
   rm -f "$0"
   ```

4. `write_and_spawn_script("debug", &script, ctx)`.

No `apm state` call. The ticket state is not modified.

### 9. Tests

**Unit tests in `wrapper/builtin/mod.rs` (or `wrapper/mod.rs`)**
- `resolve_builtin_mock_happy_returns_some` — `assert!(resolve_builtin("mock-happy").is_some())`
- `resolve_builtin_mock_sad_returns_some` — `assert!(resolve_builtin("mock-sad").is_some())`
- `resolve_builtin_mock_random_returns_some` — `assert!(resolve_builtin("mock-random").is_some())`
- `resolve_builtin_debug_returns_some` — `assert!(resolve_builtin("debug").is_some())`

**Integration tests in `apm-core/src/start.rs` `#[cfg(test)]` or a dedicated `tests/mock_wrappers.rs`**

Each test uses the same fixture helper (inline, no external files):
- Create a `tempfile::TempDir` for the project root
- Write a minimal `.apm/config.toml` with `agent = "mock-happy"` (or the wrapper under test)
- Copy (or write inline) the default workflow.toml to `.apm/workflow.toml` — including the two new `outcome = "success"` annotations
- Create a ticket file in `tickets/` in the correct starting state
- `git init`, add + commit the files (required for worktree and state operations)
- Build a `WrapperContext` pointing at the fixture with `current_state` set
- Call `spawn_worker(ctx)` (the private fn from d3b93b95), `child.wait()`, then read the updated ticket

Test list:
- `mock_happy_spec_mode_transitions_to_specd` — ticket in `in_design`; assert state = `specd`; assert all four spec section headers are present in the ticket file; assert effort = 1; assert risk = 1
- `mock_happy_impl_mode_creates_commit_and_transitions` — ticket in `in_progress`; assert state = `implemented`; assert `git log --oneline` in worktree has a commit containing "mock"
- `mock_happy_zero_success_transitions_exits_nonzero` — use a custom inline workflow where the current state has only non-success transitions; assert `child.wait().status.success() == false`; assert log contains "no success-outcome transition"
- `mock_sad_transitions_to_non_success_state` — ticket in `in_design`; assert resulting state is NOT `specd`; assert only Problem section is present in ticket spec
- `mock_sad_seed_reproducibility` — two separate spawn calls with `APM_OPT_SEED=42`; assert both end in the same target state
- `mock_random_seed_reproducibility` — same as above with `mock-random`
- `debug_does_not_change_state` — ticket in `in_design`; run debug; assert state is still `in_design`; assert log contains `APM_TICKET_ID`; assert log contains the system prompt text; assert log contains a line matching `{"type":"tool_use"...}`

All tests that require `apm` CLI calls in the script must resolve `current_exe()` correctly — in the test binary environment, `current_exe()` returns the test runner, not `apm`. Use the same workaround as the existing `spawn_worker_cwd_is_ticket_worktree` test: set a fixture env var or pass an `apm_override_bin` through `WrapperContext.options` (key `"apm_bin"`) that the mock script uses when set. The `apm_bin()` helper checks `ctx.options.get("apm_bin")` first, then falls back to `current_exe()`.

### Open questions


### Amendment requests

- [ ] Adopt `APM_BIN` as the canonical way for mocks to find the apm binary to shell out to. The `APM_BIN` env var is set by the spawn glue (per the amendment to ticket d3b93b95). Replace the spec's two competing approaches (`apm_bin()` via `std::env::current_exe()` and `apm_bin_from_ctx(ctx)`) with: read `APM_BIN` from the environment; if absent, exit non-zero with a clear error pointing at the wrapper contract. No fallback to PATH — explicit is safer than fragile. This also future-proofs custom wrappers that want to shell out to apm.
- [ ] Reconcile `APM_OPT_SEED` (env var name) with `ctx.options.get("seed")` (Rust internal field). The wrapper contract from d3b93b95 + 6cac8518 maps every entry in `[workers.options]` to an `APM_OPT_<KEY>` env var. The mocks should read `APM_OPT_SEED` from the environment (not via ctx options), consistent with how every other wrapper-specific option is consumed. Update Approach and AC accordingly. The user-facing config still writes `[workers.options] seed = "42"` — that's automatically translated to `APM_OPT_SEED=42` by the spawn glue.
- [ ] Add an AC for mock-random's "no valid transitions available" error case, symmetric with the one mock-sad already has. Right now mock-sad's spec mandates a non-zero exit if the eligible non-success transitions set is empty; mock-random has no equivalent AC for the (rarer) case where the current state has zero transitions at all. Tighten for symmetry.
- [ ] Mocks need their own per-agent instruction files so 7f5f73d5's resolution chain doesn't fall through to the hard error. Ship placeholder content at `apm-core/src/default/agents/mock-happy/apm.worker.md` and `apm.spec-writer.md`, plus the same for `mock-sad`, `mock-random`, and `debug`. Content can be one-line stubs ("This wrapper is a mock — see docs/agent-wrappers.md.") since the wrappers ignore the prompt anyway. Without these stubs, a project configured to use a mock would hit `7f5f73d5`'s level-5 hard error.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:04Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:57Z | groomed | in_design | philippepascal |
| 2026-05-01T00:08Z | in_design | ammend | philippepascal |
| 2026-05-01T00:09Z | ammend | in_design | philippepascal |
| 2026-05-01T00:21Z | in_design | specd | claude-0501-0009-cec0 |
| 2026-05-01T01:10Z | specd | ammend | philippepascal |
| 2026-05-01T01:43Z | ammend | in_design | philippepascal |