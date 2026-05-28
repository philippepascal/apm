+++
id = "d2720f0b"
title = "apm new editor flow must not checkout the ticket branch in main"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/d2720f0b-apm-new-editor-flow-must-not-checkout-th"
created_at = "2026-05-28T07:37:16.051173Z"
updated_at = "2026-05-28T07:37:16.051173Z"
+++

## Spec

### Problem

apm new without --no-edit currently checks out the ticket branch in the main repo in order to open the ticket file in the editor. This means the main repo's HEAD moves to the ticket branch for the duration of the editor session, which is a side effect that can interfere with server dispatch (see f16e4035). Fix: write the ticket file to a temp path, open that in the editor, then commit the result via plumbing to the ticket branch. The main repo working tree never moves. This makes --no-edit a performance flag rather than a safety requirement for agents.

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
| 2026-05-28T07:37Z | — | new | philippepascal |
