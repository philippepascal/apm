+++
id = "cfcf8e68"
title = "pi/manifest.toml missing [wrapper] section"
state = "closed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/cfcf8e68-pi-manifest-toml-missing-wrapper-section"
created_at = "2026-05-06T22:56:05.246110Z"
updated_at = "2026-05-06T23:39:35.191677Z"
+++

## Spec

### Problem

Ticket 80691f15 created .apm/agents/pi/manifest.toml without a [wrapper] section header. APM's parse_manifest() deserializes into struct ManifestFile { wrapper: Manifest } so a bare top-level manifest fails with 'not valid TOML'. Fixed in ticket 4726eac0 branch by adding [wrapper] header. Ticket 80691f15 should be amended so its branch is also correct.

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
| 2026-05-06T22:56Z | — | new | claude-0506-2251-a798|philippepascal |
| 2026-05-06T23:39Z | new | closed | philippepascal(apm-sync) |