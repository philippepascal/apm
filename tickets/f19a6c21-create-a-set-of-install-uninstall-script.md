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

### Files to create

- `scripts/install.sh` — POSIX-compatible shell installer
- `scripts/uninstall.sh` — POSIX-compatible shell remover

Both scripts must be POSIX `sh` (not bash-specific) so they work on the widest range of systems, including minimal Linux containers where only `/bin/sh` is guaranteed.

---

### `scripts/install.sh` — step by step

**1. Detect platform**

```sh
OS=$(uname -s)
ARCH=$(uname -m)
```

Map to release target triple:

| `uname -s` | `uname -m` | Target triple |
|------------|------------|---------------|
| Darwin | arm64 | aarch64-apple-darwin |
| Linux | x86_64 | x86_64-unknown-linux-musl |
| anything else | — | error + exit 1 |

**2. Check prerequisites**

Exit with a clear error if `curl` or `tar` is not on `$PATH`.

**3. Resolve version**

If `APM_VERSION` is set, use it (strip any leading `v`). Otherwise query:

```sh
curl -fsSL https://api.github.com/repos/philippepascal/apm/releases/latest \
  | grep '"tag_name"' | sed 's/.*"v\([^"]*\)".*/\1/'
```

**4. Build download URLs**

```
BASE=https://github.com/philippepascal/apm/releases/download/v${VERSION}
ARCHIVE=${BASE}/apm-v${VERSION}-${TARGET}.tar.gz
CHECKSUM_URL=${BASE}/checksums.txt
```

**5. Download**

Download both the archive and `checksums.txt` to a temp directory (use `mktemp -d`, remove on EXIT via `trap`).

**6. Verify checksum**

Extract the expected SHA256 for the archive name from `checksums.txt`, then:

- On Linux: `sha256sum --check`
- On macOS: `shasum -a 256 --check`

Exit 1 with an error if the checksum does not match.

**7. Resolve install directory**

Use `${APM_INSTALL_DIR:-$HOME/.local/bin}`. Create with `mkdir -p` if absent.

**8. Extract and install**

```sh
tar -xzf "$ARCHIVE" -C "$TMPDIR"
cp "$TMPDIR/apm" "$INSTALL_DIR/"
cp "$TMPDIR/apm-server" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/apm" "$INSTALL_DIR/apm-server"
```

**9. Add to PATH**

For each of `$HOME/.bashrc`, `$HOME/.zshrc`, `$HOME/.profile`:
- If the file exists and does not already contain a line exporting `INSTALL_DIR`, append:

```sh
# Added by APM installer
export PATH="$INSTALL_DIR:$PATH"
```

Use a sentinel comment (`# Added by APM installer`) so uninstall can find and remove the exact block.

**10. Print success**

```
APM v{VERSION} installed to {INSTALL_DIR}

  apm --help        to get started
  apm-server --help for the web server

Restart your shell or run:  export PATH="{INSTALL_DIR}:$PATH"
```

---

### `scripts/uninstall.sh` — step by step

**1. Resolve install directory**

Use `${APM_INSTALL_DIR:-$HOME/.local/bin}`.

**2. Remove binaries**

If `$INSTALL_DIR/apm` exists, remove it; otherwise note it was not found. Same for `$INSTALL_DIR/apm-server`.

**3. Remove PATH entries**

For each of `$HOME/.bashrc`, `$HOME/.zshrc`, `$HOME/.profile`: if the file exists, remove the two-line block:

```
# Added by APM installer
export PATH="..."
```

Use a portable `sed` in-place pattern — on macOS `sed -i ''`, on Linux `sed -i`. Detect which form to use via `uname -s`.

**4. Print confirmation**

List each file/entry that was removed. If nothing was found, print "APM does not appear to be installed — nothing to do." and exit 0.

---

### Constraints / gotchas

- `sed -i` is not portable: macOS requires `-i ''`; Linux requires `-i` alone. Detect OS and branch.
- The GitHub API may rate-limit unauthenticated requests. The script should gracefully handle a failed version lookup (exit with a clear message telling the user to set `APM_VERSION`).
- Both scripts must be idempotent: running them twice should not double-add PATH entries or error on missing files.
- Do not use `bash`-isms (`[[ ]]`, `local`, process substitution, etc.) — keep everything POSIX `sh`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T17:07Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:44Z | groomed | in_design | philippepascal |