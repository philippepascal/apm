+++
id = 47
title = "Bug: apm start does not fetch main before merging in aggressive mode"
state = "closed"
priority = 4
effort = 2
risk = 2
author = "claude-0328-c72b"
agent = "claude-0329-main"
branch = "ticket/0047-bug-apm-start-does-not-fetch-main-before"
created_at = "2026-03-28T19:50:04.169100Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

`apm start` merges `origin/<default_branch>` into the ticket branch so the agent
starts from current code (lines 82–112 of `start.rs`). However, it checks whether
`origin/main` exists locally and uses it directly — it never fetches from the
remote first. When aggressive mode is on, `apm start` already fetches the ticket
branch (lines 60–63), but not `main`. If the local `origin/main` ref is stale, the
merge brings in an outdated base even in aggressive mode.

### Acceptance criteria

- [x] When `sync.aggressive = true`, `apm start` fetches `origin/<default_branch>`
  before merging it into the ticket branch
- [x] When `sync.aggressive = false` (or `--no-aggressive` is passed), the fetch
  is skipped (existing behaviour — no regression)
- [x] If the fetch fails, a warning is printed and the merge proceeds with the
  locally-cached ref (same fail-soft pattern used elsewhere)
- [x] The fetch of the ticket branch and the fetch of the default branch are both
  present in aggressive mode (neither replaces the other)

### Out of scope

- Fetching all branches (only the default branch and the ticket branch are needed)
- Changing merge strategy or conflict handling

### Approach

In `apm/src/cmd/start.rs`, move the `default_branch` binding before the
`if aggressive` block, then add a second fetch inside it:

```rust
let default_branch = &config.project.default_branch;  // move up

if aggressive {
    if let Err(e) = git::fetch_branch(root, &branch) {
        eprintln!("warning: fetch failed: {e:#}");
    }
    if let Err(e) = git::fetch_branch(root, default_branch) {
        eprintln!("warning: fetch {} failed: {e:#}", default_branch);
    }
}
```

`default_branch` is currently declared a few lines later (line 82) — moving it
earlier makes it available for both the fetch and the existing merge logic with
no other changes needed.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T19:50Z | — | new | claude-0328-c72b |
| 2026-03-28T19:54Z | new | specd | claude-0328-c72b |
| 2026-03-29T19:08Z | specd | ready | claude-0329-1200-a1b2 |
| 2026-03-29T20:28Z | ready | in_progress | claude-0329-main |
| 2026-03-29T20:35Z | in_progress | implemented | claude-0329-main |
| 2026-03-29T20:46Z | implemented | accepted | claude-0329-main |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |