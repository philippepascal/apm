+++
id = "a7cff3aa"
title = "Homebrew tap formula for apm CLI + apm-server"
state = "closed"
priority = 0
effort = 2
risk = 2
author = "apm"
branch = "ticket/a7cff3aa-homebrew-tap-formula-for-apm-cli-apm-ser"
created_at = "2026-04-02T20:54:55.761604Z"
updated_at = "2026-04-06T23:19:15.100735Z"
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

- [x] A public GitHub repository `philippepascal/homebrew-tap` exists
- [x] The repository contains a formula file `Formula/apm.rb`
- [x] `brew tap philippepascal/tap` succeeds without errors
- [ ] `brew install philippepascal/tap/apm` installs both `apm` and `apm-server` binaries to the Homebrew prefix
- [ ] After installation, `apm --help` runs successfully
- [ ] After installation, `apm-server --help` runs successfully
- [x] The formula downloads the correct archive for the host architecture (arm64 on Apple Silicon, x86_64 on Intel)
- [x] The formula verifies the SHA-256 checksum of the downloaded archive
- [ ] `brew uninstall apm` cleanly removes both binaries
- [x] The formula includes a `test` block that verifies both binaries execute (e.g. `apm --help` and `apm-server --help`)

### Out of scope

- Automated formula version bumps when a new release is tagged (follow-up ticket)
- Linux Homebrew (Linuxbrew) support — Linux users download binaries directly or use cargo install
- Building from source via Homebrew (`brew install --build-from-source`) — formula uses pre-built bottles only
- The release CI itself (ticket #73e484df)
- Publishing to Homebrew core (requires significant adoption; tap is the right path for now)
- Windows package managers (scoop, chocolatey, winget)
- Formula for apm-proxy Docker image

### Approach

This ticket creates a new GitHub repository and a single Ruby formula file. No changes to the apm repo itself.

**Step 1: Create the tap repository**

Create `philippepascal/homebrew-tap` on GitHub (public). This is the standard Homebrew tap naming convention — `brew tap philippepascal/tap` automatically resolves to `github.com/philippepascal/homebrew-tap`.

**Step 2: Write `Formula/apm.rb`**

The formula uses Homebrew's `on_macos` + `on_intel`/`on_arm` DSL to select the correct archive URL and checksum per architecture. The release CI (ticket #73e484df) produces archives with these names:

- `apm-v{VERSION}-aarch64-apple-darwin.tar.gz` (macOS arm64)
- `apm-v{VERSION}-x86_64-apple-darwin.tar.gz` (macOS x86_64)

Each archive contains both `apm` and `apm-server` binaries at the top level.

Formula structure:

```ruby
class Apm < Formula
  desc "Agentic project manager — CLI and server"
  homepage "https://github.com/philippepascal/apm"
  version "0.1.0"  # Update on each release
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/philippepascal/apm/releases/download/v#{version}/apm-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "<sha256-for-arm64>"
    end
    on_intel do
      url "https://github.com/philippepascal/apm/releases/download/v#{version}/apm-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "<sha256-for-x86_64>"
    end
  end

  def install
    bin.install "apm"
    bin.install "apm-server"
  end

  test do
    assert_match "apm", shell_output("#{bin}/apm --help")
    assert_match "apm-server", shell_output("#{bin}/apm-server --help")
  end
end
```

Key points:
- `url` uses `version` interpolation so only the `version` line and SHA-256 hashes need updating per release
- Both binaries are installed to `bin/` via `bin.install`
- The `test` block runs `--help` on both binaries to verify they execute
- SHA-256 values come from `checksums.txt` in the GitHub Release (produced by ticket #73e484df)

**Step 3: Populate with real checksums**

This ticket depends on #73e484df (release CI). The formula must be populated with real SHA-256 checksums from an actual release. The initial commit can use placeholder checksums with a clear comment, and the first real release triggers updating them.

**Step 4: Verify**

```bash
brew tap philippepascal/tap
brew install philippepascal/tap/apm
apm --help
apm-server --help
brew test apm
brew uninstall apm
```

**Gotchas**
- Homebrew requires the tap repo to be public. If the main apm repo is private, the release assets must still be downloadable (GitHub Releases on private repos require auth tokens, which Homebrew doesn't support natively). For now, the main repo is public so this is not an issue.
- The formula file must be named `apm.rb` (lowercase, matching the formula class name `Apm`).
- If the license field in the formula doesn't match the repo's license file, `brew audit` will warn. Use whatever license the repo uses.

### Open questions

**Q:** The tap repo and formula are created (https://github.com/philippepascal/homebrew-tap). The formula structure is complete with on_arm/on_intel DSL, sha256 fields, bin.install for both binaries, and a test block.

**Q:** Four installation-verification criteria remain unverified because no GitHub Release exists yet (ticket #73e484df release CI is implemented but no v0.1.0 tag has been pushed):

**Q:** - `brew install philippepascal/tap/apm` installs both binaries
**Q:** - After installation, `apm --help` runs
**Q:** - After installation, `apm-server --help` runs
**Q:** - `brew uninstall apm` cleanly removes both binaries

**Q:** **To complete this ticket:** push a v0.1.0 tag to trigger the release CI, then update Formula/apm.rb in homebrew-tap with the real SHA-256 values from checksums.txt in the release, and verify the install/uninstall criteria manually.

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
| 2026-04-03T23:22Z | in_design | specd | claude-0403-2321-b7e2 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T02:59Z | ready | in_progress | philippepascal |
| 2026-04-04T03:03Z | in_progress | blocked | claude-0403-1430-w9k2 |
| 2026-04-06T23:19Z | blocked | closed | apm |
