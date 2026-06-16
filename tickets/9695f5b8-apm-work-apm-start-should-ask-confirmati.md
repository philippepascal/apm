+++
id = "9695f5b8"
title = "apm work, apm start, should ask confirmation if a ticket in their actionable list is in an epic that needs refresh"
state = "implemented"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9695f5b8-apm-work-apm-start-should-ask-confirmati"
created_at = "2026-06-16T18:08:19.018981Z"
updated_at = "2026-06-16T20:54:23.637661Z"
depends_on = ["ee5011b6"]
+++

## Spec

### Problem

When `apm start <id>` or `apm work` picks a ticket whose parent epic is behind the default branch (`behind_count > 0`), they proceed silently. A worker spawned under a stale epic branch may build on a snapshot that is missing recent commits, then collide with `apm epic refresh` later — creating unnecessary merge conflicts or duplicate work.

The same gap exists in the web UI. `WorkEngineControls` shows an epic dropdown and a "Start" button but gives no indication when the chosen epic (or any epic with actionable tickets, in "All" mode) is stale. A supervisor starting the work engine through the UI has no visual cue that a refresh is needed first.

### Acceptance criteria

- [x] `apm start <id>` prints a warning and prompts for confirmation (default yes) when the ticket's epic has `behind_count > 0` and stdout is a terminal; the ticket is NOT started if the user answers "n".
- [x] `apm start <id>` writes a warning to stderr and proceeds without prompting when stdout is not a terminal and the ticket's epic is stale.
- [x] `apm start <id>` proceeds normally without any warning when the ticket has no epic, or the epic is up to date.
- [x] `apm start --next` prints a warning and prompts for confirmation (default yes) when the selected ticket's epic has `behind_count > 0` and stdout is a terminal; the ticket is NOT started if the user answers "n".
- [x] `apm start --next` writes a warning to stderr and proceeds without prompting when stdout is not a terminal and the selected ticket's epic is stale.
- [x] `apm work` (non-daemon) logs a warning line to stdout when it dispatches a ticket whose epic has `behind_count > 0`, before printing the "Worker spawned" line.
- [x] `apm work --daemon` logs the same warning line when dispatching from a stale epic.
- [x] The web UI `WorkEngineControls` shows a visible warning near the "Start" button when the selected epic has `behind_count > 0`.
- [x] The web UI `WorkEngineControls` shows a visible warning near the "Start" button when "All" is selected and at least one epic has `behind_count > 0`.
- [x] The warning message in all contexts includes the epic ID and the number of commits it is behind.

### Out of scope

- Blocking (hard-erroring) `apm start` when an epic is stale — this ticket only adds a warning and a confirmable prompt.
- Automatically running `apm epic refresh` before starting — the user must refresh manually.
- Checking freshness for tickets that have no epic (i.e., tickets on the default branch).
- Any changes to `apm work --daemon` interactive prompting — daemon mode is inherently non-interactive; it logs a warning and continues.
- Filtering which epics trigger the warning based on whether they have actionable tickets — any stale epic triggers the warning.

### Approach

#### Helper: `apm-core/src/epic.rs` — `ticket_epic_staleness`

Add:

```rust
pub fn ticket_epic_staleness(root: &Path, epic_id: &str) -> Result<Option<usize>>
```

- Loads config to get `default_branch`.
- Calls `find_epic_branch(root, epic_id)`. Returns `None` if the epic has no local or remote branch.
- Calls `merge_tree_status(root, &default_branch, &epic_branch)`.
- Returns `None` if `ahead == 0`; otherwise `Some(ahead)`.

Callers supply the `epic_id` string directly (read from `frontmatter.epic`), so there is no ticket loading inside this function.

#### `apm/src/cmd/start.rs` — `run()` (`apm start <id>`)

Before calling `apm_core::start::run()`:

1. Load config and all tickets; resolve `id_arg` to `id`; read `frontmatter.epic`.
2. If the ticket has `Some(epic_id)`, call `ticket_epic_staleness(root, &epic_id)`.
3. If `Some(ahead)`:
   - If `std::io::stdout().is_terminal()`: print `"Warning: epic {epic_id} is {ahead} commit(s) behind {default_branch}. Run \`apm epic refresh {epic_id}\` first. Start anyway? [Y/n] "`, flush stdout, read a line. Return `Ok(())` without calling `run()` if the user answers `"n"` or `"N"`.
   - Otherwise: `eprintln!("warning: epic {epic_id} is {ahead} commit(s) behind the default branch")` and fall through.
4. Call `apm_core::start::run()`. The state transition happens inside this call.

Loading tickets twice (once here for the pre-flight check, once inside `apm_core::start::run`) is acceptable.

#### `apm-core/src/start.rs` — `peek_next_candidate` (new public fn) and `run_next()`

Add:

```rust
pub fn peek_next_candidate(root: &Path) -> Result<Option<(String, Option<String>)>>
```

This function reuses the ticket-picking logic from `run_next` (load config, load all tickets, apply blocked-epic filter, call `ticket::pick_next`) but performs no state transition. It returns `(ticket_id, epic_id)`.

#### `apm/src/cmd/start.rs` — `run_next()` (`apm start --next`)

Before calling `apm_core::start::run_next()`:

