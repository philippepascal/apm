+++
id = "f02d8ef3"
title = "refactor: move ticket creation logic from new.rs into apm-core"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "claude-0330-0245-main"
agent = "86899"
branch = "ticket/f02d8ef3-refactor-move-ticket-creation-logic-from"
created_at = "2026-03-30T14:27:32.493841Z"
updated_at = "2026-03-30T18:09:04.101470Z"
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

- [x] `apm_core::ticket::create()` is callable with: `root`, `config`, `title`, `author`, `context`, `context_section`, `aggressive`
- [x] The returned `Ticket` has `id`, `title`, `state = "new"`, `branch`, `created_at`, and `author` set correctly
- [x] `create()` creates a git branch named `ticket/{id}-{slug}` and commits the ticket file to it
- [x] When `aggressive = true` and a remote exists, the branch is pushed after the commit (push failure is non-fatal)
- [x] When `aggressive = false`, no push is attempted
- [x] When `context` is `None`, the body uses empty section placeholders
- [x] When `context` is `Some`, the text is injected into the correct section (resolved via `context_section`, then workflow config, then defaulting to "Problem")
- [x] When `config.ticket.sections` is non-empty, the body uses those custom sections instead of the default four
- [x] `apm new` still works end-to-end after the refactor (same observable output)
- [x] `cargo test --workspace` passes after the refactor

### Out of scope

- Editor invocation (`$VISUAL` / `$EDITOR` / `vi`) — stays in CLI `new.rs`
- `side_note` / `side_tickets` guard — stays in CLI `new.rs` (not a creation concern)
- Any new `apm-serve` code or web UI
- Behavior changes: `apm new` must produce identical output before and after
- Adding new flags or options to `apm new`

### Approach

**1. Add `create()` to `apm-core/src/ticket.rs`**

```rust
pub fn create(
    root: &Path,
    config: &Config,
    title: String,
    author: String,
    context: Option<String>,
    context_section: Option<String>,
    aggressive: bool,
) -> Result<Ticket>
```

Move the following logic verbatim from `new.rs` into this function:
- `id = git::gen_hex_id()`
- `slug = slugify(&title)`
- `branch = format!("ticket/{id}-{slug}")`
- Frontmatter construction (`Frontmatter { ... }`)
- `body_template` generation (default four sections or custom `config.ticket.sections`)
- Context injection (resolve section → inject into template)
- `Ticket { frontmatter, body, path }` construction and `serialize()`
- `git::commit_to_branch(root, &branch, &rel_path, &content, ...)`
- `if aggressive { git::push_branch(...) }`

Return the constructed `Ticket` (before the push, since push is non-fatal).

**2. Rewrite `apm/src/cmd/new.rs`**

Replace the ~100-line body with ~30 lines:
- Load config
- Check `side_note` guard (`config.agents.side_tickets`)
- Resolve `aggressive = config.sync.aggressive && !no_aggressive`
- Resolve `author` from `APM_AGENT_NAME` env var
- Call `apm_core::ticket::create(root, &config, title, author, context, context_section, aggressive)?`
- Print `Created ticket {id}: {filename} (branch: {branch})`
- Optionally call `open_editor(...)` unchanged

**3. Add a test in `apm-core/tests/`** (or inline in `ticket.rs`)

Integration test using a temp git repo:
- Call `create()` with a title and verify the returned `Ticket` has the expected state/branch
- Verify the branch exists in the repo (`git branch --list ticket/*`)
- Verify `context` injection: call with `context = Some("the context")` and confirm the Problem section contains it

**Order of steps**: Write the core function first, update `new.rs` second, add tests third, run `cargo test --workspace`.

**Constraints**:
- Do not change `open_editor` in `new.rs` — it is CLI-only and correct as-is
- `create()` must not depend on `APM_AGENT_NAME` — the caller passes `author` explicitly
- No new public types needed; return the existing `Ticket` struct

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T14:27Z | — | new | claude-0330-0245-main |
| 2026-03-30T16:31Z | new | in_design | philippepascal |
| 2026-03-30T16:34Z | in_design | specd | claude-0330-1635-spec1 |
| 2026-03-30T16:57Z | specd | ready | philippepascal |
| 2026-03-30T17:20Z | ready | in_progress | philippepascal |
| 2026-03-30T17:24Z | in_progress | implemented | claude-0330-1800-w1f0 |
| 2026-03-30T18:04Z | implemented | accepted | philippepascal |
| 2026-03-30T18:09Z | accepted | closed | apm-sync |