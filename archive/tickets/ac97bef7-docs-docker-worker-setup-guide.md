+++
id = "ac97bef7"
title = "docs: Docker worker setup guide"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
agent = "91470"
branch = "ticket/ac97bef7-docs-docker-worker-setup-guide"
created_at = "2026-03-30T19:55:27.882184Z"
updated_at = "2026-03-31T05:05:18.226155Z"
+++

## Spec

### Problem

The Docker sandbox feature (ticket #0038) adds worker isolation via Docker containers, but there is no user-facing documentation explaining how to set it up. The feature involves several manual steps — building an image, configuring `apm.toml`, optionally setting up macOS Keychain entries — that are not obvious from the CLI help text alone.

A `docs/` folder should be established in the repo, and a `docs/docker-workers.md` guide should walk users through the full setup end-to-end:

1. Prerequisites (Docker installed, `apm init --with-docker` to generate the Dockerfile)
2. Customising `Dockerfile.apm-worker` for the project's language/toolchain
3. Building the image
4. Configuring `[workers]` and `[workers.keychain]` in `.apm/config.toml`
5. Verifying the setup with `apm validate`
6. Running workers with `apm work` or `apm start --spawn`
7. Troubleshooting (credential not found, docker not in PATH, container exits immediately)

Note: the Problem section mentions `apm.toml` but the actual config file is `.apm/config.toml` (with `apm.toml` as a legacy fallback). The guide should use `.apm/config.toml`.

### Acceptance criteria

- [x] A `docs/` directory exists at the repo root
- [x] `docs/docker-workers.md` exists inside that directory
- [x] The guide opens with a prerequisites section listing Docker and the `apm init --with-docker` command
- [x] The guide explains what `apm init --with-docker` creates (`.apm/Dockerfile.apm-worker`) and that it is idempotent
- [x] The guide shows how to customise `Dockerfile.apm-worker` for a project's language/toolchain with at least one concrete example (e.g. adding Node.js or Python packages)
- [x] The guide shows the exact `docker build` command to build the image
- [x] The guide shows a complete `.apm/config.toml` example with both `[workers]` and `[workers.keychain]` sections
- [x] The guide explains that `[workers.keychain]` values are macOS Keychain service names
- [x] The guide explains that Linux users must supply credentials via environment variables instead of Keychain
- [x] The guide shows how to run `apm validate` and what passing and failing Docker checks look like
- [x] The guide explains the difference between `apm start --spawn` (dispatch a single ticket) and `apm work` (batch orchestration loop)
- [x] The troubleshooting section addresses: credential not found, docker not in PATH, container exits immediately

### Out of scope

- Documentation for non-Docker (native) worker setup
- Documentation for `apm work --daemon` mode beyond a brief mention
- Windows support (Docker Desktop on Windows is not tested)
- CI/CD pipeline integration (e.g. running workers in GitHub Actions)
- The underlying implementation of the Docker sandbox feature (covered by ticket #0038)

### Approach

This ticket creates two things: a `docs/` directory at the repo root and a single `docs/docker-workers.md` file. No code changes.

**File to create:** `docs/docker-workers.md`

**Document structure:**

1. **Prerequisites** — Docker installed and in PATH; `apm` installed; run `apm init --with-docker` to generate `.apm/Dockerfile.apm-worker` (note: idempotent, will not overwrite)

2. **Customise the Dockerfile** — show the generated template and add a concrete before/after example for a project using Node.js (adding `RUN apt-get install -y nodejs npm`) and Python (adding `RUN apt-get install -y python3`)

3. **Build the image** — show the exact command:
   ```
   docker build -f .apm/Dockerfile.apm-worker -t apm-worker .
   ```
   Explain the tag name is what goes in `config.toml`.

4. **Configure `.apm/config.toml`** — show a complete example:
   ```toml
   [workers]
   container = "apm-worker:latest"

   [workers.keychain]
   ANTHROPIC_API_KEY = "anthropic-api-key"
   GIT_AUTHOR_NAME = "git-user-name-keychain-service"
   ```
   Explain that `[workers.keychain]` maps env var names to Keychain service names (macOS only). Linux users: set env vars directly (`ANTHROPIC_API_KEY=...`) instead of using `[workers.keychain]`.

5. **Verify with `apm validate`** — show what passing output looks like (`✓ docker: image apm-worker:latest found`) and what failing looks like (docker not in PATH warning, or image not found).

6. **Run workers** — explain:
   - `apm start --spawn <id>` — dispatches a single ticket to a container
   - `apm start --next --spawn` — picks the highest-priority ready ticket
   - `apm work` — runs the dispatch loop up to `[agents] max_concurrent` (default 3) workers in parallel

7. **Troubleshooting** — three subsections:
   - *Credential not found*: `security find-generic-password` returns nothing; check the Keychain service name with `security find-generic-password -s <service> -w`; or switch to env var
   - *docker not in PATH*: `apm validate` warns; ensure Docker Desktop is running and `/usr/local/bin/docker` (or equivalent) is in the shell PATH that apm uses
   - *Container exits immediately*: run `docker run --rm apm-worker:latest` manually to see the error output; common cause is missing `claude` binary in the image

**Implementation steps:**
1. `mkdir docs` in the worktree
2. Write `docs/docker-workers.md` with the above structure
3. Commit: `git -C <wt> add docs/docker-workers.md && git -C <wt> commit -m "docs: add Docker worker setup guide"`
4. Open a PR targeting `main`
5. `apm state ac97bef7 implemented`

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T19:55Z | — | new | philippepascal |
| 2026-03-30T20:00Z | new | in_design | philippepascal |
| 2026-03-30T20:06Z | in_design | specd | claude-0330-2000-b7d2 |
| 2026-03-30T20:10Z | specd | ready | apm |
| 2026-03-30T20:10Z | ready | in_progress | philippepascal |
| 2026-03-30T20:12Z | in_progress | implemented | claude-0330-2015-d4f2 |
| 2026-03-30T20:31Z | implemented | accepted | apm-sync |
| 2026-03-31T05:05Z | accepted | closed | apm-sync |