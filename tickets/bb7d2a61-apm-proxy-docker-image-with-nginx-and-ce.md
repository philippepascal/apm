+++
id = "bb7d2a61"
title = "apm-proxy: Docker image with nginx and certbot for TLS termination"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "50928"
branch = "ticket/bb7d2a61-apm-proxy-docker-image-with-nginx-and-ce"
created_at = "2026-04-02T20:54:51.005928Z"
updated_at = "2026-04-03T00:22:47.984502Z"
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

- [ ] `docker build -t apm-proxy apm-proxy/` completes without error from repo root
- [ ] Container started with `-e DOMAIN=<host> -e EMAIL=<addr> -p 80:80 -p 443:443` binds nginx on ports 80 and 443
- [ ] Port 80 redirects all non-ACME HTTP traffic to `https://$DOMAIN/`
- [ ] HTTPS traffic on port 443 is proxied to `http://host.docker.internal:3000` by default
- [ ] Setting `-e UPSTREAM=<url>` overrides the proxy target (e.g. `http://192.168.1.10:3000` for Linux hosts where `host.docker.internal` is unavailable)
- [ ] Setting `-e TLS_MODE=self-signed` generates a self-signed cert at startup; the container starts without requiring `EMAIL`
- [ ] With default `TLS_MODE=letsencrypt`, certbot obtains a Let's Encrypt cert for `$DOMAIN` using `$EMAIL` before nginx begins serving HTTPS traffic
- [ ] An ACME challenge directory (`/.well-known/acme-challenge/`) is served over HTTP, enabling webroot-based cert renewal without nginx downtime
- [ ] A background renewal loop runs every 12 hours; nginx reloads automatically after a successful `certbot renew`
- [ ] nginx forwards `Upgrade` and `Connection` headers, enabling WebSocket connections through the proxy
- [ ] `proxy_buffering off` is set for the proxy location, enabling Server-Sent Events (SSE) streams used by apm-ui
- [ ] Cert volume `/etc/letsencrypt` can be mounted to persist certificates across container restarts

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
| 2026-04-02T20:54Z | — | new | apm |
| 2026-04-02T23:23Z | new | groomed | apm |
| 2026-04-03T00:22Z | groomed | in_design | philippepascal |