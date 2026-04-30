+++
id = "3048d7e9"
title = "Migration: validate --fix ports legacy command/args/model to agent + options"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3048d7e9-migration-validate-fix-ports-legacy-comm"
created_at = "2026-04-30T20:03:17.277300Z"
updated_at = "2026-04-30T21:36:57.594854Z"
epic = "4312fbd4"
target_branch = "epic/4312fbd4-agent-wrapper-architecture"
depends_on = ["6cac8518"]
+++

## Spec

### Problem

Existing APM projects have a `.apm/config.toml` using the legacy `[workers]` shape: `command = "claude"`, `args = ["--print", ...]`, and `model = "sonnet"`. After upgrading to the agent-wrapper architecture (ticket 6cac8518), those projects receive a deprecation warning on every `apm start` invocation but have no automated way to migrate.

The desired state is `agent = "claude"` in `[workers]` with model moved to `[workers.options]` and `args` dropped entirely (the wrapper now owns CLI flag construction). A matching migration must apply to every `[worker_profiles.<X>]` section as well.

This ticket adds that migration to `apm validate --fix`. A developer who upgrades APM runs `apm validate --fix`, sees a one-line confirmation message, and their config is correct without any manual editing. If the project was using a non-Claude command, automated migration is not safe — the tool warns and stops so the user can hand-pick a wrapper.

### Acceptance criteria

- [ ] `apm validate --fix` on a config with `[workers] command = "claude"` rewrites it to `[workers] agent = "claude"` and removes the `command` key
- [ ] `apm validate --fix` on a config with `[workers] model = "sonnet"` moves the value to `[workers.options] model = "sonnet"` and removes the top-level `model` key
- [ ] `apm validate --fix` on a config with `[workers] args = [...]` removes the `args` key regardless of its contents
- [ ] `apm validate --fix` on a config with `[worker_profiles.X] model = "opus"` moves the value to `[worker_profiles.X.options] model = "opus"` and removes the profile-level `model` key
- [ ] `apm validate --fix` on a config with `[worker_profiles.X] command = "claude"` removes the profile-level `command` key (profile inherits `agent` from global)
- [ ] `apm validate --fix` on a config with `[worker_profiles.X] args = [...]` removes the profile-level `args` key
- [ ] `apm validate --fix` on a config where `[workers] command` is anything other than `"claude"` prints a warning naming the offending command and does not modify the config file
- [ ] `apm validate --fix` on a config where any `[worker_profiles.X] command` is anything other than `"claude"` prints a warning naming the profile and command, and does not modify the config file
- [ ] After a successful migration `apm validate` (without `--fix`) exits zero on the rewritten config
- [ ] `apm validate --fix` on a config that has no legacy fields (`agent` already set, no `command`/`args`/`model`) makes no changes to the file
- [ ] `apm validate --fix` on a config with both legacy fields and new fields (e.g. `agent` already present alongside a leftover `model`) removes the legacy fields and leaves the new fields intact
- [ ] A successful migration prints exactly the line: `migrated [workers] config to agent-driven shape; legacy command/args/model removed`
- [ ] TOML comments present in the config file survive the migration unchanged
- [ ] Key ordering of unrelated sections (e.g. `[keychain]`, `[env]`) is preserved after migration

### Out of scope

- An `apm init --migrate` or `apm agents migrate` subcommand — `apm validate --fix` is the canonical migration path
- Migration of `.apm/agents.md`, `.apm/apm.worker.md`, or `.apm/apm.spec-writer.md` — those are prompt content, not config
- Hash-trip integration changes — the existing hash-trip already runs validate on config change; no adjustment needed here
- Removing deprecated `command`/`args`/`model` fields from the Rust structs — that happens after the deprecation window, tracked separately (ticket 6cac8518 retains the fields for backward compatibility)
- `apm validate --fix` for `workflow.toml` files — only `.apm/config.toml` (and `apm.toml` legacy root path) contain worker config
- Rollback or backup of the original config — the caller can use version control
- Migrating a config where `command` is non-Claude — this ticket explicitly stops and warns rather than guessing a wrapper name
- Windows execute-bit semantics or platform-specific config path differences

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-30T20:03Z | — | new | philippepascal |
| 2026-04-30T21:02Z | new | groomed | philippepascal |
| 2026-04-30T21:36Z | groomed | in_design | philippepascal |