+++
id = "0c197dd4"
title = "Remove unused [[repos.code]] section from config"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/0c197dd4-remove-unused-repos-code-section-from-co"
created_at = "2026-04-07T17:19:55.074233Z"
updated_at = "2026-04-07T17:48:33.966473Z"
+++

## Spec

### Problem

The [[repos.code]] section in .apm/config.toml is not parsed by the Config struct — there is no repos field. It is dead config left over from an earlier design. It should be removed to avoid confusion.

### Acceptance criteria

- [ ] `[[repos.code]]` section is absent from `.apm/config.toml`\n- [ ] `apm config show` (or equivalent) loads successfully after the removal\n- [ ] No other file in the repo still references `[[repos.code]]` in non-archival, non-ticket contexts

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
| 2026-04-07T17:19Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:48Z | groomed | in_design | philippepascal |