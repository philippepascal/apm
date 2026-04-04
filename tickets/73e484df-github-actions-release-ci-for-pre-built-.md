+++
id = "73e484df"
title = "GitHub Actions: release CI for pre-built binaries on 4 platforms"
state = "closed"
priority = 0
effort = 3
risk = 3
author = "apm"
branch = "ticket/73e484df-github-actions-release-ci-for-pre-built-"
created_at = "2026-04-02T20:54:44.627516Z"
updated_at = "2026-04-04T06:01:38.977494Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["48105624"]
+++

## Spec

### Problem

There is no automated release pipeline. Users must build from source via `cargo install`, which requires Rust toolchain and ~10 minutes of compile time. Pre-built binaries for macOS arm64, macOS x86_64, Linux x86_64, and Linux aarch64 are needed on GitHub Releases to support Homebrew and direct download distribution — and to unblock the distribution strategy described in `initial_specs/DESIGN-users.md` point 6.

The project produces two binaries — `apm` (CLI) and `apm-server` (HTTP server). The server currently reads UI static assets from the filesystem at runtime (`apm-ui/dist`). Ticket #48105624 (a dependency of this ticket) changes `apm-server` to embed those assets at compile time via `include_dir!`, making the server binary self-contained. The release CI must build `apm-ui` first so the assets exist when `apm-server` is compiled.

### Acceptance criteria

- [x] A workflow file `.github/workflows/release.yml` exists and triggers on pushes of tags matching `v*.*.*`
- [x] The workflow can also be triggered manually via `workflow_dispatch`
- [x] `cargo test --workspace` runs as a gate before any build step; the workflow fails and no release is created if tests fail
- [x] `apm-ui` assets are built (`npm ci && npm run build` inside `apm-ui/`) before `apm-server` is compiled on every platform
- [x] The workflow produces an `apm` binary for macOS arm64 (native `macos-14` runner)
- [x] The workflow produces an `apm-server` binary for macOS arm64
- [x] The workflow produces an `apm` binary for macOS x86_64 (native `macos-13` runner)
- [x] The workflow produces an `apm-server` binary for macOS x86_64
- [x] The workflow produces an `apm` binary for Linux x86_64, statically linked (musl target on `ubuntu-22.04`)
- [x] The workflow produces an `apm-server` binary for Linux x86_64, statically linked
- [x] The workflow produces an `apm` binary for Linux aarch64, statically linked (cross-compiled via `cargo-zigbuild` on `ubuntu-22.04`)
- [x] The workflow produces an `apm-server` binary for Linux aarch64, statically linked
- [x] Each platform's binaries are packaged into a `.tar.gz` archive named `apm-<tag>-<target-triple>.tar.gz` containing both `apm` and `apm-server`
- [x] A `checksums.txt` file containing SHA-256 hashes for all four archives is included in the release
- [x] A GitHub Release is created automatically for the tag with all four archives and `checksums.txt` as release assets

### Out of scope

- Homebrew formula generation or tap repository updates
- Docker image builds for apm-proxy
- Windows binaries
- macOS code signing or notarization (no Developer ID required for initial release)
- Publishing to crates.io or any package registry
- Automatic changelog generation from commit history
- Pre-release or nightly builds on non-tag pushes
- Static asset embedding in apm-server (covered by ticket #48105624, a declared dependency)
- CI for pull requests or pushes to branches (this workflow is release-only)

### Approach

Create a single file: `.github/workflows/release.yml`

**Triggers**

```yaml
on:
  push:
    tags: ['v*.*.*']
  workflow_dispatch:
```

**Job 1: `test` (ubuntu-22.04)**
- Checkout
- `cargo test --workspace`

All subsequent jobs declare `needs: test`.

**Job 2: `build` (matrix)**

Matrix of 4 entries — each entry is a distinct runner/target combination:

| os | target | notes |
|----|--------|-------|
| macos-14 | aarch64-apple-darwin | native Apple Silicon |
| macos-13 | x86_64-apple-darwin | native Intel |
| ubuntu-22.04 | x86_64-unknown-linux-musl | native + musl-tools |
| ubuntu-22.04 | aarch64-unknown-linux-musl | cross via cargo-zigbuild |

Steps per matrix entry:
1. `actions/checkout@v4`
2. `actions/setup-node@v4` (Node 20), then `npm ci && npm run build` inside `apm-ui/`
3. `rustup target add <matrix.target>`
4. Linux musl only: `sudo apt-get install -y musl-tools`
5. Linux aarch64 only: install Zig via `mlugg/setup-zig@v1` action and `cargo install cargo-zigbuild`
6. Build:
   - macOS and Linux x86_64: `cargo build --release --target <matrix.target> -p apm -p apm-server`
   - Linux aarch64: `cargo zigbuild --release --target <matrix.target> -p apm -p apm-server`
7. Strip binaries with `strip` to reduce size (~50%)
8. Package: `tar -czf apm-<tag>-<matrix.target>.tar.gz -C target/<matrix.target>/release apm apm-server`
9. Upload archive as a workflow artifact via `actions/upload-artifact@v4`

**Job 3: `release` (ubuntu-22.04)**

`needs: build` — runs only after all 4 build matrix jobs succeed.

Steps:
1. Download all 4 artifacts via `actions/download-artifact@v4`
2. Generate checksums: `sha256sum *.tar.gz > checksums.txt`
3. Create GitHub Release via `softprops/action-gh-release@v2` with all four `.tar.gz` files and `checksums.txt` as assets. Requires `permissions: contents: write` on the job.

**Caching**
Add `actions/cache@v4` for `~/.cargo/registry` and `~/.cargo/git` keyed on `Cargo.lock` to speed up repeat runs.

**Gotchas**
- `cargo-zigbuild` for Linux aarch64 musl requires Zig on PATH. Use the `mlugg/setup-zig@v1` action rather than `pip3 install ziglang` to avoid PATH issues in GitHub Actions.
- The `apm-ui/dist` directory must exist before `cargo build -p apm-server` runs, because ticket #48105624 makes the build fail at compile time if `dist` is absent. The Node build step must come before the cargo build step on every matrix runner.
- Release job needs `GITHUB_TOKEN` (available automatically) and `permissions: contents: write` to create releases.
- Archive filename should use the git tag name (e.g. `v0.1.0`) so filenames are deterministic and match what Homebrew formulas will reference later.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:18Z | groomed | in_design | philippepascal |
| 2026-04-03T00:22Z | in_design | specd | claude-0402-2018-spec1 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T02:56Z | ready | in_progress | philippepascal |
| 2026-04-04T02:59Z | in_progress | implemented | claude-0403-0300-b7f2 |
| 2026-04-04T06:01Z | implemented | closed | apm-sync |
