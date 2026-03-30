+++
id = "b797d318"
title = "apm ticket ID prefix matching: improve resolution and error messages"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/b797d318-apm-ticket-id-prefix-matching-improve-re"
created_at = "2026-03-30T16:56:24.264985Z"
updated_at = "2026-03-30T16:56:24.264985Z"
+++

## Spec

### Problem

When resolving a ticket ID from a short prefix, APM has two bugs:

1. **Unique prefix not resolved**: `apm review 314` fails with "no ticket matches '0314'" even when exactly one ticket has an ID starting with `314`. The prefix is unique — there is no ambiguity — but APM rejects it instead of resolving it.

2. **Ambiguous prefix error is unhelpful**: When multiple tickets share the same prefix, the error message does not list the candidates. The user has no way to disambiguate without running `apm list` separately and scanning manually.

The correct behaviour:
- Unique prefix → resolve silently (already works for longer prefixes like `3142`, broken for shorter ones like `314`)
- Ambiguous prefix → list all matching ticket IDs and titles, ask the user to be more specific

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
| 2026-03-30T16:56Z | — | new | philippepascal |