+++
id = "c9a5a1de"
title = "Add version command and version display in UI"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c9a5a1de-add-version-command-and-version-display-"
created_at = "2026-04-12T08:46:45.537269Z"
updated_at = "2026-04-12T08:46:45.537269Z"
+++

## Spec

### Problem

There is no way to check which version of apm is running or whether it's a development or release build. This matters for debugging, bug reports, and confirming deployments.

The version should be available both from the CLI (`apm version` or `apm -v`) and from the UI (displayed when clicking the "Supervisor" title in the supervisor panel).

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
| 2026-04-12T08:46Z | — | new | philippepascal |