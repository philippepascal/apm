+++
id = "de5c5a31"
title = "Deduplicate init.rs settings manipulation functions"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/de5c5a31-deduplicate-init-rs-settings-manipulatio"
created_at = "2026-04-12T09:02:35.167384Z"
updated_at = "2026-04-12T09:12:19.118254Z"
epic = "1b029f52"
target_branch = "epic/1b029f52-refactor-apm-cli-code-organization"
+++

## Spec

### Problem

`apm/src/cmd/init.rs` contains two near-identical private functions for patching `.claude/settings.json` files:

- `update_claude_settings(root, skip)` — patches the project-level `.claude/settings.json` with `APM_ALLOW_ENTRIES`
- `update_user_claude_settings()` — patches the user-level `~/.claude/settings.json` with `APM_USER_ALLOW_ENTRIES`

Both functions share ~60 lines of identical logic: read JSON (or create an empty object), navigate to `/permissions/allow`, diff against a target entry list, prompt the user, ensure the array path exists, append entries, and write back. The combined duplication spans ~140 lines across 305 total.

The only differences between them are: the resolved file path, the entry list, the prompt/confirmation strings, and whether a missing file causes an early-return (project case) or bootstraps an empty object (user case). All four differences are straightforward to parameterise.

The desired state is a single `fn update_settings_json(...)` helper that both callers delegate to, reducing the file by ~65 lines and making future changes (new allow entries, prompt wording, write logic) a single-site edit.

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
| 2026-04-12T09:02Z | — | new | philippepascal |
| 2026-04-12T09:08Z | new | groomed | apm |
| 2026-04-12T09:12Z | groomed | in_design | philippepascal |