+++
id = "753d9ba5"
title = "Document parallel tool-batching hazard in worker shell discipline"
state = "in_progress"
priority = 5
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/753d9ba5-document-parallel-tool-batching-hazard-i"
created_at = "2026-05-29T18:47:28.479025Z"
updated_at = "2026-05-29T19:22:13.486688Z"
+++

## Spec

### Problem

The shell discipline in `SHELL_DISCIPLINE_BODY` (`apm-core/src/instructions.rs`) guards against compound shell syntax hazards (&&, $(), &, heredocs) but says nothing about parallel tool-call batching — a model emitting multiple separate tool_use blocks in a single turn, which Claude Code executes concurrently.

These are distinct failure modes. A worker can be fully compliant with every existing shell rule and still trigger the batch-cancellation bug: when running headless (`--print` mode), if any one tool call in a parallel batch requires approval, Claude Code cancels the entire batch — including calls that are individually allowed. The error appears as `<tool_use_error>Cancelled: parallel tool call Bash(...) errored</tool_use_error>` on every sibling call. Bootstrap commands such as `apm instructions` are the highest-risk emitters because they typically appear at session start alongside other reads, and if not yet allow-listed they poison every sibling call in the batch. This guidance gap was confirmed by incident syn-25673007.

### Acceptance criteria

- [x] `apm instructions` output contains a named block under Shell Discipline that describes the parallel tool-call batching hazard
- [x] The block states that in headless (`--print`) mode a single unapproved call cancels every sibling call in the batch
- [x] The block gives a concrete rule: issue `apm` and bootstrap commands as their own tool call, not batched with other work
- [x] The block includes a wrong/right example showing sequential emission versus parallel batching
- [x] The new text is in `SHELL_DISCIPLINE_BODY` in `apm-core/src/instructions.rs`
- [x] No duplicate guidance is added to any agent template file under `apm-core/src/default/agents/`

### Out of scope

- Changing Claude Code's parallel batch-cancellation behavior — that is not APM's to change
- Adding allow-list entries for any `apm` commands (covered by ticket e54a7adf)
- Fixing `denial.rs` mislabeling of cancellations as denials (covered by ticket e54a7adf)
- Any runtime enforcement — this is documentation only
- Changes to agent template `.md` files; `apm.coder.md` already defers all shell discipline to `apm instructions`

### Approach

Edit `SHELL_DISCIPLINE_BODY` in `apm-core/src/instructions.rs`.

Append a new block after the existing "Off-limits" block (the final block in the string). The wording below is the target; the implementer may adjust phrasing but must preserve the three elements: named hazard, failure mode, concrete rule + example.

```
Do not batch tool calls in parallel in a headless worker:

  Claude Code runs all tool_use blocks emitted in a single turn concurrently.
  In --print (headless) mode, if any one call requires approval, the entire
  batch is cancelled — including calls that were individually allowed.

  apm and bootstrap commands must be their own single tool call:

    # Wrong — if apm instructions requires approval, Read is also cancelled
    [Bash("apm instructions"), Read("some/file")]  <- emitted together

    # Right — sequential, one at a time
    Bash("apm instructions")
    ... wait for result ...
    Read("some/file")
```

No changes to agent template files. `apm.coder.md` already states that shell discipline is covered by `apm instructions`, so the new text propagates to all workers automatically.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-29T18:47Z | — | new | philippepascal |
| 2026-05-29T18:49Z | new | groomed | philippepascal |
| 2026-05-29T19:04Z | groomed | in_design | philippepascal |
| 2026-05-29T19:06Z | in_design | specd | claude |
| 2026-05-29T19:22Z | specd | ready | philippepascal |
| 2026-05-29T19:22Z | ready | in_progress | philippepascal |
