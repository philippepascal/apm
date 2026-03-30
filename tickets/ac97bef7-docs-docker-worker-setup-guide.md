+++
id = "ac97bef7"
title = "docs: Docker worker setup guide"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "46385"
branch = "ticket/ac97bef7-docs-docker-worker-setup-guide"
created_at = "2026-03-30T19:55:27.882184Z"
updated_at = "2026-03-30T20:00:35.089582Z"
+++

## Spec

### Problem

The Docker sandbox feature (ticket #0038) adds worker isolation via Docker containers, but there is no user-facing documentation explaining how to set it up. The feature involves several manual steps — building an image, configuring `apm.toml`, optionally setting up macOS Keychain entries — that are not obvious from the CLI help text alone.

A `docs/` folder should be established in the repo, and a `docs/docker-workers.md` guide should walk users through the full setup end-to-end:

1. Prerequisites (Docker installed, `apm init --with-docker` to generate the Dockerfile)
2. Customising `Dockerfile.apm-worker` for the project's language/toolchain
3. Building the image
4. Configuring `[workers]` and `[workers.keychain]` in `apm.toml`
5. Verifying the setup with `apm validate`
6. Running workers with `apm work` or `apm start --spawn`
7. Troubleshooting (credential not found, docker not in PATH, container exits immediately)

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T19:55Z | — | new | philippepascal |
| 2026-03-30T20:00Z | new | in_design | philippepascal |