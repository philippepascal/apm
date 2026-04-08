+++
id = "32a9a8b5"
title = "refactor: move sync candidate detection from sync.rs into apm-core"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "claude-0330-0245-main"
agent = "23686"
branch = "ticket/32a9a8b5-refactor-move-sync-candidate-detection-f"
created_at = "2026-03-30T14:27:39.762926Z"
updated_at = "2026-03-30T18:08:17.043447Z"
+++

## Spec

### Problem

The sync candidate detection logic in `apm/src/cmd/sync.rs` is tightly coupled to the CLI despite being pure domain logic. Two responsibilities are currently mixed in 172 lines:

1. **Candidate detection** — deciding which tickets are ready to accept (implemented tickets on merged branches) and which are ready to close (accepted tickets, or implemented tickets whose branch no longer exists). This is algorithmic logic with no user-facing I/O.
2. **User prompting** — the interactive `[y/N]` prompts and the orchestration of fetch/push and state transitions. This belongs in the CLI.

Because detection lives in the CLI binary, any future server component (`apm-serve`) that wants to preview a sync — "these N tickets would be accepted, these M would be closed" — cannot do so without shelling out to `apm sync`. Moving detection into `apm-core` makes it available to any caller.

The target shape is `apm_core::sync::detect(root, config)` returning a structured `Candidates` value, and `apm_core::sync::apply(root, config, candidates, author)` executing the state transitions. The CLI retains only I/O: fetch, push, interactive prompts, and calling `detect`/`apply`.

### Acceptance criteria

- [x] `apm_core::sync` is a public module in `apm-core` and re-exported from `apm-core/src/lib.rs`
- [x] `apm_core::sync::detect(root, config)` returns a `Candidates` value containing separate `Vec<AcceptCandidate>` and `Vec<CloseCandidate>`
- [x] `AcceptCandidate` holds the `Ticket` for an implemented ticket whose branch is merged into main
- [x] `CloseCandidate` holds the `Ticket` and a `reason` string for a ticket that is either in `accepted` state or in `implemented` state with its branch gone
- [x] `apm_core::sync::apply(root, config, candidates, author)` transitions each accept candidate to `accepted` state and closes each close candidate
- [x] `apm/src/cmd/sync.rs` no longer defines `AcceptCandidate`, `CloseCandidate`, or `detect_closeable`
- [x] `apm sync` produces identical output and behaviour to before this refactor
- [x] `cargo test --workspace` passes with no regressions

### Out of scope

- Interactive prompting logic (`prompt_accept`, `prompt_close`, `is_interactive`) — stays in the CLI
- The `run()` orchestration in `sync.rs` (git fetch, push, flag handling) — stays in the CLI
- Any changes to the sync workflow's observable behaviour or flags
- Building `apm-serve` — this ticket only prepares the library API for it
- Adding new sync capabilities (e.g. dry-run mode, filtering by state)

### Approach

1. **Create `apm-core/src/sync.rs`** with the following public API:
   - `pub struct AcceptCandidate { pub ticket: Ticket }`
   - `pub struct CloseCandidate { pub ticket: Ticket, pub reason: &'static str }`
   - `pub struct Candidates { pub accept: Vec<AcceptCandidate>, pub close: Vec<CloseCandidate> }`
   - `pub fn detect(root: &Path, config: &Config) -> Result<Candidates>` — lift the merged-branch loop (lines 33–53 of current `sync.rs`) and the full body of `detect_closeable` (lines 111–158) verbatim; the only difference is calling `git::` and `Ticket::` functions that are already in `apm-core`
   - `pub fn apply(root: &Path, config: &Config, candidates: &Candidates, author: &str) -> Result<()>` — for each accept candidate call a new `ticket::accept(root, config, id, author)` function; for each close candidate call the existing `ticket::close(root, config, id, None, author)`

2. **Add `ticket::accept`** to `apm-core/src/ticket.rs`: a minimal function that transitions a ticket from `implemented` to `accepted`, following the same pattern as `ticket::close` (read from branch, update state field, append history row, commit to branch). No merge or push — those stay in the CLI.

3. **Add `pub mod sync;`** to `apm-core/src/lib.rs`

4. **Update `apm/src/cmd/sync.rs`**:
   - Remove the `AcceptCandidate`, `CloseCandidate` struct definitions and `detect_closeable`
   - Import `apm_core::sync::{Candidates, detect, apply}`
   - Replace the inline accept-candidate detection loop and `detect_closeable` call with `let candidates = apm_core::sync::detect(root, &config)?;`
   - Replace the per-candidate `super::state::run(...)` and `ticket::close(...)` calls with `apm_core::sync::apply(root, &config, &candidates, "apm-sync")?`
   - Keep `prompt_accept`, `prompt_close`, `is_interactive`, and all flag handling unchanged

5. **Run `cargo test --workspace`** before committing

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:34Z | new | in_design | philippepascal |
| 2026-03-30T16:37Z | 65590 | claude-0330-1640-spec1 | handoff |
| 2026-03-30T16:41Z | in_design | specd | claude-0330-1640-spec1 |
| 2026-03-30T16:59Z | specd | ready | philippepascal |
| 2026-03-30T17:22Z | ready | in_progress | philippepascal |
| 2026-03-30T17:29Z | in_progress | implemented | claude-0330-1730-work1 |
| 2026-03-30T18:04Z | implemented | accepted | philippepascal |
| 2026-03-30T18:08Z | accepted | closed | apm-sync |