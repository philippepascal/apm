+++
id = 17
title = "apm sync missing --quiet and --offline flags"
state = "closed"
priority = 10
effort = 1
risk = 1
branch = "ticket/0017-apm-sync-missing-quiet-and-offline-flags"
updated_at = "2026-03-27T00:06:01.145860Z"
+++

## Spec

### Problem

The `post-merge` git hook installed by `apm init` calls `apm sync --quiet --offline`.
Neither `--quiet` nor `--offline` are wired as CLI arguments in `main.rs` or
`cmd/sync.rs`. Every merge fires the hook and immediately fails with "unexpected
argument", breaking the auto-sync on merge.

### Acceptance criteria

- [ ] `apm sync --offline` skips `git fetch --all`; re-processes local branches only
- [ ] `apm sync --quiet` suppresses all non-error output
- [ ] Both flags can be combined: `apm sync --quiet --offline`
- [ ] Without flags, behavior is unchanged

### Out of scope

- Actually firing merge auto-transitions (tracked in #4)

### Approach

Add `--offline: bool` and `--quiet: bool` to the `Sync` variant in `main.rs`.
Pass both into `cmd::sync::run`. In `sync.rs`, skip `fetch_all` when `offline`,
and gate all `println!` calls behind `!quiet`.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-26 | manual | new → specd | |
| 2026-03-26 | manual | specd → ready | |
| 2026-03-27T00:06Z | ready | closed | apm |