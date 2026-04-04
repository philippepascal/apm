+++
id = "bb7d2a61"
title = "apm-proxy: Docker image with nginx and certbot for TLS termination"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "apm"
branch = "ticket/bb7d2a61-apm-proxy-docker-image-with-nginx-and-ce"
created_at = "2026-04-02T20:54:51.005928Z"
updated_at = "2026-04-04T06:02:13.551670Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["48105624", "73e484df"]
+++

## Spec

### Problem

External access to apm-server (from a phone or remote laptop) requires TLS, but apm-server speaks plain HTTP and WebAuthn is blocked by browsers on plain HTTP for non-localhost origins. A lightweight Docker image containing only nginx (reverse proxy) and certbot (automatic Let's Encrypt cert renewal) handles TLS termination without adding complexity to the native server binary.

Currently there is no Docker image or nginx config in the repo. A developer who wants to expose apm-server externally has no supported path. This ticket creates the `apm-proxy/` directory containing a Dockerfile and supporting config files that implement the deployment model described in `initial_specs/DESIGN-users.md` point 6:

  phone/laptop ──HTTPS──▶ apm-proxy (Docker, nginx+certbot) ──HTTP──▶ apm-server (native, :3000)

apm-server itself is not changed — it continues to serve plain HTTP on localhost.

### Acceptance criteria

- [x] `docker build -t apm-proxy apm-proxy/` completes without error from repo root
- [x] Container started with `-e DOMAIN=<host> -e EMAIL=<addr> -p 80:80 -p 443:443` binds nginx on ports 80 and 443
- [x] Port 80 redirects all non-ACME HTTP traffic to `https://$DOMAIN/`
- [x] HTTPS traffic on port 443 is proxied to `http://host.docker.internal:3000` by default
- [x] Setting `-e UPSTREAM=<url>` overrides the proxy target (e.g. `http://192.168.1.10:3000` for Linux hosts where `host.docker.internal` is unavailable)
- [x] Setting `-e TLS_MODE=self-signed` generates a self-signed cert at startup; the container starts without requiring `EMAIL`
- [x] With default `TLS_MODE=letsencrypt`, certbot obtains a Let's Encrypt cert for `$DOMAIN` using `$EMAIL` before nginx begins serving HTTPS traffic
- [x] An ACME challenge directory (`/.well-known/acme-challenge/`) is served over HTTP, enabling webroot-based cert renewal without nginx downtime
- [x] A background renewal loop runs every 12 hours; nginx reloads automatically after a successful `certbot renew`
- [x] nginx forwards `Upgrade` and `Connection` headers, enabling WebSocket connections through the proxy
- [x] `proxy_buffering off` is set for the proxy location, enabling Server-Sent Events (SSE) streams used by apm-ui
- [x] Cert volume `/etc/letsencrypt` can be mounted to persist certificates across container restarts

### Out of scope

- Publishing the image to `ghcr.io/philippepascal/apm-proxy` (no CI workflow for the Docker image in this ticket)
- DNS configuration (operator responsibility)
- Multi-domain or wildcard TLS cert support
- HTTP/2, HSTS preloading, or other advanced TLS hardening beyond basic HTTPS
- Rate limiting, IP allowlists, or access control in nginx
- docker-compose file (the `docker run` command in DESIGN-users.md is the reference)
- Any changes to apm-server (it continues to serve plain HTTP)
- Homebrew or native binary distribution (covered by ticket #73e484df)

### Approach

Create a new top-level directory `apm-proxy/` containing three files. No existing files change.

**`apm-proxy/Dockerfile`**

Base image: `nginx:alpine`. Install `certbot`, `openssl`, `bash`, and `gettext` (provides `envsubst`). Copy `nginx.conf`, `nginx.conf.template`, and `entrypoint.sh`. Mark entrypoint executable. Expose 80 and 443. Set `CMD ["/entrypoint.sh"]`.

**`apm-proxy/nginx.conf`**

Replaces the default `/etc/nginx/nginx.conf`. Must include an `http { }` block with a WebSocket upgrade map and `include /etc/nginx/conf.d/*.conf`. The map block maps `$http_upgrade` to `$connection_upgrade` (default: `upgrade`, empty string: `close`). This must live in the `http` block, not a server block.

**`apm-proxy/nginx.conf.template`**

Template rendered at container startup via `envsubst` with only `DOMAIN` and `UPSTREAM` substituted. All other nginx variables are left untouched by passing an explicit variable list.

HTTP server block (port 80):
- `/.well-known/acme-challenge/` served from `/var/www/certbot` (webroot for certbot renewal)
- All other requests redirected 301 to `https://DOMAIN/`

HTTPS server block (port 443):
- SSL cert at `/etc/letsencrypt/live/DOMAIN/fullchain.pem` and `privkey.pem`
- `proxy_pass UPSTREAM`
- `proxy_http_version 1.1`
- Upgrade/Connection headers forwarded for WebSocket support
- `proxy_buffering off` for SSE

**`apm-proxy/entrypoint.sh`**

Steps in order:

1. Apply defaults: UPSTREAM defaults to `http://host.docker.internal:3000`, TLS_MODE defaults to `letsencrypt`
2. Render nginx config from template into `/etc/nginx/conf.d/default.conf`
3. Create `/var/www/certbot`
4. If TLS_MODE=self-signed: generate self-signed cert with openssl into `/etc/letsencrypt/live/DOMAIN/` and skip certbot
5. If TLS_MODE=letsencrypt:
   - Check for `/etc/letsencrypt/live/DOMAIN/cert.pem` (written by certbot; absent when only a bootstrap cert exists). Presence means a real LE cert is already on the mounted volume.
   - If no real cert: generate temp self-signed cert, start nginx in background, run `certbot certonly --webroot` to obtain LE cert, reload nginx
   - If real cert present: start nginx in background (skip issuance)
   - Start background renewal loop every 12 hours: `certbot renew --webroot` then `nginx -s reload`
6. `exec nginx -g 'daemon off;'`

**Cert path consistency:** Both modes write to `/etc/letsencrypt/live/DOMAIN/fullchain.pem` and `privkey.pem`. The nginx template references this path unconditionally.

**Known constraints:**
- `host.docker.internal` works on macOS and Windows Docker Desktop. On Linux, users must set `-e UPSTREAM=http://<host-ip>:3000` or pass `--add-host=host.docker.internal:host-gateway`
- Webroot approach (not --standalone) avoids nginx downtime during cert renewal
- The `cert.pem` presence check distinguishes a real LE cert from a temporary bootstrap self-signed cert on container restart

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:22Z | groomed | in_design | philippepascal |
| 2026-04-03T00:26Z | in_design | specd | claude-0402-2022-b7d2 |
| 2026-04-04T00:29Z | specd | ready | apm |
| 2026-04-04T03:04Z | ready | in_progress | philippepascal |
| 2026-04-04T03:06Z | in_progress | implemented | claude-0403-0300-f4e1 |
| 2026-04-04T06:02Z | implemented | closed | apm-sync |
