# APM User Identity, Auth & Distribution Design

> Working doc — not committed. Updated as the design conversation progresses.

---

## Motivation

The current `agent` field in ticket frontmatter conflates two distinct concepts:
- **who created the ticket** (a human collaborator)
- **who is currently working on the ticket** (an ephemeral agent process)

In practice, agents are spawned once per ticket and never reused, making the agent name (e.g. `claude-0402-1430-a3f9`) low-signal noise in the frontmatter. Meanwhile there is no reliable way to ask "which human owns or created this ticket."

---

## Point 1 — `author` in frontmatter represents a collaborator username

### Current state

Frontmatter has an `author` field, currently set to the agent name or `"apm"` for automated transitions. There is no concept of a human collaborator identity.

### Collaborators list

The project maintains a list of known collaborators in the tracked config. Each entry is a username string. This list is either managed manually or synced from a git host (see point 4).

```toml
# .apm/config.toml (tracked)
[project]
collaborators = ["philippepascal", "alice", "bob"]
```

`"unassigned"` is an implicit member of every collaborators list — it is a reserved sentinel for tickets not yet owned by anyone.

### Local identity file

Each collaborator establishes their identity locally via a **gitignored, untracked file** at `.apm/local.toml`. This file is never committed. `apm init` prompts for a username and writes it.

```toml
# .apm/local.toml (gitignored, per-machine)
username = "philippepascal"
```

`.apm/local.toml` must be added to `.gitignore` by `apm init`.

### Resolution order for current user

1. Git host plugin authenticated identity — when a git host plugin is active (see point 4)
2. `username` in `.apm/local.toml` — explicit local identity
3. `"unassigned"` — fallback when neither source is available

`git config user.name` is **not** used — it is a free-text display name that does not reliably map to a collaborator username.

### `author` field semantics

- Set once at ticket creation to the resolved current-user identity
- Never changed after creation
- For tickets created by agents running under a human's identity, the human's username is used (the agent reads the same local identity file)
- For fully automated creation (CI, no local identity configured), value is `"apm"`

### Frontmatter example

```toml
id = "abc123"
title = "Fix login bug"
state = "ready"
author = "philippepascal"
branch = "ticket/abc123-fix-login-bug"
created_at = "2026-04-02T00:00:00Z"
```

### Migration

Field is already present on all tickets. Existing values (agent-name strings, `"apm"`) are left as-is — they are valid strings and no rewrite pass is needed. Going forward, new tickets get a real collaborator username.

---

## Point 2 — Remove agent name from frontmatter

### Why agent name has not been useful

- Workers are spawned once per ticket and die when the ticket reaches `implemented`. The agent identity string is meaningful only within a single session.
- The UI's worker panel already reads live state from `.apm-worker.pid` in each worktree — a more reliable and up-to-date source than a committed frontmatter field.
- The resumability use case ("pick up where I left off") is served by state + worktree presence, not by the agent name.
- apm is agent-agnostic (see point 6) — tying frontmatter to a specific agent naming convention is the wrong direction.

### Recommendation

Remove the `agent` field from frontmatter entirely.

- **Resumability**: `apm start <id>` already checks whether a worktree and branch exist; the agent name is not used in that logic. No change needed.
- **Live worker tracking**: the UI reads `.apm-worker.pid`. No change needed.
- **History**: the `## History` table in the ticket body already records which agent made each transition with a timestamp — sufficient for audit purposes.
- **`apm list` / `apm show`**: remove the `agent` column/field from output. The `state` column already conveys "being worked on."

### Migration

Drop `agent` from `Frontmatter` with `#[serde(default)]` semantics — the field is ignored on read, and new writes omit it. No rewrite of existing ticket files needed.

---

## Point 3 — Author automatically assigned on ticket creation

`apm new` resolves the current-user identity (see resolution order in point 1) and writes it to `author` in the new ticket's frontmatter. No manual step required.

`apm init` is the setup path: it prompts once for a username, validates it against the collaborators list if one exists, and writes `.apm/local.toml`.

### What "unassigned" means

`"unassigned"` is a reserved sentinel, not a real user. It:
- Displays distinctly in the UI (greyed out)
- Is filterable via `apm list --unassigned`
- Never matches a real username in any auth or assignment logic

---

## Point 4 — Git host plugin (GitHub)

When a GitHub plugin is configured, it provides two things:

1. **Current user identity** — resolved via GitHub API (`GET /user` with stored token, or `gh auth status`). This takes precedence over `.apm/local.toml`.
2. **Collaborators list** — synced from GitHub repo collaborators (`GET /repos/{owner}/{repo}/collaborators`) rather than maintained manually in config.

Configuration in `.apm/config.toml`:

```toml
[git_host]
provider = "github"
repo = "philippepascal/apm"
# token stored in .apm/local.toml (gitignored), not here
```

Token stored in `.apm/local.toml`:
```toml
username = "philippepascal"
github_token = "ghp_..."
```

The plugin is optional — everything degrades gracefully to the local-only flow when no git host is configured. Other providers (GitLab, Gitea) follow the same interface; GitHub is first.

---

## Point 5 — apm-server authentication

### TLS

Left to the operator. The `apm-proxy` Docker image (see point 6) is the recommended path — nginx + certbot in a single container handles TLS termination and automatic Let's Encrypt cert renewal. apm-server itself always speaks plain HTTP.

### Auth scheme: OTP bootstrap + WebAuthn/passkeys

WebAuthn is implemented from the start. The OTP serves as the trust gate for the registration ceremony — it is never itself a persistent credential.

