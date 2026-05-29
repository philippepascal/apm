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

GAP found by verifying agent instructions in three places (apm source apm-core/src/instructions.rs SHELL_DISCIPLINE_BODY; apm project .apm/agents/claude/*.md; syn project .apm/agents/claude/*.md): the shell discipline covers COMPOUND shell syntax (&& chains, $() subshells, background &, cd &&, heredocs) but says NOTHING about PARALLEL TOOL-CALL BATCHING — the model emitting several separate tool_use blocks (e.g. multiple Bash calls, or Bash + Read + Glob) in a single turn, which Claude Code runs concurrently. grep for parallel|batch|single operation|simultaneous returns nothing in any of the three locations except the single-line 'Keep each Bash call to a single operation' (which is about compound syntax, not parallel emission).

WHY IT MATTERS (incident syn 25673007): when a worker emits multiple tool calls in parallel and ONE of them requires approval (not in the allow-list) in a headless --print worker, Claude Code CANCELS THE ENTIRE BATCH — every sibling call fails with '<tool_use_error>Cancelled: parallel tool call Bash(...) errored</tool_use_error>', even calls that are individually allowed. So a single un-allow-listed command poisons the whole batch. A worker can be fully shell-discipline-compliant (each command is a single operation) and still hit this, because it is about how tool calls are EMITTED (parallel vs sequential), not shell syntax. This is distinct from, and not covered by, the existing compound-command guidance.

DESIRED: add guidance to the worker shell discipline (SHELL_DISCIPLINE_BODY in apm-core/src/instructions.rs, which is the dynamic 'apm instructions' output all workers receive) that in a headless worker, tool calls should be issued ONE AT A TIME rather than batched in parallel when any call might require approval — and specifically that apm/bootstrap commands should be run as their own single tool call, never parallel-batched with other work, because one approval-required member cancels the whole batch. The spec-writer should decide exact wording and whether to also surface it in the agent md templates (apm-core/src/default/agents/claude/apm.coder.md etc.) vs only in the shared instructions output. Keep it concise; this is a discipline note, not a large doc.

CONTEXT/RELATED: the complementary fixes (allow-listing apm instructions so it does not trigger approval; fixing denial.rs to not mislabel cancellations as denials) are in ticket e54a7adf. This ticket is purely the agent-instruction guidance gap. Out of scope: changing Claude Code's parallel-cancellation behavior (not ours to change); any code beyond the instruction text.

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
