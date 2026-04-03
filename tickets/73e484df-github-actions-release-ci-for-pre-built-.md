+++
id = "73e484df"
title = "GitHub Actions: release CI for pre-built binaries on 4 platforms"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "6286"
branch = "ticket/73e484df-github-actions-release-ci-for-pre-built-"
created_at = "2026-04-02T20:54:44.627516Z"
updated_at = "2026-04-03T00:18:09.955596Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["48105624"]
+++

## Spec

### Problem

There is no automated release pipeline. Users must build from source via `cargo install`, which requires Rust toolchain and ~10 minutes of compile time. Pre-built binaries for macOS arm64, macOS x86_64, Linux x86_64, and Linux aarch64 are needed on GitHub Releases to support Homebrew and direct download distribution — and to unblock the distribution strategy described in `initial_specs/DESIGN-users.md` point 6.

The project produces two binaries — `apm` (CLI) and `apm-server` (HTTP server). The server currently reads UI static assets from the filesystem at runtime (`apm-ui/dist`). Ticket #48105624 (a dependency of this ticket) changes `apm-server` to embed those assets at compile time via `include_dir!`, making the server binary self-contained. The release CI must build `apm-ui` first so the assets exist when `apm-server` is compiled.

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