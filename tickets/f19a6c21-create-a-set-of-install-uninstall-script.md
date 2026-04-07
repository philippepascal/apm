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

- [ ] Running `curl -fsSL https://raw.githubusercontent.com/philippepascal/apm/main/scripts/install.sh | sh` completes without error on macOS aarch64
- [ ] Running the same command completes without error on Linux x86_64
- [ ] After install, `apm --help` runs successfully from a new shell session
- [ ] After install, `apm-server --help` runs successfully from a new shell session
- [ ] The script detects the platform automatically without user input
- [ ] The script fetches the latest release version from the GitHub API when no version is specified
- [ ] The script respects an `APM_VERSION` environment variable to install a specific version
- [ ] The script verifies the SHA256 checksum of the downloaded archive against the published `checksums.txt`
- [ ] The script exits with a non-zero code and a descriptive error message if the checksum does not match
- [ ] The script installs binaries to `~/.local/bin` by default
- [ ] The script respects an `APM_INSTALL_DIR` environment variable to override the install directory
- [ ] The script creates the install directory if it does not already exist
- [ ] The script adds the install directory to `$PATH` in `~/.bashrc`, `~/.zshrc`, and `~/.profile` if it is not already present
- [ ] The script prints a clear success message with the installed version and next steps
- [ ] The script exits with a non-zero code and a descriptive error message when run on an unsupported platform (e.g. Windows, macOS x86_64)
- [ ] The script exits with a non-zero code and a descriptive error message if `curl` or `tar` are not available
- [ ] Running `scripts/uninstall.sh` removes the `apm` binary from the install directory
- [ ] Running `scripts/uninstall.sh` removes the `apm-server` binary from the install directory
- [ ] Running `scripts/uninstall.sh` removes the PATH-export lines added by `install.sh` from `~/.bashrc`, `~/.zshrc`, and `~/.profile`
- [ ] Running `scripts/uninstall.sh` prints a confirmation message listing what was removed
- [ ] Running `scripts/uninstall.sh` when APM is not installed exits cleanly with an informative message (no error)

### Out of scope

- Windows support (no Windows build target exists)
- macOS x86_64 / Intel Mac (no x86_64-apple-darwin build target exists)
- System-wide installation to `/usr/local/bin` or `/usr/bin` (requires sudo; brew covers this use-case)
- Fish shell PATH configuration (bash and zsh only)
- Upgrading an existing APM installation (a future ticket)
- Modifying or replacing the Homebrew formula or tap automation
- Packaging APM as a `.deb`, `.rpm`, or any other system package format
- Installing the `apm-server` separately from the `apm` CLI (both are installed together from the same archive)

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