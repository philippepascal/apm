+++
id = "2973f8d1"
title = "Move compute_blocking_deps and compute_valid_transitions to apm_core"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2973f8d1-move-compute-blocking-deps-and-compute-v"
created_at = "2026-04-12T09:02:59.113894Z"
updated_at = "2026-04-12T11:04:03.581712Z"
epic = "1e706443"
target_branch = "epic/1e706443-refactor-apm-server-code-organization"
+++

## Spec

### Problem

`apm-server/src/main.rs` contains two business-logic functions that belong in `apm-core`, not in the HTTP server:

1. **`compute_blocking_deps()`** (lines ~416-443) — given a ticket and all tickets, computes which dependencies are blocking it. This is pure domain logic with no HTTP or server concerns. It duplicates reasoning that `apm_core::ticket` already partially implements (e.g., `dep_satisfied`, `build_reverse_index`).

2. **`compute_valid_transitions()`** (lines ~445-469) — given a ticket's current state and the workflow config, returns the list of valid next states. This duplicates `apm_core::state::available_transitions()`.

Both functions are called from ticket/epic handlers. Moving them to `apm_core` makes them testable independently and available to the CLI if needed. This should be done before extracting handlers, since the extracted handler modules will need to import these from `apm_core` rather than from a sibling module.

### Acceptance criteria

- [x] `apm_core` exports a `compute_blocking_deps(ticket, all_tickets, config) -> Vec<BlockingDep>` function
- [x] `apm_core` exports a `compute_valid_transitions(state, config) -> Vec<TransitionOption>` function
- [ ] `apm_core` exports the `BlockingDep` struct with fields `id: String` and `state: String`
- [ ] `apm_core` exports the `TransitionOption` struct with fields `to: String`, `label: String`, and `warning: Option<String>`
- [ ] Both functions are no longer defined in `apm-server/src/main.rs`
- [ ] `apm-server` call sites import and call the functions from `apm_core` with identical observable behaviour
- [ ] `apm-server` compiles without warnings after the move
- [ ] `apm-core` compiles without warnings after the addition
- [ ] `compute_blocking_deps` and `compute_valid_transitions` each have at least one unit test in `apm-core`

### Out of scope

- Merging or replacing the existing `apm_core::state::available_transitions()` function — `compute_valid_transitions` will coexist alongside it\n- Extracting HTTP handlers from `apm-server/src/main.rs` (covered by a separate ticket)\n- Changing the JSON shape of any HTTP response\n- Adding the CLI to call these functions\n- Moving any other functions from `main.rs` to `apm-core`

### Approach

**Target modules:** place both functions and their return-type structs in existing `apm-core` modules that already own the related logic.

- `compute_blocking_deps` + `BlockingDep` → `apm-core/src/ticket/ticket_util.rs` (alongside `dep_satisfied`, which it calls)
- `compute_valid_transitions` + `TransitionOption` → `apm-core/src/state.rs` (alongside `available_transitions`, which covers similar ground)

**Signature changes:** The current server-side signatures accept `root: &PathBuf` and call `Config::load(root)` internally. In `apm-core` the config is already loaded by callers; pass `config: &Config` directly instead. Update all call sites in `apm-server` to load config before calling (they already have access to `root`).

**Steps:**

1. **`apm-core/src/ticket/ticket_util.rs`**
   - Add `#[derive(serde::Serialize, Clone, Debug)]` struct `BlockingDep { pub id: String, pub state: String }`.
   - Add `pub fn compute_blocking_deps(ticket: &Ticket, all_tickets: &[Ticket], config: &Config) -> Vec<BlockingDep>` — body is identical to the server version minus the `Config::load` call (caller supplies `config`).
   - Add unit tests: one where all deps are satisfied (returns empty), one where a dep is in a non-terminal state (returns it), one where `depends_on` is absent (returns empty).

2. **`apm-core/src/state.rs`**
   - Add `#[derive(serde::Serialize, Clone, Debug)]` struct `TransitionOption { pub to: String, pub label: String, pub warning: Option<String> }`.
   - Add `pub fn compute_valid_transitions(state: &str, config: &Config) -> Vec<TransitionOption>` — body is identical to the server version minus the `Config::load` call.
   - Add unit tests: one for a state with transitions (returns expected options, default label applied when `tr.label` is empty), one for an unknown state (returns empty vec).

3. **`apm-core/src/lib.rs`**
   - Re-export `BlockingDep` and `compute_blocking_deps` via `pub use ticket::...` (or whatever the existing re-export path is).
   - Re-export `TransitionOption` and `compute_valid_transitions` from `state`.

4. **`apm-server/src/main.rs`**
   - Delete the local `BlockingDep`, `TransitionOption` struct definitions (lines ~57-98).
   - Delete the local `compute_blocking_deps` and `compute_valid_transitions` fn definitions (lines ~416-469).
   - At each of the 6 call sites (handlers: GET /tickets/{id}, PUT /transition/{id}, PUT /update-fields/{id}):
     - Load `config` with `Config::load(&root)` if not already available in scope.
     - Replace `compute_blocking_deps(ticket, &tickets, &root)` → `apm_core::compute_blocking_deps(ticket, &tickets, &config)`.
     - Replace `compute_valid_transitions(&root, &state_str)` → `apm_core::compute_valid_transitions(&state_str, &config)`.
   - Adjust any `use` imports accordingly.

**Constraints:**
- `serde` is already a dependency of `apm-core`, so deriving `Serialize` on the new structs requires no new dependencies.
- The server currently calls `compute_valid_transitions` inside `tokio::task::spawn_blocking`; after the move the function is still synchronous (no I/O), but callers will now need to load config outside the closure and pass it in — or load config inside the closure if the closure needs it. Keep the spawn_blocking wrapping unchanged; move the `Config::load` call inside the closure.
- Do not change the JSON shape of any HTTP response — `BlockingDep` and `TransitionOption` must serialize identically to before (field names, optionality of `warning`).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:36Z | groomed | in_design | philippepascal |
| 2026-04-12T09:39Z | in_design | specd | claude-0412-0936-6a40 |
| 2026-04-12T10:24Z | specd | ready | apm |
| 2026-04-12T11:04Z | ready | in_progress | philippepascal |