+++
id = "73e484df"
title = "GitHub Actions: release CI for pre-built binaries on 4 platforms"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/73e484df-github-actions-release-ci-for-pre-built-"
created_at = "2026-04-02T20:54:44.627516Z"
updated_at = "2026-04-03T00:18:09.955596Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["48105624"]
+++

## Spec

### Problem

There is no automated release pipeline. Users must build from source via `cargo install`. Pre-built binaries for macOS arm64, macOS x86_64, Linux x86_64, and Linux aarch64 are needed on GitHub Releases to support Homebrew and direct download distribution. See `initial_specs/DESIGN-users.md` point 6.

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
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:18Z | groomed | in_design | philippepascal |
