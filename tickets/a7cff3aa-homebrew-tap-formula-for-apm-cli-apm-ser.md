+++
id = "a7cff3aa"
title = "Homebrew tap formula for apm CLI + apm-server"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "25124"
branch = "ticket/a7cff3aa-homebrew-tap-formula-for-apm-cli-apm-ser"
created_at = "2026-04-02T20:54:55.761604Z"
updated_at = "2026-04-03T23:20:44.923085Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["73e484df"]
+++

## Spec

### Problem

There is no Homebrew tap for apm. Users on macOS must either install the Rust toolchain and run `cargo install` (~10 minutes compile time) or manually download binaries from GitHub Releases and place them on PATH. Neither path is acceptable for a tool that targets single developers who want a quick setup.

A Homebrew tap (`philippepascal/tap`) with a formula pointing at the pre-built GitHub Release archives is the standard macOS distribution path. Once the release CI (ticket #73e484df) publishes `apm-<tag>-<target-triple>.tar.gz` archives, a formula can install both `apm` and `apm-server` with a single `brew install` command. See `initial_specs/DESIGN-users.md` point 6.

This ticket creates the tap repository and the formula. It does not automate formula updates on new releases — that is a follow-up concern.

### Acceptance criteria

- [ ] A public GitHub repository `philippepascal/homebrew-tap` exists
- [ ] The repository contains a formula file `Formula/apm.rb`
- [ ] `brew tap philippepascal/tap` succeeds without errors
- [ ] `brew install philippepascal/tap/apm` installs both `apm` and `apm-server` binaries to the Homebrew prefix
- [ ] After installation, `apm --help` runs successfully
- [ ] After installation, `apm-server --help` runs successfully
- [ ] The formula downloads the correct archive for the host architecture (arm64 on Apple Silicon, x86_64 on Intel)
- [ ] The formula verifies the SHA-256 checksum of the downloaded archive
- [ ] `brew uninstall apm` cleanly removes both binaries
- [ ] The formula includes a `test` block that verifies both binaries execute (e.g. `apm --help` and `apm-server --help`)

### Out of scope

- Automated formula version bumps when a new release is tagged (follow-up ticket)
- Linux Homebrew (Linuxbrew) support — Linux users download binaries directly or use cargo install
- Building from source via Homebrew (`brew install --build-from-source`) — formula uses pre-built bottles only
- The release CI itself (ticket #73e484df)
- Publishing to Homebrew core (requires significant adoption; tap is the right path for now)
- Windows package managers (scoop, chocolatey, winget)
- Formula for apm-proxy Docker image

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
| 2026-04-03T00:26Z | groomed | in_design | philippepascal |
| 2026-04-03T22:47Z | in_design | ready | apm |
| 2026-04-03T22:50Z | ready | ammend | apm |
| 2026-04-03T23:20Z | ammend | in_design | philippepascal |