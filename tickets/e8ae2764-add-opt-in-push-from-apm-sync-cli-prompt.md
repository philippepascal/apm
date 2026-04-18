+++
id = "e8ae2764"
title = "Add opt-in push from apm sync CLI prompt, flag, and UI button"
state = "specd"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e8ae2764-add-opt-in-push-from-apm-sync-cli-prompt"
created_at = "2026-04-18T02:21:50.164931Z"
updated_at = "2026-04-18T06:49:14.792956Z"
depends_on = ["b15354a6"]
+++

## Spec

### Problem

`apm sync` deliberately never pushes automatically, following a multi-user-safety principle. When `<default>` is ahead of `origin/<default>`, it prints guidance ("run `git push` when ready") and exits. This is correct for shared repos but creates unnecessary friction when a sole developer wants to push immediately: they must alt-tab, run `git push`, then re-run `apm sync` to pick up the close candidates that are now reachable — three context switches for one intent.

The same gap exists for ahead ticket/* and epic/* branches surfaced by `sync_non_checked_out_refs`: the user sees "push when ready: git push origin <slug>" for each branch but cannot act from inside sync.

The desired behaviour is a user-authorized push path on both surfaces — CLI and UI — that is always opt-in and never automatic by default. The existing guardrails (no push when diverged, no push mid-merge, no push in offline mode) must be preserved unconditionally.

### Acceptance criteria

- [ ] **CLI — default branch push**

- [ ] When `<default>` is ahead of `origin/<default>`, stdin is a TTY, and `--quiet` is not set, `apm sync` prints a prompt: `push <default> to origin now? [y/N]`
- [ ] Answering `y` at the prompt causes `apm sync` to run `git push origin <default>` and re-evaluate close candidates against the updated remote state
- [ ] Answering `N` (or pressing Enter) causes `apm sync` to proceed without pushing and print the MAIN_AHEAD guidance line
- [ ] When stdin is not a TTY, `apm sync` does not prompt, does not push, and prints the MAIN_AHEAD guidance line
- [ ] `apm sync --push-default` pushes `<default>` when it is ahead without prompting, and re-evaluates close candidates after
- [ ] `apm sync --push-default` does not push and emits the existing MAIN_DIVERGED guidance when `<default>` has diverged from `origin/<default>`
- [ ] `apm sync --push-default --offline` does not attempt any push

- [ ] **CLI — ticket/epic branch push**

- [ ] When `sync_non_checked_out_refs` finds one or more ahead ticket/* or epic/* branches, stdin is a TTY, and `--quiet` is not set, `apm sync` prints a single bundled prompt: `push N ahead branch(es) to origin now? [y/N]`
- [ ] Answering `y` at the bundled prompt causes `apm sync` to push each ahead branch and print a summary line
- [ ] `apm sync --push-refs` pushes all ahead ticket/* and epic/* branches without prompting
- [ ] `apm sync --push-refs --offline` does not attempt any push

- [ ] **Server**

- [ ] POST `/api/sync` with body `{ "push_default": true }` pushes `<default>` when it is ahead, appends a push confirmation line to the returned `log`, and includes newly closeable tickets in `closed`
- [ ] POST `/api/sync` with body `{ "push_default": true }` when `<default>` is diverged does not push and returns the diverged warning in `log`
- [ ] POST `/api/sync` with body `{ "push_refs": true }` pushes all ahead ticket/* and epic/* branches and appends per-branch confirmation lines to `log`
- [ ] The sync response includes `ahead_branches: string[]` listing branch short names that are ahead of origin after the sync completes
- [ ] The sync response includes `default_branch: string` with the configured default branch name

- [ ] **UI**

- [ ] When the sync response `ahead_branches` contains the value of `default_branch`, the Sync modal renders a `Push <default>` button below the log
- [ ] Clicking `Push <default>` sends POST `/api/sync` with `{ "push_default": true }`, replaces the log display with the new response, and invalidates ticket queries
- [ ] When `ahead_branches` contains non-default branches, the Sync modal renders a `Push N ahead branch(es)` button
- [ ] Clicking the ahead-branches button sends POST `/api/sync` with `{ "push_refs": true }` and updates the log display
- [ ] The Sync modal contains a checkbox labelled "Auto-push `<default>` when ahead"; it is unchecked by default
- [ ] When the auto-push checkbox is checked, the initial sync POST is sent with `{ "push_default": true }`, bypassing the button
- [ ] The auto-push checkbox state persists across page reloads (stored in `localStorage`)

### Out of scope

- Automatic push without any user action (the no-auto-push default is never changed for users who haven't opted in)
- Push from any command other than `apm sync` (start, close, state transitions, etc. are unaffected)
- Changing the push remote target — always `origin`
- Force-push or push with lease (plain `git push origin <branch>` only)
- Per-branch UI push buttons for individual ticket/epic branches — the UI uses a single bundled "Push N ahead branches" button
- User preference storage on the server / in `.apm/local.toml` — localStorage only
- Scheduling or automating push+sync via `apm` cron facilities
- Diverged-branch resolution — existing MAIN_DIVERGED and TICKET_OR_EPIC_DIVERGED guidance is unchanged
- The MAIN_AHEAD message text update — that is ticket b15354a6

### Approach

Four files change. The dependency order is: core → CLI and server in parallel → UI.

**1. `apm-core/src/git_util.rs`**

Change `sync_default_branch(root, default_branch, warnings) -> ()` to return `bool`: `true` when local `<default>` is Ahead of origin (and was not pushed). All other code paths return `false`. The existing warning-push into `warnings` stays; the return value is an additional signal for the caller.

Change `sync_non_checked_out_refs(root, warnings) -> ()` to return `Vec<String>`: the list of ahead branch short names (e.g. `["ticket/abc-feature", "epic/my-epic"]`). These are the branches that were in `Ahead` state during the scan. The existing warning messages continue to be appended to `warnings` for display.

**2. `apm/src/cmd/sync.rs`**

Add two new `bool` parameters to `run()`, wired from new CLI flags:
- `push_default: bool` — flag `--push-default`
- `push_refs: bool` — flag `--push-refs`

TTY detection: `std::io::IsTerminal::is_terminal(&std::io::stdin())` (stable Rust ≥1.70; no new dependency).

After calling `sync_default_branch` (inside the existing `if !offline` block):
- If returned `true` (default is ahead):
  - Resolve "should push": `push_default` flag is set, OR (is_tty AND not quiet AND user answers `y` to `"push <default> to origin now? [y/N]"`)
  - If should push: call `apm_core::git::push_branch(root, default_branch)`. `git push` updates the local remote-tracking ref automatically, so no second fetch is needed. Then fall through to the close-candidate detection that already follows in `run()` — it will now see the updated remote state.
  - If should not push: the MAIN_AHEAD warning was already appended to warnings by `sync_default_branch`; print it as today.

After calling `sync_non_checked_out_refs` (inside the same `if !offline` block):
- If returned non-empty `ahead_branches`:
  - Resolve "should push": `push_refs` flag is set, OR (is_tty AND not quiet AND user answers `y` to `"push N ahead branch(es) to origin now? [y/N]"`)
  - If should push: call `push_branch(root, branch)` for each branch in the returned list; print a summary line (unless quiet). Diverged branches are never in the ahead list, so no additional divergence check is needed here.

The offline guard is implicit: both blocks are already inside `if !offline { … }`, so no additional check is needed.

**3. `apm-server/src/handlers/maintenance.rs`**

Add a request body struct (derive `Deserialize`, `Default`):
```rust
#[derive(Deserialize, Default)]
struct SyncRequest {
    push_default: Option<bool>,
    push_refs: Option<bool>,
}
```
Extract it from the Axum handler (optional JSON body; use `Option<Json<SyncRequest>>`).

Extend the existing response struct with two fields:
```rust
ahead_branches: Vec<String>,
default_branch: String,
```
Populate `default_branch` from `config.project.default_branch`.

In `sync_handler`, after `sync_default_branch` returns:
- Capture the `bool` return value.
- If `push_default == Some(true)` and the branch is ahead: call `push_branch(root, default_branch)`, append `"pushed <default> to origin"` to `log`.
- If `push_default == Some(true)` and the branch is diverged: do not push; the diverged warning is already in warnings/log.

After `sync_non_checked_out_refs` returns:
- Capture the `Vec<String>` of ahead branch names.
- If `push_refs == Some(true)` and the list is non-empty: push each, append per-branch lines to `log`.

After all sync and push: populate `ahead_branches` with any branches still ahead of origin (re-run `classify_branch` for the default branch and for each ticket/epic branch that was ahead but not pushed — i.e. when push was not requested or the push was not for that surface). If all were pushed, `ahead_branches` is empty.

**4. `apm-ui/src/components/SyncModal.tsx`**

Update the response type interface to add `ahead_branches: string[]` and `default_branch: string`.

After a sync response, derive two display flags:
- `defaultIsAhead = result.ahead_branches.includes(result.default_branch)`
- `refAheadCount = result.ahead_branches.filter(b => b !== result.default_branch).length`

Render conditionally below the log:
- If `defaultIsAhead` and auto-push preference is off: `<button>Push {result.default_branch}</button>` — on click, POST `/api/sync` with `{ push_default: true }`, update state with new response, call `invalidateQueries`.
- If `refAheadCount > 0`: `<button>Push {refAheadCount} ahead branch{refAheadCount > 1 ? 'es' : ''}</button>` — on click, POST `/api/sync` with `{ push_refs: true }`, same update.

Add a settings row above the Run button:
```tsx
<label>
  <input type="checkbox" checked={autoPush} onChange={…} />
  Auto-push {defaultBranch ?? 'default'} when ahead
</label>
```
Persist with `localStorage.getItem/setItem('apm:sync:auto-push-default')`. Initialize `autoPush` state from localStorage on mount. When `autoPush` is true, include `push_default: true` in every initial sync POST body — the buttons are not shown in this case since the push is already happening.

The `defaultBranch` for the label can be read from the last sync result's `default_branch` field (or show a generic label before the first sync).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-18T02:21Z | — | new | philippepascal |
| 2026-04-18T02:23Z | new | groomed | apm |
| 2026-04-18T02:33Z | groomed | in_design | philippepascal |
| 2026-04-18T06:38Z | in_design | ready | apm |
| 2026-04-18T06:39Z | ready | groomed | apm |
| 2026-04-18T06:39Z | groomed | in_design | philippepascal |
| 2026-04-18T06:42Z | in_design | groomed | apm |
| 2026-04-18T06:42Z | groomed | in_design | philippepascal |
| 2026-04-18T06:49Z | in_design | specd | claude-0418-0642-8788 |