1. Call `apm_core::start::peek_next_candidate(root)`.
2. If `Some((_, Some(epic_id)))`, call `ticket_epic_staleness(root, &epic_id)`.
3. If stale: same tty / non-tty handling as `run()` above (prompt on tty, `eprintln!` on non-tty). Return `Ok(())` without calling `run_next()` if the user answers `"n"`.
4. Otherwise call `apm_core::start::run_next()`.

There is a small TOCTOU window between `peek_next_candidate` and `run_next` re-picking the same candidate; this is acceptable.

#### `apm-core/src/start.rs` — `spawn_next_worker()` (`apm work` / `apm work --daemon`)

After selecting `candidate` (line 804) and before calling `run()` (line 819):

1. If `candidate.frontmatter.epic` is `Some(ref epic_id)`, call `ticket_epic_staleness(root, epic_id)`.
2. If `Some(ahead)`: push `format!("warning: epic {epic_id} is {ahead} commit(s) behind the default branch")` onto `messages`.
3. Proceed with `run()` unconditionally — no prompting in daemon/work mode.

The warning appears in `messages` before the state-transition line (which is pushed at line 825), so it prints before "Worker spawned". No new fields on `RunNextOutput` or `StartOutput` are needed.

#### `apm-ui/src/components/WorkEngineControls.tsx`

Update the local `Epic` type to include `behind_count: number`. The `/api/epics` endpoint already returns this field via `EpicSummary`.

Add staleness logic before the "Start" button:

```tsx
const staleWarning = (() => {
  if (selectedEpic) {
    const ep = epics.find(e => e.id === selectedEpic)
    if (ep && ep.behind_count > 0) {
      return `Epic ${ep.id.slice(0, 8)} is ${ep.behind_count} commit(s) behind — run \`apm epic refresh ${ep.id}\` first.`
    }
  } else {
    const staleEpics = epics.filter(e => e.behind_count > 0)
    if (staleEpics.length > 0) {
      return `${staleEpics.length} epic(s) need refresh — workers may start on stale branches.`
    }
  }
  return null
})()
```

Render `staleWarning` as an amber warning `<span>` between the epic selector and the "Start" button. Do not disable the button — this is a warning, not a blocker.

#### Tests

- Unit test in `apm-core/src/epic.rs` for `ticket_epic_staleness`: create a temp git repo with an epic branch that is behind `main`, call the function with the epic ID, assert the returned `ahead` count matches.
- Integration test in `apm/tests/integration.rs`: run `apm start <id>` with stdout piped (non-tty), pointing at a ticket whose epic is behind the default branch; assert stderr contains `"warning: epic"` and the ticket transitions to `in_progress`.

### Open questions


### Amendment requests

- [x] Fix wrong return type in the Approach. It claims apm_core::start::run_next() returns StartOutput and proposes adding a stale_warning field to StartOutput. It actually returns RunNextOutput (apm-core/src/start.rs:294, returned at :586) — a distinct struct from StartOutput (:280). The CLI run_next (apm/src/cmd/start.rs:23) prints out.messages, so the surface for run_next is RunNextOutput.messages, not StartOutput. Also note: gating the prompt via a field set inside run() is too late — run() performs the state transition at start.rs:445, but AC1 requires prompting BEFORE the transition. Correct the struct name and drop the StartOutput.stale_warning idea.
- [x] Resolve the self-contradictory Approach. The start.rs section reverses itself three times ('Add stale_warning to StartOutput' -> 'keep it simpler' -> 'Actually this creates a timing issue: keep the pre-flight call in the CLI layer'), leaving no final design. Collapse to one coherent design an implementer can follow: for 'apm start', do the pre-flight ticket_epic_staleness() check and prompt in the CLI layer (apm/src/cmd/start.rs::run) BEFORE calling apm_core::start::run(), so the transition is gated; for the dispatch paths (apm work, --daemon), push the warning into the existing messages: &mut Vec<String> in spawn_next_worker. Delete the abandoned StartOutput.stale_warning branch entirely.
- [x] Pin the behaviour of 'apm start --next' (CLI run_next). The ACs cover 'apm start <id>', 'apm work', 'apm work --daemon', and the web UI, but say nothing about 'apm start --next', which the Approach discusses at length — leaving it an orphan. Either add an AC for it (it is interactive-capable like 'apm start': prompt on tty, warn on non-tty) or explicitly list it in Out of scope.
- [x] Minor: reuse the existing helper find_epic_branch(root, short_id) -> Option<String> (apm-core/src/epic.rs:199) to resolve an epic id to its branch, instead of manually scanning epic_branches(root) for epic/{epic_id}-* as the Approach currently describes.

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-16T18:08Z | — | new | philippepascal |
| 2026-06-16T18:09Z | new | groomed | philippepascal |
| 2026-06-16T18:13Z | groomed | in_design | philippepascal |
| 2026-06-16T18:19Z | in_design | specd | claude |
| 2026-06-16T19:29Z | specd | ammend | philippepascal |
| 2026-06-16T19:30Z | ammend | in_design | philippepascal |
| 2026-06-16T19:35Z | in_design | specd | claude |
| 2026-06-16T20:24Z | specd | ready | philippepascal |
| 2026-06-16T20:40Z | ready | in_progress | philippepascal |
| 2026-06-16T20:54Z | in_progress | implemented | claude |
