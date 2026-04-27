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
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:44Z | new | groomed | philippepascal |
| 2026-04-27T21:24Z | groomed | in_design | philippepascal |