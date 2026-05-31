+++
id = "4d20ba2f"
title = "Make [workers].default mandatory; remove built-in coder fallback"
state = "groomed"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4d20ba2f-make-workers-default-mandatory-remove-bu"
created_at = "2026-05-31T02:58:15.922691Z"
updated_at = "2026-05-31T07:04:43.122908Z"
epic = "9c3c4c20"
target_branch = "epic/9c3c4c20-workflow-schema-cleanup-state-level-work"
+++

## Spec

### Problem

STEP 7 of the incremental workflow schema cleanup. Independent of schema changes.

PROBLEM: apm-core/src/start.rs has multiple hardcoded fallbacks of the form .unwrap_or('claude/coder') in resolution sites. This embeds 'claude' and 'coder' literally in code, violating the project rule that role and agent names are configuration. The fallback also masks misconfigurations: a project that forgets to set workers.default still dispatches via the fallback, hiding the missing config.

DESIGN: make config.workers.default mandatory.

SCOPE:

1. apm-core/src/config.rs::WorkersConfig:
   - Mark the default field as required (remove any #[serde(default)] making it optional).
   - Change the type from Option<String> to String if it was Option, or ensure deserialization fails when the key is missing.

2. apm-core/src/start.rs resolution sites (run, run_next, spawn_next_worker, resolve_for_diagnostic):
   - Drop the .unwrap_or('claude/coder') hardcoded fallback. The cascade is now state.worker_profile → config.workers.default (where the latter is guaranteed to be set).
   - Drop the include_str! constants for default role files if they are tied to the hardcoded cascade (DEFAULT_CODER_DEFAULT, etc.). Audit whether they are still needed for any non-fallback purpose; if not, delete.

3. apm-core/src/init.rs:
   - Ensure apm init writes a [workers] block with default = 'claude/coder' in the scaffolded config.toml. (This must already happen but verify.)
   - The user's migration path: the new .apm/config.toml.init has the right default. They copy it over their existing config.toml if they did not have one, or add the line manually.

4. apm-core/src/validate.rs:
   - Add a check that config.workers.default is set and non-empty.
   - Error message names config.toml and explains how to set the field.

5. Update apm-core/src/default/config.toml.init if it exists, to include the [workers] section by default.

6. Update unit tests that constructed a Config without setting workers.default. They now need to set it explicitly.

OUT OF SCOPE:
- Schema changes (covered by earlier tickets).
- Worker command list (separate ticket).
- Help text (separate ticket).

TESTS:
- A config.toml with no [workers] section fails validate with a clear message.
- A config.toml with [workers] default = '' (empty string) fails validate.
- A config.toml with default = 'claude/coder' passes validate; dispatch resolves accordingly.
- A search for 'claude/coder' literal in apm-core/src/ (excluding init scaffolds and test fixtures and default workflow content) returns nothing.

REFERENCES:
- apm-core/src/config.rs (WorkersConfig)
- apm-core/src/start.rs (resolution sites; the .unwrap_or('claude/coder') calls)
- apm-core/src/init.rs (scaffold writing)
- apm-core/src/validate.rs
- apm-core/src/default/config.toml.init if present

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
| 2026-05-31T02:58Z | — | new | philippepascal |
| 2026-05-31T07:04Z | new | groomed | philippepascal |
