+++
id = "f19a6c21"
title = "create a set of install/uninstall scripts for apm on all platforms supported, including brew"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/f19a6c21-create-a-set-of-install-uninstall-script"
created_at = "2026-04-07T17:07:48.816446Z"
updated_at = "2026-04-07T17:44:01.761396Z"
+++

## Spec

### Problem

APM currently ships binaries via a Homebrew tap (`philippepascal/homebrew-tap`), but Homebrew is not available on all systems and is not the preferred installation path for users on Linux or for those who want a quick one-liner from the APM home page. There is no general-purpose install/uninstall script today.

The desired end-state is a pair of shell scripts — `scripts/install.sh` and `scripts/uninstall.sh` — that let any user on a supported platform install APM with a single `curl | sh` command, without needing a package manager. The scripts handle platform detection, binary download from the GitHub release, checksum verification, placement on `$PATH`, and clean removal.

Supported platforms match what the release workflow already builds: `aarch64-apple-darwin` (macOS Apple Silicon) and `x86_64-unknown-linux-musl` (Linux x86_64). The scripts complement Homebrew — they are an alternative path, not a replacement for it.

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
| 2026-04-07T17:07Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:44Z | groomed | in_design | philippepascal |