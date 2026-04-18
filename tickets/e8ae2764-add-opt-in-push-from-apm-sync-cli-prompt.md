+++
id = "e8ae2764"
title = "Add opt-in push from apm sync CLI prompt, flag, and UI button"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e8ae2764-add-opt-in-push-from-apm-sync-cli-prompt"
created_at = "2026-04-18T02:21:50.164931Z"
updated_at = "2026-04-18T06:42:41.556290Z"
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

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

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