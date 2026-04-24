+++
id = "12e947b1"
title = "apm init Claude settings allow-list missing commands"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/12e947b1-apm-init-claude-settings-allow-list-miss"
created_at = "2026-04-24T06:28:47.480554Z"
updated_at = "2026-04-24T06:28:47.480554Z"
+++

## Spec

### Problem

APM_ALLOW_ENTRIES (apm/src/cmd/init.rs:121-136) and APM_USER_ALLOW_ENTRIES (init.rs:140-156) define the subset of apm commands added to .claude/settings.json and ~/.claude/settings.json so Claude does not prompt for each invocation. Several commonly-used apm commands are missing. Known gaps observed during ticker project use: apm help (triggers prompt mid-session), apm review, apm close, apm register, apm epic*, apm archive*, apm clean*, apm work*, apm assign*, apm validate*, apm version*, apm sessions*, apm revoke*. Expected: audit both lists against the canonical command set in apm/src/cmd/*.rs and add the missing entries. Use the "apm <sub>*" glob for each to cover args.

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
| 2026-04-24T06:28Z | — | new | philippepascal |
