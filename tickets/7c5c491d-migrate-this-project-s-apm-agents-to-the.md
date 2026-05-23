+++
id = "7c5c491d"
title = "Migrate this project's .apm/agents/ to the new structure"
state = "in_progress"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7c5c491d-migrate-this-project-s-apm-agents-to-the"
created_at = "2026-05-22T23:23:29.954873Z"
updated_at = "2026-05-23T04:28:17.244676Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["34ad9126", "78eeb755", "02bbcc2f", "1fce91bd", "7ef960f2"]
+++

## Spec

### Problem

The APM project's own `.apm/agents/` directory was created from the original monolithic `agents.md` built-in default. With the prompt management redesign (epic ab6e5db7), that monolith is being split into three composed layers: dynamic APM system knowledge from `apm instructions` (T1/4bee5771), project context from `apm.project.md`, and role-specific instructions from role files. The sibling tickets (T2–T8) update the built-ins and the `apm init` scaffold, but none of them update this project's own `.apm/agents/` directory.

Until this ticket is implemented, the project's agents receive stale instructions — specifically: `agents.md` still referenced as the single instructions file, role files still contain shell-discipline and session-identity content that will be covered by `apm instructions`, `CLAUDE.md` imports only `agents.md`, and `.apm/config.toml` still uses `instructions =` rather than the renamed `project =` key that T3 introduces.

The desired end state: `agents.md` deleted; two new files (`apm.project.md`, `apm.main-agent.md`) created with project-specific content; `apm.spec-writer.md` and `apm.worker.md` updated to match the rewritten built-ins; `claude/apm.spec-writer.md` and `claude/apm.worker.md` deleted (both are stale overrides that should fall through to the updated defaults); `CLAUDE.md` updated to import the two new files; and `.apm/config.toml` `[agents]` `instructions` key renamed to `project`.

### Acceptance criteria

- [x] `.apm/agents/default/agents.md` does not exist in the repo
- [x] `.apm/agents/default/apm.project.md` exists and contains APM-specific project context (crate structure, module responsibilities)
- [x] `.apm/agents/default/apm.main-agent.md` exists and matches the built-in default created by edb0cf35
- [x] `.apm/agents/default/apm.spec-writer.md` matches the rewritten built-in from 34ad9126 (no runtime notice, no permitted-commands list, no shell-discipline block in § How to save spec sections, amendment step 6 references auto-commit not a manual git block)
- [ ] `.apm/agents/default/apm.worker.md` matches the rewritten built-in from 78eeb755 (no `agents.md` back-reference, no `## Shell discipline` section, has `## Ticket file discipline`)
- [ ] `.apm/agents/claude/apm.worker.md` does not exist in the repo
- [ ] `.apm/agents/claude/apm.spec-writer.md` does not exist in the repo
- [ ] `CLAUDE.md` contains `@.apm/agents/default/apm.project.md` and `@.apm/agents/default/apm.main-agent.md`
- [ ] `CLAUDE.md` does not contain `@.apm/agents/default/agents.md`
- [ ] `.apm/config.toml` `[agents]` section has `project = ".apm/agents/default/apm.project.md"` and does not contain an `instructions =` key

### Out of scope

- Rewriting the built-in defaults in `apm-core/src/default/agents/` — covered by 34ad9126, 78eeb755, edb0cf35
- Removing the `claude/` overrides from the built-in defaults in `apm-core/` — covered by 02bbcc2f and 34ad9126
- Updating `apm-core/src/init.rs` or `apm-core/src/start.rs` — covered by 7ef960f2 and d8e2fa0e
- Updating `apm instructions` CLI help text — covered by bfa41899
- The `wrapper.sh` file — not affected by the redesign
- The `style.md` file — not affected by the redesign
- Running `cargo test --workspace` — no Rust source changes in this ticket

### Approach

All changes are file operations inside the worktree. There are no Rust source changes and no `cargo test` run. Work in this order:

**1. Create `.apm/agents/default/apm.project.md`**

This is the project-context layer — it replaces the `## Repo structure` placeholder in the old `agents.md`. Copy `apm-core/src/default/agents/default/apm.project.md` (created by edb0cf35) as the starting point, then replace its placeholder body with real APM project content:

