+++
id = "32a9a8b5"
title = "refactor: move sync candidate detection from sync.rs into apm-core"
state = "in_design"
priority = 0
effort = 3
risk = 0
author = "claude-0330-0245-main"
agent = "claude-0330-1640-spec1"
branch = "ticket/32a9a8b5-refactor-move-sync-candidate-detection-f"
created_at = "2026-03-30T14:27:39.762926Z"
updated_at = "2026-03-30T16:39:00.167183Z"
+++

## Spec

### Problem

The sync candidate detection logic in `apm/src/cmd/sync.rs` is tightly coupled to the CLI despite being pure domain logic. Two responsibilities are currently mixed in 172 lines:

1. **Candidate detection** — deciding which tickets are ready to accept (implemented tickets on merged branches) and which are ready to close (accepted tickets, or implemented tickets whose branch no longer exists). This is algorithmic logic with no user-facing I/O.
2. **User prompting** — the interactive `[y/N]` prompts and the orchestration of fetch/push and state transitions. This belongs in the CLI.

Because detection lives in the CLI binary, any future server component (`apm-serve`) that wants to preview a sync — "these N tickets would be accepted, these M would be closed" — cannot do so without shelling out to `apm sync`. Moving detection into `apm-core` makes it available to any caller.

The target shape is `apm_core::sync::detect(root, config)` returning a structured `Candidates` value, and `apm_core::sync::apply(root, config, candidates, author)` executing the state transitions. The CLI retains only I/O: fetch, push, interactive prompts, and calling `detect`/`apply`.

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
| 2026-03-30T16:34Z | new | in_design | philippepascal |
| 2026-03-30T16:37Z | 65590 | claude-0330-1640-spec1 | handoff |