+++
id = "9c0edcbb"
title = "apm init creates commented-out manifest stubs for claude profiles"
state = "closed"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9c0edcbb-apm-init-creates-commented-out-manifest-"
created_at = "2026-05-25T00:46:37.072625Z"
updated_at = "2026-05-25T06:56:44.305131Z"
depends_on = ["daf83745"]
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

- [x] `apm init` on a fresh repo creates `.apm/agents/claude/spec-writer.toml` with all fields commented out
- [x] `apm init` on a fresh repo creates `.apm/agents/claude/coder.toml` with all fields commented out
- [x] The content of each stub file is valid TOML (parses without error; the all-comment file produces an empty table)
- [x] `apm init` output includes a `Created .apm/agents/claude/spec-writer.toml` line and a `Created .apm/agents/claude/coder.toml` line on first run
- [x] Re-running `apm init` when stub files are unchanged produces no new output for those files (idempotent)
- [x] Re-running `apm init` when a stub file has been edited by the user does not overwrite the user's content; a `.init` comparison copy is written instead

### Out of scope

- Manifest stubs for agent directories other than `claude` (mock-happy, mock-sad, mock-random, debug)
- Generating stubs for user-defined custom profiles beyond `spec-writer` and `coder`
- Adding new fields to `WorkerProfileManifest` — the struct in `start.rs` is unchanged
- Updating the `apm validate` or `apm instructions` output to document manifest files

### Approach

All changes are in apm-core/src/init.rs.

#### Stub content constants

Add two module-level string constants near the other default-content functions. Each is an all-comment TOML file stub exposing the two supported fields (model and [env]). The comments differ only in the profile name mentioned.

#### Write calls in setup()

After the existing write_default call that writes apm.coder.md (currently line 131), add two more write_default calls: one for spec-writer.toml and one for coder.toml, both rooted at agents_claude_dir.

write_default already handles idempotency: create when absent, no-op when content matches, write a .init copy when the user has edited the file.

#### Test updates

In setup_creates_expected_files (line 724), add assertions that both new files exist.

Add setup_creates_valid_manifest_stubs: calls setup(), reads each stub, confirms toml::from_str succeeds and yields an empty TOML table (all lines are comments).

Add setup_does_not_overwrite_edited_manifest_stub: calls setup(), writes user content to coder.toml, calls setup() again, asserts coder.toml is unchanged and coder.toml.init exists.

### Open questions


### Amendment requests


### Code review


### Merge notes

merge conflict — resolve manually and push: 

## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-25T00:46Z | — | new | philippepascal |
| 2026-05-25T01:20Z | new | groomed | philippepascal |
| 2026-05-25T01:26Z | groomed | in_design | philippepascal |
| 2026-05-25T01:29Z | in_design | specd | claude |
| 2026-05-25T01:43Z | specd | ready | philippepascal |
| 2026-05-25T02:03Z | ready | in_progress | philippepascal |
| 2026-05-25T02:10Z | in_progress | implemented | claude |
| 2026-05-25T02:10Z | implemented | merge_failed | claude |
| 2026-05-25T06:56Z | merge_failed | closed | philippepascal(apm-sync) |
