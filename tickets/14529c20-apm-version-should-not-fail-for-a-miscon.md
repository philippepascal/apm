+++
id = "14529c20"
title = "apm version should not fail for a misconfiguration. it should not rely on any config"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/14529c20-apm-version-should-not-fail-for-a-miscon"
created_at = "2026-08-21T19:46:23.139006Z"
updated_at = "2026-08-21T19:52:53.854148Z"
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
| 2026-08-21T19:46Z | — | new | philippepascal |
| 2026-08-21T19:49Z | new | groomed | philippepascal |
| 2026-08-21T19:49Z | groomed | in_design | philippepascal |
| 2026-08-21T19:50Z | in_design | groomed | philippepascal |
| 2026-08-21T19:52Z | groomed | in_design | philippepascal |