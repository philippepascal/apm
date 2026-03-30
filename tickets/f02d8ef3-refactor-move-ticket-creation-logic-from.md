+++
id = "f02d8ef3"
title = "refactor: move ticket creation logic from new.rs into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "philippepascal"
branch = "ticket/f02d8ef3-refactor-move-ticket-creation-logic-from"
created_at = "2026-03-30T14:27:32.493841Z"
updated_at = "2026-03-30T16:31:31.237281Z"
+++

## Spec

### Problem

`new.rs` contains 158 lines of ticket creation logic that belongs in `apm-core`:

- Hex ID generation (timestamp + random bytes → sha256 → 8-char hex)
- Slug generation from title
- Ticket frontmatter construction
- Template body generation with section placeholders
- Context injection into spec body (`--context` flag)
- Git branch creation and initial commit
- Aggressive push logic

Only the editor invocation (`$VISUAL` → `$EDITOR` → `vi`) belongs in the CLI.

`apm-serve` will need to create tickets from the web UI. Without this refactor
it must shell out to `apm new` and cannot get a structured response (the new
ticket ID and branch) back from the operation.

Target: `apm_core::ticket::create()` returning the new ticket. CLI `new.rs`
calls it and optionally opens an editor. ~30 lines in CLI.

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
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:31Z | new | in_design | philippepascal |
