# Docker Worker Setup Guide

This guide walks you through running APM worker agents inside Docker containers.
Isolated containers prevent workers from touching your host filesystem and make
credential injection explicit.

---

## Prerequisites

- **Docker** installed and running (Docker Desktop on macOS/Linux, or Docker
  Engine on Linux). Verify with `docker version`.
- **`apm`** installed and a repo already initialised with `apm init`.

Generate the worker Dockerfile:

```bash
apm init --with-docker
```

This creates `.apm/Dockerfile.apm-worker` in your repo. The command is
**idempotent** — if the file already exists it will not be overwritten.

---

## Customise the Dockerfile

Open `.apm/Dockerfile.apm-worker`. The generated template looks like this:

```dockerfile
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl git ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install claude CLI
RUN curl -fsSL https://claude.ai/install.sh | sh

WORKDIR /workspace
```

Add whatever language runtimes or tools your project needs.

**Node.js example:**

```dockerfile
RUN apt-get update && apt-get install -y \
    curl git ca-certificates nodejs npm \
    && rm -rf /var/lib/apt/lists/*
```

**Python example:**

```dockerfile
RUN apt-get update && apt-get install -y \
    curl git ca-certificates python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*
```

Make sure the `claude` binary ends up in `PATH` — the worker entrypoint depends
on it. If you change the base image, verify that `curl` and `git` are still
available.

---

## Build the image

From your repo root:

```bash
docker build -f .apm/Dockerfile.apm-worker -t apm-worker .
```

The `-t apm-worker` tag is what you will reference in `.apm/config.toml`. You
can use any tag you like (e.g. `apm-worker:1.2.3`), but keep it consistent with
the config.

Rebuild whenever you change the Dockerfile.

---

## Configure `.apm/config.toml`

Add a `[workers]` section and, on macOS, a `[workers.keychain]` section:

```toml
[workers]
container = "apm-worker:latest"

[workers.keychain]
ANTHROPIC_API_KEY = "anthropic-api-key"
GIT_AUTHOR_NAME   = "git-user-name-keychain-service"
GIT_AUTHOR_EMAIL  = "git-user-email-keychain-service"
```

### `[workers]`

| Key | Description |
|-----|-------------|
| `container` | Docker image tag to use when spawning workers |

### `[workers.keychain]` — macOS only

Each entry maps an **environment variable name** (left) to a **macOS Keychain
service name** (right). APM calls `security find-generic-password -s <service>
-w` at spawn time and injects the result as the named env var into the
container.

**Linux users:** macOS Keychain is not available on Linux. Set credentials as
plain environment variables in the shell that runs `apm` instead:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
export GIT_AUTHOR_NAME="Bot Name"
apm work
```

---

## Verify with `apm validate`

```bash
apm validate
```

**Passing output:**

```
✓ config: .apm/config.toml found
✓ docker: image apm-worker:latest found
✓ keychain: ANTHROPIC_API_KEY resolved
```

**Failing output examples:**

```
✗ docker: docker not found in PATH
  → Ensure Docker Desktop is running and docker is in your PATH
```

```
✗ docker: image apm-worker:latest not found
  → Run: docker build -f .apm/Dockerfile.apm-worker -t apm-worker .
```

Fix every `✗` entry before spawning workers.

---

## Run workers

### Dispatch a single ticket

```bash
apm start --spawn <ticket-id>
```

Spawns one container for the given ticket and returns immediately. The container
runs the worker, opens a PR, and marks the ticket `implemented`.

### Dispatch the highest-priority ready ticket

```bash
apm start --next --spawn
```

Picks the highest-priority `ready` ticket automatically.

### Batch orchestration loop

```bash
apm work
```

Runs `apm start --next --spawn` repeatedly until no `ready` tickets remain,
keeping up to `[agents] max_concurrent` (default `3`) workers running in
parallel. Use this when you have a queue of tickets to drain.

> **`apm work --daemon`** runs the loop continuously, polling for new `ready`
> tickets as they arrive. It is outside the scope of this guide.

---

## Troubleshooting

### Credential not found

**Symptom:** `apm validate` reports `✗ keychain: ANTHROPIC_API_KEY not found`
or a worker container exits with an authentication error.

**Fix:** Confirm the Keychain entry exists and the service name matches exactly:

```bash
security find-generic-password -s anthropic-api-key -w
```

If it returns nothing, add the secret via Keychain Access.app or:

```bash
security add-generic-password -s anthropic-api-key -a "$USER" -w
```

You will be prompted for the password. Alternatively, remove the entry from
`[workers.keychain]` and pass the credential via environment variable instead.

---

### docker not in PATH

**Symptom:** `apm validate` reports `✗ docker: docker not found in PATH`.

**Fix:** Ensure Docker Desktop is running. On macOS, Docker Desktop installs
the CLI at `/usr/local/bin/docker` or `/usr/bin/docker`. Check which shell APM
inherits:

```bash
echo $PATH
which docker
```

If `docker` is only available in interactive shells (e.g. via `.zshrc`), add
its directory to `/etc/paths` or set `PATH` in your launch environment so that
non-interactive shells see it.

---

### Container exits immediately

**Symptom:** The worker container starts and stops within seconds; the ticket
stays `in_progress` and no PR is opened.

**Fix:** Run the image manually to see the error:

```bash
docker run --rm apm-worker:latest
```

Common causes:

| Cause | Fix |
|-------|-----|
| `claude` binary missing | Verify the Dockerfile installs the Claude CLI and that the install script completed without error during `docker build` |
| Network unreachable | The container needs outbound HTTPS; check Docker Desktop network settings |
| Wrong entrypoint | Do not override `CMD` or `ENTRYPOINT` in the Dockerfile unless you know what the worker expects |
