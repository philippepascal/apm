+++
id = "9f4869d6"
title = "refactor: move review transition and body manipulation logic into apm-core"
state = "in_design"
priority = 0
effort = 3
risk = 2
author = "claude-0330-0245-main"
agent = "85310"
branch = "ticket/9f4869d6-refactor-move-review-transition-and-body"
created_at = "2026-03-30T14:27:50.402284Z"
updated_at = "2026-03-30T16:39:13.841699Z"
+++

## Spec

### Problem

review.rs (321 lines) mixes editor orchestration — a CLI concern — with document-manipulation logic that belongs in apm-core:

- split_body: splits a ticket body into editable spec and preserved history
- extract_spec: strips the editor-header from a saved temp file to recover the spec text
- manual_transitions (aka available_transitions): reads the config to determine which transitions a supervisor can trigger manually (filters out event: auto-triggers)
- normalise_amendment_checkboxes: rewrites plain - bullets in ### Amendment requests to - [ ] checkboxes when a ticket is transitioning to ammend state

Currently all four live in apm/src/cmd/review.rs alongside editor temp-file management, the $VISUAL/$EDITOR/vi invocation, and the interactive stdin prompt. This coupling means apm-serve — which will let a supervisor approve or request amendments via a web UI without a local editor — cannot reuse the logic without depending on the CLI crate.

Moving the document-manipulation functions into apm_core::review gives apm-serve (and tests) a stable, editor-free API surface. The CLI keeps open_editor, build_header, and prompt_transition; it calls into apm_core::review for everything else.

### Acceptance criteria


### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:35Z | new | in_design | philippepascal |