# External TLS Setup with Passkey Authentication

This guide covers running apm-server with TLS on a public domain and
registering passkeys so remote browsers can access the dashboard.

---

## Prerequisites

- A domain with DNS pointing to your server's public IP (e.g. `apm.example.com`)
- Port 443 reachable from the internet (no firewall blocking)
- `apm-server` built (`cargo build --release -p apm-server`)

---

## 1. Configure apm.toml

Set `server.url` to your public HTTPS origin so the CLI talks to the right
endpoint:

```toml
[server]
url = "https://apm.example.com"
origin = "https://apm.example.com"
```

`origin` is used by WebAuthn to validate passkey challenges. It must match
the URL the browser sees exactly (scheme + host, no trailing slash).

---

## 2. Start the server

```bash
apm-server --tls --tls-domain apm.example.com --tls-email you@example.com --port 443
```

On first start, the server requests a certificate from Let's Encrypt via
TLS-ALPN-01. This takes a few seconds. You'll see:

```
Listening on https://0.0.0.0:443 (Let's Encrypt ACME)
acme event: DeployedNewCert
```

Certificates are cached in `~/.apm/certs/` and renewed automatically.

If you need to run on a non-privileged port, bind to 443 with port forwarding
or use `sudo` / `setcap`.

---

## 3. Register a passkey

Registration pairs a browser passkey with a username. The OTP step proves you
have local access to the server machine.

**On the server machine (localhost):**

```bash
apm register <username>
```

This prints a one-time password (OTP), valid for 5 minutes:

```
A7K2X9F1
```

**In your browser (any device):**

1. Open `https://apm.example.com/register`
2. Enter the same `<username>` and the OTP from above
3. Follow the browser's passkey prompt (Touch ID, security key, etc.)

On success, the browser receives a session cookie (`__Host-apm-session`) and
you're authenticated. Sessions last 7 days.

---

## 4. Log in on subsequent visits

Once registered, use the login flow instead:

1. Open `https://apm.example.com/login`
2. Enter your username
3. Authenticate with your passkey

No OTP is needed for login -- only for initial registration.

---

## 5. Managing sessions

From the server machine you can list and revoke active sessions:

```bash
# List active sessions
curl http://127.0.0.1:3000/api/auth/sessions

# Revoke all sessions
curl -X DELETE http://127.0.0.1:3000/api/auth/sessions
```

Note: the session management endpoints are localhost-only, so these commands
must be run on the server machine itself. When the server is running with TLS
on port 443, these endpoints are still accessible via loopback.

---

## How authentication works

- **Localhost requests** (127.0.0.1 / ::1) pass through without a session.
  This keeps the CLI and worker agents working without authentication.
- **External requests** require a valid `__Host-apm-session` cookie. Requests
  without one receive HTTP 401.
- **Auth endpoints** (`/register`, `/login`, and their API routes) are always
  open so unauthenticated browsers can complete the registration/login flow.
- **`/health`** is always open for monitoring.

---

## Troubleshooting

**`acme error: rateLimited`** -- Let's Encrypt limits failed authorizations to
5 per hour per domain. Wait for the retry time shown in the error, then try
again. Use `--tls=self-signed` or staging (in code: `.directory_lets_encrypt(false)`)
to test without hitting production rate limits.

**Safari: "did not accept the certificate"** -- If this happens immediately on
first start, the ACME certificate hasn't been issued yet. Check stderr for
`acme event: DeployedNewCert`. If you see `acme error:` instead, the challenge
is failing (usually a firewall or DNS issue).

**OTP expired** -- OTPs are valid for 5 minutes. Run `apm register <username>`
again to generate a new one.
