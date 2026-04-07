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

- Removing or changing any Rust source code (no `repos` field exists in Config to delete)\n- Adding a `repos` field to Config or implementing the multi-repo feature\n- Changing `git_host` or any other config section\n- Updating archive files or historical specs that mention `repos.code`

### Approach

1. Open `.apm/config.toml`\n2. Delete the three lines that make up the `[[repos.code]]` block:\n   ```toml\n   [[repos.code]]\n   path = "philippepascal/apm"\n   default_branch = "main"\n   ```\n3. No code changes required — the Config struct has no `repos` field and the TOML parser silently ignores unknown keys, so this is a pure config-file deletion.\n4. Verify the config still loads cleanly (e.g. `apm config show` or `cargo test`).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T17:19Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:48Z | groomed | in_design | philippepascal |