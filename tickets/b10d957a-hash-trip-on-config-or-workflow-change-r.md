+++
id = "b10d957a"
title = "Hash-trip on config or workflow change runs apm validate"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b10d957a-hash-trip-on-config-or-workflow-change-r"
created_at = "2026-04-27T20:28:59.343081Z"
updated_at = "2026-04-27T21:24:54.837160Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
depends_on = ["e845127e"]
+++

## Spec

### Problem

When a user modifies `.apm/config.toml` (e.g., switching the completion strategy from `merge` to `pr`) or `.apm/workflow.toml` after tickets with `depends_on` relationships have already been created, existing dependencies can silently become invalid. APM currently has no mechanism to detect this drift: the changed config takes effect immediately, but the tickets that were created under the old rules remain unchanged and unchecked.

The result is that tickets proceed through the workflow carrying stale, invalid dependency configurations. Violations only surface later as confusing failures in branch topology or merge conflicts — not as a clear diagnostic at the moment the configuration changed.

`docs/strategy-and-dependencies.md` (§ 'Hash-trip on config change') specifies the detection mechanism: APM stores a SHA-256 hash of both config files in a local stamp file (`.apm/.validate-stamp`, gitignored). On every `apm` invocation, the live hash is compared to the stored stamp. If they differ, `apm validate` is run automatically in-process. Mutating commands (`apm new`, `apm state`, `apm set`, `apm spec`, `apm start`) are blocked if validation fails; read-only commands (`apm list`, `apm show`, `apm next`) warn but proceed. The stamp is refreshed only after a clean validation pass.

This ticket wires the trigger mechanism. The dependency-rule validation logic itself (`validate_depends_on`, `check_depends_on_rules`) is implemented in ticket e845127e and must land before this ticket is implemented.

### Acceptance criteria

- [ ] When `.apm/config.toml` and `.apm/workflow.toml` are unchanged since the last successful hash-trip, `apm` commands run without invoking validation (no extra output, negligible overhead beyond a hash comparison)
- [ ] When `.apm/.validate-stamp` is absent, the hash-trip runs on the next invocation; if validation passes, the stamp is created and the command proceeds normally
- [ ] When either config file changes and validation passes, the stamp is updated to the new hash and the command proceeds normally with no user-visible output
- [ ] When either config file changes and validation fails, `apm new`, `apm state`, `apm set`, `apm spec`, and `apm start` exit with a non-zero code and a message explaining that mutating commands are blocked until validation passes
- [ ] When validation fails after a config change, `apm list`, `apm show`, and `apm next` run normally but print a warning to stderr that the config has changed and validation is failing
- [ ] `apm validate` is never blocked by the hash-trip gate (it must always be runnable so users can diagnose and fix issues)
- [ ] `apm init` is never blocked by the hash-trip gate (it runs before or during initial config creation)
- [ ] When `apm validate` completes with no issues, it updates the stamp file, clearing any stale hash-trip block on subsequent commands
- [ ] `.apm/.validate-stamp` does not appear in `git status` output (it is gitignored via `.apm/.gitignore`)
- [ ] When `.apm/config.toml` does not exist (not an APM repo), the hash-trip logic is skipped entirely and no stamp file is written

### Out of scope

- Auto-fixing dependency violations — no safe automatic correction exists; requires user intervention
- Enforcing dependency rules at `apm new` or `apm set` write time — ticket a3dc64db
- Implementing `validate_depends_on` and `check_depends_on_rules` — ticket e845127e; this ticket only wires the trigger
- Hash-tripping on changes to ticket files, `agents.md`, or any file other than `config.toml` and `workflow.toml`
- Network-based or CI-triggered re-validation
- A dedicated `apm stamp reset` or `apm stamp clear` command
- Sharing the stamp file across machines or storing it in git — the stamp is intentionally machine-local and gitignored
- Blocking `apm workers`, `apm sessions`, `apm revoke`, `apm version`, `apm register`, `apm show`, `apm list`, `apm next`, or other read-only / administrative commands (they warn but are not blocked)
- Changing the default completion strategy (ticket 941e57fa) or removing the per-epic max_workers override (ticket 6e3f9e91)

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:44Z | new | groomed | philippepascal |
| 2026-04-27T21:24Z | groomed | in_design | philippepascal |