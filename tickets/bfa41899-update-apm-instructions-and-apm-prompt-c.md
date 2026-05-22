+++
id = "bfa41899"
title = "Update apm instructions and apm prompt CLI help for new model"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bfa41899-update-apm-instructions-and-apm-prompt-c"
created_at = "2026-05-22T23:23:41.917063Z"
updated_at = "2026-05-22T23:23:41.917063Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
+++

## Spec

### Problem

Two CLI help texts need updating after the redesign. First: apm instructions (apm/src/cmd/instructions.rs) — the PREAMBLE and render() function describe it as a command list; after T1 it emits full APM system knowledge. Update the about string in main.rs and the preamble/intro in instructions.rs to reflect what it emits: state machine, ticket format, shell discipline, session identity, command reference. Second: apm prompt (apm/src/main.rs Prompt subcommand) — the help text and examples describe the old cascade model; after T3 it composes three layers. Update the about string and examples to document: (a) layer 1 = apm instructions (dynamic), (b) layer 2 = apm.project.md, (c) layer 3 = role file. Also update apm prompt --explain output labels to match the new layer names (currently labels say 'per-agent file', 'transition.instructions', etc.).

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
| 2026-05-22T23:23Z | — | new | philippepascal |
