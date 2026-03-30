+++
id = "eb4ee6d7"
title = "aggressive sync should default to true"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/eb4ee6d7-aggressive-sync-should-default-to-true"
created_at = "2026-03-30T19:53:26.019513Z"
updated_at = "2026-03-30T19:53:26.019513Z"
+++

## Spec

### Problem

The `sync.aggressive` flag controls whether commands fetch before reading and push after writing. It defaults to `false` because `bool` fields in Rust/serde default to false when absent.

This is the wrong default. Aggressive mode is the safe, correct behaviour for any team or single-user workflow where GitHub is the source of truth. Without it, commands silently operate on stale local state — a footgun that only manifests when things go wrong (merge conflicts, double-transitions, stale PR detection).

The fix is a one-liner: add `#[serde(default = "default_true")]` to the `aggressive` field in `SyncConfig` so that new repos and repos without an explicit `aggressive` line in `config.toml` get `true` automatically. Existing repos with `aggressive = false` explicitly set are unaffected.

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
| 2026-03-30T19:53Z | — | new | philippepascal |