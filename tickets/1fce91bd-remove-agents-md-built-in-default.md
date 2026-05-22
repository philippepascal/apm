+++
id = "1fce91bd"
title = "Remove agents.md built-in default"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1fce91bd-remove-agents-md-built-in-default"
created_at = "2026-05-22T23:22:54.150045Z"
updated_at = "2026-05-22T23:22:54.150045Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
+++

## Spec

### Problem

After T2 creates apm.project.md and apm.main-agent.md, and T4/T5 rewrite the role files, agents.md (apm-core/src/default/agents/default/agents.md) is no longer needed — all its content has been redistributed. Delete it. Changes: (1) delete apm-core/src/default/agents/default/agents.md; (2) remove fn default_agents_md() in apm-core/src/init.rs:392; (3) remove the write_default call for agents.md in setup() at init.rs:142; (4) ensure no remaining include_str! references it. The init.rs test setup_creates_expected_files currently asserts .apm/agents/default/agents.md exists — this assertion must be removed (or changed to assert it does NOT exist after init).

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
| 2026-05-22T23:22Z | — | new | philippepascal |
