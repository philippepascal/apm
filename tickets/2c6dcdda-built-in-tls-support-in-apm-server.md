+++
id = "2c6dcdda"
title = "Built-in TLS support in apm-server"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/2c6dcdda-built-in-tls-support-in-apm-server"
created_at = "2026-04-06T06:09:24.235043Z"
updated_at = "2026-04-06T06:09:24.235043Z"
+++

## Spec

### Problem

apm-server currently runs plain HTTP only. Production deployments require a separate reverse proxy (nginx via apm-proxy Docker image) for TLS termination. This adds operational complexity — users need Docker, a separate component, and a different mental model — undermining APM's single-binary simplicity.

The goal is to make production HTTPS as easy as `apm-server --tls --domain=apm.example.com --email=you@example.com`, with automatic certificate management built into the binary itself.

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
| 2026-04-06T06:09Z | — | new | philippepascal |