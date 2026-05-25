+++
id = "9c0edcbb"
title = "apm init creates commented-out manifest stubs for claude profiles"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9c0edcbb-apm-init-creates-commented-out-manifest-"
created_at = "2026-05-25T00:46:37.072625Z"
updated_at = "2026-05-25T00:46:37.072625Z"
+++

## Spec

### Problem

Ticket 4691685e introduced per-profile manifest files at .apm/agents/<agent>/<role>.toml that let users override model and env per profile. The feature works but is invisible: a fresh apm init repo contains no manifest files, and there is no hint in the generated output that they exist or what goes in them.

The fix is the same pattern used for config.toml in ticket 76dc81c5: generate the manifest files during apm init with every supported field commented out, so the default behaviour is unchanged but the format is self-documenting.

**Files to create during apm init**

.apm/agents/claude/spec-writer.toml:
  # model = "sonnet"   # model for the spec-writer profile; overrides [workers].model
  #
  # [env]
  # MY_VAR = "value"   # environment variables injected into spec-writer workers

.apm/agents/claude/coder.toml:
  # model = "sonnet"   # model for the coder profile; overrides [workers].model
  #
  # [env]
  # MY_VAR = "value"   # environment variables injected into coder workers

(Both files are identical in content — the header comment is the only difference.)

**Where the change goes**

apm-core/src/init.rs. The apm init path already creates .apm/agents/claude/ (to write the agent instruction .md files). Add two std::fs::write calls after the directory is created, one for each manifest stub. Use the same write_default() idempotency pattern already in use for config.toml: write only when the file is absent, do not overwrite user edits.

**Idempotency**

apm init must not overwrite an existing manifest file that the user has edited. Check for existence before writing, same as write_default() does for config.toml.

**Note on profile name**

This ticket assumes the rename in the companion ticket (worker → coder) is complete. The files should be named coder.toml and spec-writer.toml.

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
| 2026-05-25T00:46Z | — | new | philippepascal |
