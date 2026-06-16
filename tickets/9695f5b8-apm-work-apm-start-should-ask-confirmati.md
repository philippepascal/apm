+++
id = "9695f5b8"
title = "apm work, apm start, should ask confirmation if a ticket in their actionable list is in an epic that needs refresh"
state = "ammend"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9695f5b8-apm-work-apm-start-should-ask-confirmati"
created_at = "2026-06-16T18:08:19.018981Z"
updated_at = "2026-06-16T19:29:11.571834Z"
+++

## Spec

### Problem

When `apm start <id>` or `apm work` picks a ticket whose parent epic is behind the default branch (`behind_count > 0`), they proceed silently. A worker spawned under a stale epic branch may build on a snapshot that is missing recent commits, then collide with `apm epic refresh` later — creating unnecessary merge conflicts or duplicate work.

The same gap exists in the web UI. `WorkEngineControls` shows an epic dropdown and a "Start" button but gives no indication when the chosen epic (or any epic with actionable tickets, in "All" mode) is stale. A supervisor starting the work engine through the UI has no visual cue that a refresh is needed first.

### Acceptance criteria

- [ ] `apm start <id>` prints a warning and prompts for confirmation (default yes) when the ticket's epic has `behind_count > 0` and stdout is a terminal; the ticket is NOT started if the user answers "n".
- [ ] `apm start <id>` writes a warning to stderr and proceeds without prompting when stdout is not a terminal and the ticket's epic is stale.
- [ ] `apm start <id>` proceeds normally without any warning when the ticket has no epic, or the epic is up to date.
- [ ] `apm work` (non-daemon) logs a warning line to stdout when it dispatches a ticket whose epic has `behind_count > 0`, before printing the "Dispatched worker" line.
- [ ] `apm work --daemon` logs the same warning line when dispatching from a stale epic.
- [ ] The web UI `WorkEngineControls` shows a visible warning near the "Start" button when the selected epic has `behind_count > 0`.
- [ ] The web UI `WorkEngineControls` shows a visible warning near the "Start" button when "All" is selected and at least one epic has `behind_count > 0`.
- [ ] The warning message in all contexts includes the epic ID and the number of commits it is behind.

### Out of scope

- Blocking (hard-erroring) `apm start` when an epic is stale — this ticket only adds a warning and a confirmable prompt.
- Automatically running `apm epic refresh` before starting — the user must refresh manually.
- Checking freshness for tickets that have no epic (i.e., tickets on the default branch).
- Any changes to `apm work --daemon` interactive prompting — daemon mode is inherently non-interactive; it logs a warning and continues.
- Filtering which epics trigger the warning based on whether they have actionable tickets — any stale epic triggers the warning.

### Approach

#### New helper: `apm-core/src/epic.rs`

Add `ticket_epic_staleness(root, ticket_id) -> Result<Option<(String, usize)>>`:
- Loads config and all tickets from git.
- Resolves `ticket_id` to a full ID, finds the ticket, reads `frontmatter.epic`.
- Returns `None` if the ticket has no epic.
- Calls `epic_branches(root)` to find the branch matching `epic/{epic_id}-*`.
- Calls `merge_tree_status(root, &config.project.default_branch, &epic_branch)`.
- Returns `None` if `ahead == 0`; otherwise `Some((epic_id, ahead))`.

This function is pure (no side effects, no git mutations) and cheap to call before committing.

#### `apm/src/cmd/start.rs` — `run()` function

Before calling `apm_core::start::run()`, call `ticket_epic_staleness()`:
- If it returns `Some((epic_id, ahead))`:
  - If `std::io::stdout().is_terminal()`: call `prompt_yes_no_default_yes()` with message `"Warning: epic {epic_id} is {ahead} commit(s) behind {default_branch}. Run \`apm epic refresh {epic_id}\` first. Start anyway? [Y/n] "`. Return `Ok(())` if user answers "n".
  - If not a terminal: `eprintln!("warning: epic {epic_id} is {ahead} commit(s) behind the default branch")` and proceed.
- `run_next()` gets the same treatment: after resolving which ticket `run_next` would pick, call the check. `run_next()` internally calls `apm_core::start::run_next()` which returns a `StartOutput`; the staleness check needs the ticket id, which can be surfaced by reading `StartOutput` fields. Alternatively, add a helper `next_ticket_id(root) -> Result<Option<String>>` to resolve the ID before starting. (Simpler: do the check inside `apm_core::start::run_next()` and add a `stale_warning: Option<String>` to `RunNextOutput`; CLI layer prints it.)

Actually, keep it simpler: for `run_next()` in the CLI, add `stale_warning` to `StartOutput` in `apm-core` and surface it in the CLI's output rather than doing a pre-flight call. This avoids loading tickets twice.

#### `apm-core/src/start.rs` — `StartOutput` struct and `run()`

- Add `pub stale_warning: Option<String>` to `StartOutput`.
- After loading the ticket and before the state transition, call `ticket_epic_staleness()`. If stale, store `Some("epic {id} is {ahead} commit(s) behind {default_branch}")` in a local; set it on `StartOutput` at the end.
- The CLI (`apm/src/cmd/start.rs`) then checks `out.stale_warning`: if terminal, it should have already confirmed before calling (see above for `start`). For `run_next` / `spawn_next_worker`, it logs the warning.

Actually this creates a timing issue: for `apm start`, we want to prompt BEFORE the state transition. So for `apm start` specifically, keep the pre-flight call in the CLI layer. For `run_next` / `spawn_next_worker`, add `stale_warning` to `StartOutput` / messages and log post-hoc.

#### `apm-core/src/start.rs` — `spawn_next_worker()`

After picking `candidate` at line 800 and before calling `run()` at line 819:
- Call `ticket_epic_staleness(root, &id)`.
- If `Some((epic_id, ahead))`, push `format!("warning: epic {epic_id} is {ahead} commit(s) behind the default branch")` to `messages`.
- Proceed with `run()` unconditionally (no prompt; `spawn_next_worker` is inherently non-interactive).

The `apm work` loop in `apm/src/cmd/work.rs` already prints all `messages` via `spawn_next_worker` wrapper, so the warning surfaces automatically.

#### `apm-ui/src/components/WorkEngineControls.tsx`

Update the local `Epic` type:
```ts
type Epic = { id: string; title: string; branch: string; behind_count: number; conflicts: boolean }
```
The `/api/epics` endpoint already returns these fields (see `EpicSummary` in `apm-server/src/models.rs`).

Add staleness logic before the "Start" button (in the JSX):
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

Render `staleWarning` as an amber warning `<span>` placed between the epic selector and the "Start" button. Do not disable the button — it is a warning, not a blocker.

#### Tests

- Unit test in `apm-core/src/epic.rs` for `ticket_epic_staleness()`: create a temp git repo with an epic branch, ticket with the epic ID, and verify the function returns the correct `ahead` count.
- Integration test in `apm/tests/integration.rs`: run `apm start <id>` with a pipe (non-tty) pointing at a ticket with a stale epic; verify stderr contains the warning and the ticket transitions normally.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-16T18:08Z | — | new | philippepascal |
| 2026-06-16T18:09Z | new | groomed | philippepascal |
| 2026-06-16T18:13Z | groomed | in_design | philippepascal |
| 2026-06-16T18:19Z | in_design | specd | claude |
| 2026-06-16T19:29Z | specd | ammend | philippepascal |
