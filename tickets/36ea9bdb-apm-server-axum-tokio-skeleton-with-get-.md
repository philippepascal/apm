+++
id = "36ea9bdb"
title = "apm-server: axum/tokio skeleton with GET /health endpoint"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "philippepascal"
branch = "ticket/36ea9bdb-apm-server-axum-tokio-skeleton-with-get-"
created_at = "2026-03-31T06:05:42.967376Z"
updated_at = "2026-03-31T06:06:10.291031Z"
+++

## Spec

### Problem

The UI needs a Rust HTTP backend. Create the apm-server crate (or apm serve command) with axum + tokio. The only endpoint at this stage is GET /health returning {"ok":true}. No business logic yet — the goal is to confirm the crate compiles, ships, and serves. Full spec context: initial_specs/UIdraft_spec_starter.md Step 1.

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
| 2026-03-31T06:05Z | — | new | apm |
| 2026-03-31T06:06Z | new | in_design | philippepascal |
