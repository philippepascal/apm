+++
id = "753d9ba5"
title = "Document parallel tool-batching hazard in worker shell discipline"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/753d9ba5-document-parallel-tool-batching-hazard-i"
created_at = "2026-05-29T18:47:28.479025Z"
updated_at = "2026-05-29T19:04:42.387460Z"
+++

## Spec

### Problem

The shell discipline in `SHELL_DISCIPLINE_BODY` (`apm-core/src/instructions.rs`) guards against compound shell syntax hazards (&&, $(), &, heredocs) but says nothing about parallel tool-call batching — a model emitting multiple separate tool_use blocks in a single turn, which Claude Code executes concurrently.

These are distinct failure modes. A worker can be fully compliant with every existing shell rule and still trigger the batch-cancellation bug: when running headless (`--print` mode), if any one tool call in a parallel batch requires approval, Claude Code cancels the entire batch — including calls that are individually allowed. The error appears as `<tool_use_error>Cancelled: parallel tool call Bash(...) errored</tool_use_error>` on every sibling call. Bootstrap commands such as `apm instructions` are the highest-risk emitters because they typically appear at session start alongside other reads, and if not yet allow-listed they poison every sibling call in the batch. This guidance gap was confirmed by incident syn-25673007.

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
| 2026-05-29T18:47Z | — | new | philippepascal |
| 2026-05-29T18:49Z | new | groomed | philippepascal |
| 2026-05-29T19:04Z | groomed | in_design | philippepascal |