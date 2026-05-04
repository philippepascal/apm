+++
id = "e04e1b3f"
title = "Revise apm-demo creation script for mock worker support"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/e04e1b3f-revise-apm-demo-creation-script-for-mock"
created_at = "2026-05-04T16:48:32.146018Z"
updated_at = "2026-05-04T16:50:43.622215Z"
epic = "65af2998"
target_branch = "epic/65af2998-apm-demo-enhancements"
+++

## Spec

### Problem

The `create-demo.sh` script always writes `command = "claude"` and `args = ["--print"]` into the `[workers]` block of the generated `.apm/config.toml`. Anyone who runs `apm work` against the resulting demo must have a live Claude CLI session, which is a barrier for documentation, CI, and onboarding.

The sibling ticket 295ff9ba ("Add mock_happy demo script for GIF recording") depends on this ticket because it needs a way to create a demo repo that uses the `mock-happy` built-in wrapper instead. `mock-happy` processes tickets deterministically and instantly without Claude — ideal for recording a repeatable GIF of the APM workflow. The creation script must be extended to support this use case while leaving the existing Claude-based default intact.

### Acceptance criteria

- [ ] `create-demo.sh --mock` produces a demo repo whose `.apm/config.toml` `[workers]` block contains `command = "mock-happy"` with no `args` field
- [ ] `create-demo.sh` with no flags produces a demo repo whose `[workers]` block contains `command = "claude"` and `args = ["--print"]` (existing behaviour unchanged)
- [ ] `create-demo.sh --mock` runs to completion without error on a clean GitHub account that has `gh`, `apm`, and internet access
- [ ] Passing an unrecognised flag to `create-demo.sh` prints an error message and exits non-zero

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
| 2026-05-04T16:48Z | — | new | philippepascal |
| 2026-05-04T16:50Z | new | groomed | philippepascal |
| 2026-05-04T16:50Z | groomed | in_design | philippepascal |