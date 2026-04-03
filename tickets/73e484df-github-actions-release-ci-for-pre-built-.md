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

- [ ] A workflow file `.github/workflows/release.yml` exists and triggers on pushes of tags matching `v*.*.*`
- [ ] The workflow can also be triggered manually via `workflow_dispatch`
- [ ] `cargo test --workspace` runs as a gate before any build step; the workflow fails and no release is created if tests fail
- [ ] `apm-ui` assets are built (`npm ci && npm run build` inside `apm-ui/`) before `apm-server` is compiled on every platform
- [ ] The workflow produces an `apm` binary for macOS arm64 (native `macos-14` runner)
- [ ] The workflow produces an `apm-server` binary for macOS arm64
- [ ] The workflow produces an `apm` binary for macOS x86_64 (native `macos-13` runner)
- [ ] The workflow produces an `apm-server` binary for macOS x86_64
- [ ] The workflow produces an `apm` binary for Linux x86_64, statically linked (musl target on `ubuntu-22.04`)
- [ ] The workflow produces an `apm-server` binary for Linux x86_64, statically linked
- [ ] The workflow produces an `apm` binary for Linux aarch64, statically linked (cross-compiled via `cargo-zigbuild` on `ubuntu-22.04`)
- [ ] The workflow produces an `apm-server` binary for Linux aarch64, statically linked
- [ ] Each platform's binaries are packaged into a `.tar.gz` archive named `apm-<tag>-<target-triple>.tar.gz` containing both `apm` and `apm-server`
- [ ] A `checksums.txt` file containing SHA-256 hashes for all four archives is included in the release
- [ ] A GitHub Release is created automatically for the tag with all four archives and `checksums.txt` as release assets

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