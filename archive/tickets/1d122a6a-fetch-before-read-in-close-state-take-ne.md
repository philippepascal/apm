+++
id = "1d122a6a"
title = "fetch before read in close, state, take, next, list when aggressive"
state = "closed"
priority = 0
effort = 4
risk = 2
author = "philippepascal"
agent = "65756"
branch = "ticket/1d122a6a-fetch-before-read-in-close-state-take-ne"
created_at = "2026-03-30T19:50:51.401850Z"
updated_at = "2026-03-31T05:04:54.282593Z"
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

- [x] `apm next` fetches all remote branches before reading ticket state when aggressive mode is on
- [x] `apm list` fetches all remote branches before reading ticket state when aggressive mode is on
- [x] `apm close` fetches the ticket branch before reading and pushes after writing when aggressive mode is on
- [x] `apm take` fetches the ticket branch before reading (in addition to the existing push after write) when aggressive mode is on
- [x] `apm spec` fetches the ticket branch before reading and pushes after writing when aggressive mode is on
- [x] `apm set` pushes the ticket branch after writing field changes when aggressive mode is on (fetch before is already implemented)
- [x] `apm review` pushes the ticket branch after writing amendments when aggressive mode is on (fetch before is already implemented)
- [x] `apm verify` fetches all remote branches before reading ticket state when aggressive mode is on
- [x] `apm validate` fetches all remote branches before reading ticket state when aggressive mode is on
- [x] All affected commands accept a `--no-aggressive` flag that suppresses fetch/push behaviour regardless of config
- [x] Fetch/push failures emit a `warning: fetch/push failed: ...` message to stderr and do not abort the command
- [x] When aggressive mode is off, behaviour of all commands is identical to current behaviour

### Out of scope

- `apm sync` — already manages its own fetch/push logic
- `apm start` — already has aggressive fetch/push support
- `apm show` — already has aggressive fetch support
- `apm new` — already has aggressive push support
- Retry logic on fetch/push failure — warnings are sufficient
- Atomic fetch-then-write (optimistic concurrency) — not in scope; warnings on stale state are sufficient

### Approach

All changes follow the same pattern already used in `show`, `start`, `take` (push half), and `review` (fetch half). No new git functions are needed — `git::fetch_branch`, `git::push_branch`, and `git::fetch_all` already exist in `apm-core/src/git.rs`.

**1. `apm/src/main.rs`** — Add `no_aggressive: bool` (with `#[arg(long)]`) to the `Close`, `Spec`, `Next`, `List`, `Verify`, and `Validate` subcommand structs. `Set` and `Review` already have it but need the push-after change below.

**2. `apm/src/cmd/close.rs`**
- Accept `no_aggressive: bool` parameter
- Resolve ticket branch name
- Before calling `ticket::close`: if aggressive, `git::fetch_branch(root, &branch)` (warn on error)
- After `ticket::close` completes: if aggressive, `git::push_branch(root, &branch)` (warn on error)
- Note: `apm-core/src/ticket.rs::close` already pushes unconditionally — either move the push here and make it conditional, or add a `no_push` flag; prefer moving the push out of `ticket::close` to keep the pattern consistent.

**3. `apm/src/cmd/take.rs`**
- Aggressive flag is already computed but unused for fetch
- Add fetch before the agent-field write: if aggressive, `git::fetch_branch(root, &branch)` before reading the ticket from git

**4. `apm/src/cmd/spec.rs`**
- Accept `no_aggressive: bool`
- Resolve ticket branch name from ticket ID
- Before reading the section: if aggressive, `git::fetch_branch(root, &branch)` (warn on error)
- After committing the section: if aggressive, `git::push_branch(root, &branch)` (warn on error)

**5. `apm/src/cmd/set.rs`**
- Already fetches before read
- After committing field changes: if aggressive, `git::push_branch(root, &branch)` (warn on error)

**6. `apm/src/cmd/review.rs`**
- Already fetches before opening the editor
- After committing amendments (after the editor closes and changes are committed): if aggressive, `git::push_branch(root, &branch)` (warn on error)
- The subsequent `state::transition` call may also push (aggressive already flows there) — that is acceptable; a redundant push is harmless

**7. `apm/src/cmd/next.rs`**
- Accept `no_aggressive: bool`
- Before loading tickets: if aggressive, `git::fetch_all(root)` (warn on error)

**8. `apm/src/cmd/list.rs`**
- Accept `no_aggressive: bool`
- Before loading tickets: if aggressive, `git::fetch_all(root)` (warn on error)

**9. `apm/src/cmd/verify.rs`**
- Accept `no_aggressive: bool`
- Before running checks: if aggressive, `git::fetch_all(root)` (warn on error)

**10. `apm/src/cmd/validate.rs`**
- Accept `no_aggressive: bool`
- Before running checks: if aggressive, `git::fetch_all(root)` (warn on error)

**Order of changes:** Work command-by-command. The pattern is mechanical — no design decisions. Start with the lowest-risk read-only commands (`next`, `list`, `verify`, `validate`), then the write commands (`set`, `review`, `spec`, `take`, `close`).

**Gotcha — `ticket::close` unconditional push:** `apm-core/src/ticket.rs` lines ~316-320 push unconditionally. The push should move to the CLI layer (`cmd/close.rs`) where the aggressive flag is available. This avoids double-push and keeps the pattern consistent. Change `ticket::close` to not push, and handle push in `cmd/close.rs`.

**Tests:** Add integration tests in `apm/tests/integration.rs` that verify: (a) the `--no-aggressive` flag suppresses fetch/push even when config has `aggressive = true`; and (b) the commands do not fail when there is no remote configured (fetch/push warnings are non-fatal).

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T19:50Z | — | new | philippepascal |
| 2026-03-30T20:00Z | new | in_design | philippepascal |
| 2026-03-30T20:04Z | in_design | specd | claude-0330-2005-b7f2 |
| 2026-03-30T20:09Z | specd | ready | apm |
| 2026-03-30T20:10Z | ready | in_progress | philippepascal |
| 2026-03-30T20:30Z | in_progress | implemented | claude-0330-2015-c4d2 |
| 2026-03-30T20:31Z | implemented | accepted | apm-sync |
| 2026-03-31T05:04Z | accepted | closed | apm-sync |