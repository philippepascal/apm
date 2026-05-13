+++
id = "fcf0b213"
title = "apm command to feed apm parsed help for agents"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/fcf0b213-apm-command-to-feed-apm-parsed-help-for-"
created_at = "2026-05-07T20:41:08.889701Z"
updated_at = "2026-05-13T00:11:22.186416Z"
+++

## Spec

### Problem

currently agents load markdown files that are static to learn how to use apm. instead apm needs a special command similar to help but specialized for agents. it may be a subcommand of apm help. it may have subcommand for every other apm commands. it needs to be very precise to improve agent understanding of apm commands, and very compact as it will be used often by agents.

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
| 2026-05-07T20:41Z | — | new | philippepascal |