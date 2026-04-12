+++
id = "2eec2b83"
title = "Refactor clean.rs: extract epic cleanup and apply shared helpers"
state = "closed"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/2eec2b83-refactor-clean-rs-extract-epic-cleanup-a"
created_at = "2026-04-12T09:02:46.720913Z"
updated_at = "2026-04-12T17:10:57.420141Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
depends_on = ["d3ebdc0f", "aeacd066"]
+++

## Spec

### Problem

`apm/src/cmd/clean.rs` (296 lines) currently bundles two unrelated responsibilities:

1. **Local worktree/branch cleanup** (`run()`, ~70 lines) — identifies and removes worktrees and git branches for closed tickets. This logic belongs in `clean.rs`.

2. **Epic cleanup** (`run_epic_clean()`, ~130 lines) — lists `epic/*` branches, derives each epic's state, prompts the user for confirmation, deletes local and remote branches, and removes the entry from `.apm/epics.toml`. Epic cleanup is epic-domain logic and belongs in `epic.rs`.

The misplacement produces two concrete problems. First, `run_epic_clean()` calls `crate::cmd::epic::branch_to_title()` — creating a reverse dependency from `clean.rs` back into `epic.rs`. After ticket aeacd066 moves `branch_to_title()` and `epic_id_from_branch()` to `apm_core::epic`, that call becomes `apm_core::epic::branch_to_title()`, and the function that makes it should live alongside the other epic command handlers. Second, `run_epic_clean()` contains inline user-prompt sequences (print / flush / read_line / trim / eq_ignore_ascii_case) that ticket d3ebdc0f replaced with `util::prompt_yes_no()` everywhere — but the function is still in the wrong file.

The desired end-state: `clean.rs` owns only ticket-level cleanup; `epic.rs` owns `run_epic_clean()` as a peer of its other `run_*` helpers. The public `apm clean --epics` invocation and its observable behaviour do not change.

### Acceptance criteria

- [x] `run_epic_clean()` no longer exists in `apm/src/cmd/clean.rs`
- [x] `apm/src/cmd/epic.rs` contains a `pub(crate) fn run_epic_clean()` with the same signature as the removed function
- [x] `apm/src/cmd/clean.rs::run()` delegates to `crate::cmd::epic::run_epic_clean()` when the epics flag is set
- [x] All calls to `crate::cmd::epic::branch_to_title()` inside the moved function are replaced with `apm_core::epic::branch_to_title()`
- [x] All inline prompt sequences inside the moved function use `crate::util::prompt_yes_no()` instead of the raw print/flush/read_line pattern
- [x] Imports in `clean.rs` that were only needed by `run_epic_clean()` are removed
- [x] `apm clean --epics` lists done epic branches and prompts for each deletion, unchanged from before
- [x] `apm clean --epics --dry-run` prints what would be deleted without making any changes
- [x] `apm clean --epics --yes` skips all prompts and deletes without asking
- [x] `cargo test` passes across all crates

### Out of scope

- Moving any TOML or git-operation logic into apm_core (the core logic stays; only the CLI wrapper moves)\n- Changing the signature or behaviour of run_epic_clean() — this is a pure relocation\n- Replacing inline fetch patterns in clean.rs (covered by ticket d3ebdc0f)\n- Replacing the 3 inline epic-ID patterns in clean.rs (covered by ticket aeacd066)\n- Adding a new apm epic clean subcommand — the apm clean --epics flag and its wiring are untouched\n- Modifying apm-server

### Approach

1. Prerequisites d3ebdc0f and aeacd066 are already merged into the target branch before this ticket starts. Their changes provide `crate::util::prompt_yes_no()`, `apm_core::epic::epic_id_from_branch()`, and `apm_core::epic::branch_to_title()`, and have already replaced the 3 inline epic-ID patterns in `clean.rs`.

2. **In `apm/src/cmd/epic.rs`** — add a new function near the bottom, before the test block:
   ```rust
   pub(crate) fn run_epic_clean(
       root: &Path,
       config: &apm_core::config::Config,
       dry_run: bool,
       yes: bool,
   ) -> anyhow::Result<()> { ... }
   ```
   Copy the body verbatim from `clean.rs::run_epic_clean()`. Within the copied body:
   - Replace every `crate::cmd::epic::branch_to_title(...)` call with `apm_core::epic::branch_to_title(...)`.
   - Replace the inline prompt block (print / flush / read_line / trim / eq_ignore_ascii_case) with `crate::util::prompt_yes_no(\"...\")? `.
   - Adjust `use` imports as needed (remove items already in scope in `epic.rs`, add any missing ones).

3. **In `apm/src/cmd/clean.rs`**:
   - Delete the `run_epic_clean()` function (all ~130 lines, from its `fn` header to its closing `}`).
   - In `run()`, change the call site from `run_epic_clean(root, &config, dry_run, yes)?` to `crate::cmd::epic::run_epic_clean(root, &config, dry_run, yes)?`.
   - Remove `use` imports that were only consumed by the deleted function (e.g. any `apm_core::epic::*` items no longer referenced in `clean.rs`, and `crate::cmd::epic::branch_to_title` if present).

4. Run `cargo build` and `cargo test --workspace` to confirm nothing is broken.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:09Z | new | groomed | apm |
| 2026-04-12T09:24Z | groomed | in_design | philippepascal |
| 2026-04-12T09:28Z | in_design | specd | claude-0412-0924-ca90 |
| 2026-04-12T10:24Z | specd | ready | apm |
| 2026-04-12T10:46Z | ready | in_progress | philippepascal |
| 2026-04-12T10:50Z | in_progress | implemented | claude-0412-1046-0fc8 |
| 2026-04-12T17:10Z | implemented | closed | philippepascal |
