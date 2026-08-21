+++
id = "14529c20"
title = "apm version should not fail for a misconfiguration. it should not rely on any config"
state = "specd"
priority = 3
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/14529c20-apm-version-should-not-fail-for-a-miscon"
created_at = "2026-08-21T19:46:23.139006Z"
updated_at = "2026-08-21T19:55:02.408372Z"
+++

## Spec

### Problem

`apm version` fails with a hard error when the project's `.apm/config.toml` is
missing a required field or otherwise fails to parse:

```
apm version
Error: cannot parse /Users/philippepascal/repos/ticker/.apm/config.toml: TOML parse error at line 19, column 1
   |
19 | [workers]
   | ^^^^^^^^^
missing field `default`
```

`apm version` only needs to print the compiled version string
(`env!("CARGO_PKG_VERSION")` / `env!("APM_GIT_DESCRIBE")`) — it has no
legitimate dependency on `.apm/config.toml` at all. The failure comes from
`main()`'s pre-dispatch pipeline in `apm/src/main.rs`: every command not
listed in `hash_trip::is_exempt_command` goes through `hash_trip::run(&root)`,
which — when the config hash has drifted from the stored stamp (including on
a repo's first invocation, when no stamp exists yet) — calls
`apm_core::config::Config::load(root)?` and propagates any parse error with
`?` all the way up through `main()`, aborting the process before `Command::Version`
is ever dispatched. `Command::Version` is not currently in the
`is_exempt_command` list (unlike `Validate`, `Init`, `Help`, `PathGuard`, and
`Instructions`, which already skip this check).

This breaks a basic diagnostic workflow: a user or agent hitting a config
error can't even run `apm version` to sanity-check which build they're on
without first fixing the config that they may be trying to diagnose.

### Acceptance criteria

- [ ] `apm version` exits 0 and prints the version line when `.apm/config.toml` contains invalid TOML (e.g. a missing required field)
- [ ] `apm version` exits 0 and prints the version line when `.apm/config.toml` is absent entirely
- [ ] `apm version` exits 0 and prints the version line when `.apm/config.toml` is valid (no regression)
- [ ] `apm version`'s printed output is unchanged in format/content from before this fix when config is valid
- [ ] Running `apm version` against a repo with a broken `.apm/config.toml` does not write or update the hash-trip stamp file
- [ ] Other commands' handling of a broken `.apm/config.toml` (e.g. `apm list` still erroring, `apm validate` still exempt and reporting the issue) is unchanged

### Out of scope

- Making `apm version` work outside a git repository — it still calls `repo_root()` and requires being inside a git repo, since that failure is unrelated to config parsing and not what this ticket reports
- Exempting any command other than `Version` from the hash-trip/config-load check (e.g. `Help`, `Instructions`, etc. are already exempt; no other command is added here)
- Changing `hash_trip::run`'s behavior for commands that legitimately need config (e.g. `apm list`, `apm next`) — those should continue to surface config errors
- Changing the silent `if let Ok(ref config) = Config::load(&root)` logging-setup block earlier in `main()` — it already tolerates a broken config and is not the source of this bug

### Approach

#### Fix

In `apm/src/hash_trip.rs`, add `super::Command::Version` to the `matches!` arm
inside `is_exempt_command` (alongside `Validate`, `Init`, `Help`, `PathGuard`,
and `Instructions`):

```rust
pub fn is_exempt_command(cmd: &super::Command) -> bool {
    matches!(
        cmd,
        super::Command::Validate { .. }
            | super::Command::Init { .. }
            | super::Command::Help { .. }
            | super::Command::PathGuard
            | super::Command::Instructions { .. }
            | super::Command::Version
    )
}
```

This is a one-line change. With `Version` exempt, `main()` in
`apm/src/main.rs` skips the `hash_trip::run(&root)?` call entirely for `apm
version`, so `Config::load` is never invoked on that path and a broken
`.apm/config.toml` can no longer abort the command. `Command::Version` is a
unit variant (`Command::Version` with no fields), so the match arm is a bare
`super::Command::Version`, not `super::Command::Version { .. }`.

Nothing else in `apm version`'s path touches config: `cmd::version::run()`
(`apm/src/cmd/version.rs`) only prints `CARGO_PKG_VERSION` and
`APM_GIT_DESCRIBE`, and `main()`'s earlier logging-setup block
(`if let Ok(ref config) = apm_core::config::Config::load(&root) { ... }`)
already discards `Config::load` errors via `if let Ok(...)`, so it does not
need to change.

`repo_root()` is still called unconditionally at the top of `main()` before
command dispatch, so `apm version` still requires being run inside a git
repository — that is unrelated to this ticket's config-parsing failure and is
left as-is (see Out of scope).

#### Tests

Add a `hash_trip.rs` unit test mirroring the existing `validate_is_exempt` /
`init_is_exempt` tests: construct `Command::Version` and assert
`is_exempt_command` returns `true`.

Add an `apm/tests/e2e.rs` test (pattern like `setup_resolve_repo` /
`write_valid_spec_for_test`, which already write `.apm/config.toml` directly
via `std::fs::write` into a temp repo): initialize a temp repo with `apm
init`, then overwrite `.apm/config.toml` with TOML missing a required field
(e.g. a `[workers]` table with no `default` key, matching the ticket's
reported error), run `apm version`, and assert the process exits 0 and stdout
contains the version string. Add a second case with `.apm/config.toml`
deleted entirely, asserting the same. Also assert the hash-stamp file (see
`apm_core::hash_stamp::read_stamp`) is unchanged/absent after running `apm
version` against the broken config, to cover the stamp-not-written AC.

Run `cargo test --workspace` before marking implemented.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-08-21T19:46Z | — | new | philippepascal |
| 2026-08-21T19:49Z | new | groomed | philippepascal |
| 2026-08-21T19:49Z | groomed | in_design | philippepascal |
| 2026-08-21T19:50Z | in_design | groomed | philippepascal |
| 2026-08-21T19:52Z | groomed | in_design | philippepascal |
| 2026-08-21T19:55Z | in_design | specd | claude |
