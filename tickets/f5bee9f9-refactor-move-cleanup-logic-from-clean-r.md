+++
id = "f5bee9f9"
title = "refactor: move cleanup logic from clean.rs into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "7492"
branch = "ticket/f5bee9f9-refactor-move-cleanup-logic-from-clean-r"
created_at = "2026-03-30T14:27:36.851282Z"
updated_at = "2026-03-30T16:31:35.753224Z"
+++

## Spec

### Problem

clean.rs (171 lines) contains all cleanup detection and orchestration logic as a single CLI command. This logic — terminal state resolution, merged branch detection via git branch --merged, ancestor checking via git merge-base --is-ancestor, ticket state cross-checking between the ticket branch and main, remote tip agreement checking, worktree dirty-checking, and local branch existence checking — belongs in apm-core.

These are pure data checks on git state, not CLI presentation concerns. Embedding them in the CLI command prevents apm-serve from reusing them: the server will need to show a 'ready to clean' list and trigger cleanup without shelling out to the apm binary.

The target is apm_core::clean::candidates() returning a structured list of branches safe to remove (with reasons), and apm_core::clean::remove() performing the actual deletion. The CLI becomes thin: call candidates(), format output, prompt, then call remove().

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
| 2026-03-30T16:31Z | new | in_design | philippepascal |