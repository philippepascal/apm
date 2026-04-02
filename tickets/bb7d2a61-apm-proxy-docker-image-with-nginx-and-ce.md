+++
id = "bb7d2a61"
title = "apm-proxy: Docker image with nginx and certbot for TLS termination"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/bb7d2a61-apm-proxy-docker-image-with-nginx-and-ce"
created_at = "2026-04-02T20:54:51.005928Z"
updated_at = "2026-04-02T20:54:51.005928Z"
epic = "8db73240"
target_branch = "epic/8db73240-user-mgmt"
depends_on = ["48105624", "73e484df"]
+++

## Spec

### Problem

External access to apm-server (from a phone or remote laptop) requires TLS, but apm-server speaks plain HTTP and WebAuthn is blocked by browsers on plain HTTP for non-localhost origins. A lightweight Docker image containing only nginx (reverse proxy) and certbot (automatic Let's Encrypt cert renewal) handles TLS termination without adding complexity to the native server binary. See `initial_specs/DESIGN-users.md` point 6.

### Acceptance criteria

Checkboxes; each one independently testable.

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