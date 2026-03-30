+++
id = "f02d8ef3"
title = "refactor: move ticket creation logic from new.rs into apm-core"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "claude-0330-0245-main"
agent = "3975"
branch = "ticket/f02d8ef3-refactor-move-ticket-creation-logic-from"
created_at = "2026-03-30T14:27:32.493841Z"
updated_at = "2026-03-30T16:31:31.237281Z"
+++

## Spec

### Problem

\`new.rs\` contains ~120 lines of ticket-creation business logic that belongs in \`apm-core\`:

- Hex ID generation (delegated to \`git::gen_hex_id()\`, already in core)
- Slug generation (delegated to \`slugify()\`, already in core)
- Frontmatter construction
- Body template generation with section placeholders (default or custom from \`config.ticket.sections\`)
- Context injection into a spec section (\`--context\` flag)
- Git branch creation and initial commit (via \`git::commit_to_branch\`)
- Aggressive-push logic (via \`git::push_branch\`)

Only the editor invocation (\`\$VISUAL\` → \`\$EDITOR\` → \`vi\`) and the \`side_note\` guard are CLI concerns.

\`apm-serve\` will need to create tickets from its web UI. Without this refactor it must shell out to \`apm new\` and cannot receive a structured response (the new ticket ID and branch).

Target state: \`apm_core::ticket::create()\` encapsulates all creation logic and returns the new \`Ticket\`. \`new.rs\` becomes ~30 lines: load config, check the side-note guard, call \`create()\`, print output, and optionally open an editor.

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