- **What APM is:** a Git-native, agent-first project management tool; each ticket lives on its own `ticket/<id>-<slug>` branch as a Markdown file with TOML frontmatter.
- **Rust workspace crates:**
  - `apm-core` — core library: ticket parsing, state machine, `build_system_prompt`, `apm init` scaffolding, instructions generation
  - `apm` — CLI binary; depends on `apm-core`; subcommands live in `apm/src/cmd/`
  - `apm-server` — web UI; depends on `apm-core`
- **State machine:** transitions defined in `apm.toml` under `[[workflow.states]]`; the `apm state` command enforces valid transitions and auto-commits the History table row
- **Key conventions:** unit tests inline in `apm-core/src/`, integration tests in `apm/tests/integration.rs` using temp git repos; `cargo test --workspace` is the required test command

**2. Create `.apm/agents/default/apm.main-agent.md`**

Copy `apm-core/src/default/agents/default/apm.main-agent.md` (created by edb0cf35) verbatim. No project-specific additions are needed — the project-specific main-agent rules live in CLAUDE.md directly.

**3. Update `.apm/agents/default/apm.spec-writer.md`**

Apply the removals specified in 34ad9126 to the current 273-line project file:

- Remove the two-sentence runtime notice at the top of `## Scope limits` (lines 11–13: "This session was started…" and "If you see skill availability information…")
- Remove the **Permitted `apm` commands** bullet list (five items, lines 15–20)
- Remove the opening prose and code block of `## How to save spec sections` (lines 36–48: the `# Short content` / `# Long content` block). Keep only the single line "Do NOT write the ticket markdown file directly. Always use `apm spec`." Keep the `### Never hand-edit the History table` and `### Filename is fixed` subsections.
- Fix amendment step 6: delete the `FILE=$(ls ...)` / `git -C` / `git commit` block (lines 220–225). Replace with a note: "`apm spec` calls auto-commit to the ticket branch — no manual git step is needed."

Do not touch any other section.

**4. Update `.apm/agents/default/apm.worker.md`**

Apply the changes specified in 78eeb755 to the current 205-line project file:

- Replace the second paragraph (lines 5–8: "Read `.apm/agents/default/agents.md` for startup…") with: "Shell discipline, session identity, and startup sequence are covered by `apm instructions` — this file covers the implementation phase only."
- Delete the `## Shell discipline` section (lines 133–181) in its entirety, including its heading.
- Add a `## Ticket file discipline` section immediately after `## Side tickets`, copying the two subsections verbatim from the updated `apm.spec-writer.md`: `### Never hand-edit the History table` and `### Filename is fixed — never rename the ticket file`.

**5. Delete stale claude/ overrides**

Delete both files:
- `.apm/agents/claude/apm.worker.md` — identical to the old default; stale after step 4
- `.apm/agents/claude/apm.spec-writer.md` — older version missing History/Filename sections; stale after step 3

After deletion, `build_system_prompt` for claude agents falls through to the updated defaults.

**6. Update `CLAUDE.md`**

Replace the line `@.apm/agents/default/agents.md` with two lines:
```
@.apm/agents/default/apm.project.md
@.apm/agents/default/apm.main-agent.md
```

Keep `@.apm/agents/default/style.md` and all other content unchanged.

**7. Update `.apm/config.toml`**

In the `[agents]` section, rename:
```toml
instructions = ".apm/agents/default/agents.md"
```
to:
```toml
project = ".apm/agents/default/apm.project.md"
```

**8. Delete `.apm/agents/default/agents.md`**

Remove the file from the repo with `git -C <wt> rm .apm/agents/default/agents.md`.

**9. Verify**

Run `apm prompt --agent claude --role worker` and confirm the assembled prompt contains:
- Project context from `apm.project.md`
- Worker instructions from the updated `apm.worker.md` (no shell discipline section, has ticket file discipline)
- No references to `agents.md`

Commit all changes to the ticket branch in the worktree.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:23Z | — | new | philippepascal |
| 2026-05-22T23:51Z | new | groomed | philippepascal |
| 2026-05-23T00:30Z | groomed | in_design | philippepascal |
| 2026-05-23T00:34Z | in_design | specd | claude-0522-1445-b3f7 |
| 2026-05-23T02:58Z | specd | ready | philippepascal |
| 2026-05-23T04:28Z | ready | in_progress | philippepascal |