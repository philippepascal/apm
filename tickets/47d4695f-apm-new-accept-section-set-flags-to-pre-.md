+++
id = "47d4695f"
title = "apm new: accept --section/--set flags to pre-populate spec sections"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/47d4695f-apm-new-accept-section-set-flags-to-pre-"
created_at = "2026-03-31T00:05:27.351459Z"
updated_at = "2026-03-31T00:05:33.516592Z"
+++

## Spec

### Problem

apm new accepts --no-edit to skip the interactive editor, but agents cannot pre-populate spec sections in a single command. Without section content, the ticket is created empty in `new` state and immediately eligible for pickup by a running `apm work` daemon — a worker may start writing the spec before the creating agent has a chance to fill it in.

Interactive users avoid this because the editor opens synchronously during `apm new`, keeping the ticket in a transient state until they save and close. Agents have no equivalent: they must create the ticket first, then make separate `apm spec` calls — a window where the ticket is vulnerable to premature worker pickup.

The fix is to allow `--section`/`--set` pairs on `apm new`, with the same API as `apm spec`. Sections are written into the ticket file before the first commit, so the ticket never exists in an empty `new` state.

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
| 2026-03-31T00:05Z | — | new | apm |
| 2026-03-31T00:05Z | new | in_design | apm |