+++
id = 36
title = "push-pull-agressive-mode"
state = "closed"
priority = 0
effort = 4
risk = 2
author = "apm"
agent = "claude-0327-2000-36ff"
branch = "ticket/0036-push-pull-agressive-mode"
created_at = "2026-03-28T00:45:49.063412Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

In the default mode, APM commands (`apm show`, `apm state`, `apm start`) operate on local branch data and do not fetch from remote first. In a team setting with multiple agents and humans pushing to ticket branches, local state can fall behind, causing stale reads and avoidable conflicts. An opt-in "aggressive" mode would fetch before reading and push after writing, trading latency for freshness.

### Acceptance criteria

- [ ] `apm.toml` supports `[sync] aggressive = true/false` (default `false`)
- [ ] `apm show <id>` fetches the ticket branch before reading when aggressive mode is on
- [ ] `apm state <id> <state>` pushes the ticket branch after committing when aggressive mode is on
- [ ] `apm start <id>` fetches and merges the latest remote branch before checking out when aggressive mode is on
- [ ] `apm sync` always pushes all ticket branches when aggressive mode is on (in addition to fetching)
- [ ] All commands that read/write ticket branches accept a `--no-aggressive` flag to override aggressive mode for that invocation
- [ ] When a fetch or push fails in aggressive mode, the command prints a warning and continues (non-fatal)

### Out of scope

- Real-time sync or webhook-based triggers
- Conflict resolution strategy (push conflicts still require manual resolution)
- Changing the behavior of non-aggressive (default) mode
- Per-command aggressive toggles in `apm.toml` (one global flag is enough)

### Approach

Add `AggressiveSync: bool` to a `[sync]` section in `apm-core/src/config.rs`. Thread the config through to the relevant command handlers. In each affected command, wrap the existing logic with a pre-step (`git fetch origin <branch>`) and/or post-step (`git push origin <branch>`) conditional on `config.sync.aggressive`. The `--no-aggressive` flag is added as a shared boolean on affected subcommands in `main.rs`. Failures in fetch/push emit `eprintln!` warnings and do not propagate as errors.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-28T00:45Z | — | new | apm |
| 2026-03-28T01:04Z | new | specd | claude-0327-1757-391b |
| 2026-03-28T01:07Z | specd | ready | apm |
| 2026-03-28T02:09Z | ready | in_progress | claude-0327-2000-36ff |
| 2026-03-28T02:12Z | in_progress | implemented | claude-0327-2000-36ff |
| 2026-03-28T07:31Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |