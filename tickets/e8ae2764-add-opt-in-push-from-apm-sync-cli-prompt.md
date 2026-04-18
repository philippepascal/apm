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

Checkboxes; each one independently testable.

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