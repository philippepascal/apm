+++
id = "1d122a6a"
title = "fetch before read in close, state, take, next, list when aggressive"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "35767"
branch = "ticket/1d122a6a-fetch-before-read-in-close-state-take-ne"
created_at = "2026-03-30T19:50:51.401850Z"
updated_at = "2026-03-30T20:00:23.114115Z"
+++

## Spec

### Problem

When `config.sync.aggressive` is true, commands that read ticket state should fetch from remote first, and commands that write to a ticket branch should push after. This pattern is already partially implemented but inconsistently applied.

**Missing fetch before read/write (aggressive flag ignored):**

- `close` — writes a terminal state transition; working on stale data here is the highest risk (may close a ticket that was already modified remotely)
- `state` — same risk as close for any write transition
- `take` — reads agent field before handoff; could overwrite a remote reassignment
- `next` — returns stale priority ordering if branches have changed
- `list` — shows stale state if PRs were merged on GitHub without a local pull
- `verify` and `validate` — may report false positives/negatives against stale data

**Missing push after write (write goes to local branch only):**

- `spec` — no fetch, no push; writes spec section content to the branch but leaves it local
- `review` — fetches before opening editor, but never pushes after writing amendments or transition
- `set` — fetches before reading, but never pushes after writing field changes
- `close` — neither fetches nor pushes

The root cause: supervisors who merge PRs on GitHub and skip `git pull` quickly accumulate stale local state. All write commands must fetch before acting and push after when aggressive.

The existing pattern to follow:
```rust
let aggressive = config.sync.aggressive && !no_aggressive;
if aggressive {
    if let Err(e) = git::fetch_branch(root, &branch) {
        eprintln!("warning: fetch failed: {e:#}");
    }
}
// ... write changes ...
if aggressive {
    if let Err(e) = git::push_branch(root, &branch) {
        eprintln!("warning: push failed: {e:#}");
    }
}
```
Write commands fetch/push the specific ticket branch. Read-only commands use `git::fetch_all`.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T19:50Z | — | new | philippepascal |
| 2026-03-30T20:00Z | new | in_design | philippepascal |