+++
id = "eb4ee6d7"
title = "aggressive sync should default to true"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
agent = "85958"
branch = "ticket/eb4ee6d7-aggressive-sync-should-default-to-true"
created_at = "2026-03-30T19:53:26.019513Z"
updated_at = "2026-03-31T05:05:26.290356Z"
+++

## Spec

### Problem

The `sync.aggressive` flag controls whether commands fetch before reading and push after writing. It defaults to `false` because `bool` fields in Rust/serde default to false when absent.

This is the wrong default. Aggressive mode is the safe, correct behaviour for any team or single-user workflow where GitHub is the source of truth. Without it, commands silently operate on stale local state — a footgun that only manifests when things go wrong (merge conflicts, double-transitions, stale PR detection).

The fix is a one-liner: add `#[serde(default = "default_true")]` to the `aggressive` field in `SyncConfig` so that new repos and repos without an explicit `aggressive` line in `config.toml` get `true` automatically. Existing repos with `aggressive = false` explicitly set are unaffected.

### Acceptance criteria

- [x] When `apm.toml` has no `[sync]` section, `sync.aggressive` resolves to `true`
- [x] When `apm.toml` has `[sync]` with no `aggressive` key, `sync.aggressive` resolves to `true`
- [x] When `apm.toml` has `aggressive = false` explicitly, `sync.aggressive` resolves to `false`
- [x] When `apm.toml` has `aggressive = true` explicitly, `sync.aggressive` resolves to `true`

### Out of scope

- Changes to how the `--no-aggressive` CLI flag works on any command
- Adding new tests beyond what is needed to verify the serde default behaviour
- Documentation updates

### Approach

In `apm-core/src/config.rs`, change the `SyncConfig` struct from:

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SyncConfig {
    #[serde(default)]
    pub aggressive: bool,
}
```

to:

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SyncConfig {
    #[serde(default = "default_true")]
    pub aggressive: bool,
}
```

The `default_true` helper function already exists in the same file (used by `AgentsConfig::side_tickets`), so no new function is needed.

Add a unit test in `apm-core/src/config.rs` (or its existing test module) covering all four acceptance criteria: no section, section without key, explicit `false`, and explicit `true`.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T19:53Z | — | new | philippepascal |
| 2026-03-30T20:00Z | new | in_design | philippepascal |
| 2026-03-30T20:02Z | in_design | specd | claude-0330-2005-b7c2 |
| 2026-03-30T20:10Z | specd | ready | apm |
| 2026-03-30T20:10Z | ready | in_progress | philippepascal |
| 2026-03-30T20:12Z | in_progress | implemented | claude-0330-2015-f4a1 |
| 2026-03-30T20:31Z | implemented | accepted | apm-sync |
| 2026-03-31T05:05Z | accepted | closed | apm-sync |