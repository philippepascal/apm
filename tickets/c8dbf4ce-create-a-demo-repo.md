+++
id = "c8dbf4ce"
title = "create a demo repo"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/c8dbf4ce-create-a-demo-repo"
created_at = "2026-04-07T17:01:04.559759Z"
updated_at = "2026-04-07T17:43:43.554278Z"
+++

## Spec

### Problem

APM has no standalone public demo that a new user can clone and explore without first building a project from scratch. The only way to currently "kick the tires" is to run `apm init` on a blank repo (no pre-existing tickets, no context) or wade through the actual APM source tickets (complex, hundreds of entries, opaque to outsiders).

A purpose-built demo repo solves this by giving new users a realistic, self-contained project they can clone and immediately explore. It provides a believable software project with a representative ticket backlog, so every APM command has something meaningful to act on.

The demo must cover the full feature surface: multiple ticket states, epics, cross-ticket dependencies, the `apm-server` web UI, and the README-driven onboarding flow. Without it, the "getting started" story for APM is fragile and requires significant upfront investment from the user.

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
| 2026-04-07T17:01Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:43Z | groomed | in_design | philippepascal |