**Why WebAuthn:**
- The device's private key is generated in and never leaves the secure enclave (TouchID, Windows Hello TPM)
- The server stores only public keys — a server compromise exposes nothing usable
- No phishing: the keypair is origin-bound, so a spoofed site gets a different key
- No shared secrets to rotate or leak
- Natural UX on mobile: TouchID/FaceID prompt on every login

**Requirement: HTTPS for external access.** WebAuthn is blocked by browsers on plain HTTP except for `localhost`. The local browser (same machine as apm-server) works without TLS; external devices require TLS — provided by `apm-proxy`.

**OTP generation:**

`apm register <username>` calls `POST /api/auth/otp` on localhost (trusted, no auth required). The server generates a short-lived OTP (8-char alphanumeric, 5-minute TTL, single-use), stores it, and the CLI prints it to stdout. The server owns the OTP; the CLI is only the trigger and display.

**Registration flow:**

1. `apm register <username>` — CLI calls localhost server, prints OTP
2. External browser hits apm-server — no registered credential → server returns registration page (username + OTP form)
3. User enters username + OTP → server validates (exists, not expired, not yet used, username is a known collaborator)
4. Server initiates WebAuthn registration ceremony: sends challenge + relying-party config to the browser
5. Browser asks OS to generate a keypair in the secure enclave; user confirms with biometric/PIN
6. Browser returns public key + attestation to server
7. Server stores public key for username; marks OTP consumed
8. Sets a session cookie for this browser session

**Login flow (subsequent visits):**

1. Browser hits apm-server — session cookie absent or expired → server returns login page
2. User enters username → server sends a WebAuthn challenge
3. Browser asks OS to sign the challenge with the stored private key; user confirms with biometric/PIN
4. Server verifies signature against stored public key → sets new session cookie

**Session cookie:**

After WebAuthn verification, the server issues a `__Host-apm-session` cookie (HttpOnly, Secure, SameSite=Lax) containing a random session token. Avoids a WebAuthn round-trip on every request. Session lifetime: 7 days (hardcoded default for now). `__Host-` prefix enforces Secure flag and host-locking at the browser level.

**Local access (same machine):**

Requests from `127.0.0.1` / `::1` bypass WebAuthn entirely — always trusted. Covers all `apm` CLI commands (including `apm register`) and the local browser during development.

**Session management:**

- `apm sessions` — list active sessions (username, device hint, last seen, expiry)
- `apm revoke <username>` — invalidate all sessions and registered credentials for a user
- `apm revoke --all` — full reset (useful if the server is moved to a new domain)
- Server-side state stored in `.apm/sessions.json` (gitignored), survives restarts
- Multiple credentials per user supported (phone + laptop can both be registered)

**Rust library:** `webauthn-rs`. Handles challenge generation, attestation parsing, assertion verification.

---

## Point 6 — Distribution and packaging

### Design principle

apm is a **single-developer tool**. The developer runs apm-server natively on their own machine alongside the repo. Workers (whatever agent is configured — Claude Code, or any other) spawn as native subprocesses with full filesystem access. Docker is not involved in the core workflow.

apm is **agent-agnostic**: it spawns whatever command is configured in `[agents]`. Claude Code is the default but not the only option. Nothing in the frontmatter, history, or config should assume a specific agent identity format.

### Native distribution (primary)

| Artifact | Channels |
|---|---|
| `apm` CLI + `apm-server` binary | GitHub Releases (pre-built), Homebrew tap, `cargo install` |
| Platforms | macOS arm64, macOS x86_64, Linux x86_64, Linux aarch64 |

Both binaries ship together in the same release — they are part of the same workspace and versioned together.

`apm-server` serves the built `apm-ui` static assets from an embedded binary (via `include_dir!` or similar) — no separate static file deployment needed.

### apm-proxy Docker image (remote access only)

For developers who want to access apm-server from a phone or remote laptop, `apm-proxy` is a lightweight Docker image containing only:
- **nginx** — reverse proxy + static file serving
- **certbot** — automatic Let's Encrypt cert provisioning and renewal

It contains no Rust, no Node, no repo access. It simply terminates TLS and proxies to the host's apm-server.

```
phone/laptop ──HTTPS──▶ apm-proxy (Docker, nginx+certbot) ──HTTP──▶ apm-server (native, :3000)
                                                                              │
                                                                    workers, worktrees, repo
```

**Usage:**

```bash
docker run -d \
  -p 80:80 -p 443:443 \
  -e DOMAIN=apm.example.com \
  -e EMAIL=you@example.com \
  -v apm-certs:/etc/letsencrypt \
  ghcr.io/philippepascal/apm-proxy
```

nginx proxies `https://apm.example.com/` → `http://host.docker.internal:3000` (macOS/Windows) or the host's IP on Linux.

**For LAN use without a public domain**, set `TLS_MODE=self-signed` — nginx generates a self-signed cert at startup. The browser will show a warning; the user accepts once.

---

## Open questions

- Should the collaborators list be validated strictly at `apm new` time (error if username not in list) or advisory (warn only)? Recommendation: warn only — strict validation breaks automated agents that may run before a collaborators list is configured.
- Should `assignee` be a separate field from `author` for "currently responsible human"? Deferred.
- Should `apm take <id>` update `assignee` (if introduced) or leave frontmatter unchanged? Deferred.
- Should multiple registered WebAuthn credentials per user share a single session store entry, or each get independent session tracking? Recommendation: independent — makes `apm revoke` per-device possible.
