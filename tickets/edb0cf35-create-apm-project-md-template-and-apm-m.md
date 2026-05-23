+++
id = "edb0cf35"
title = "Create apm.project.md template and apm.main-agent.md built-in defaults"
state = "specd"
priority = 0
effort = 3
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/edb0cf35-create-apm-project-md-template-and-apm-m"
created_at = "2026-05-22T23:22:36.259605Z"
updated_at = "2026-05-23T00:13:50.718701Z"
epic = "ab6e5db7"
target_branch = "epic/ab6e5db7-prompt-management-redesign"
depends_on = ["4bee5771"]
+++

## Spec

### Problem

The APM prompt redesign (epic ab6e5db7) splits the monolithic `agents.md` into three composed layers: (1) dynamic APM system knowledge from `apm instructions` (T1/4bee5771), (2) project-specific context from `apm.project.md`, and (3) role-specific instructions from a role file. Two built-in defaults for layers 2 and 3 are missing from apm-core: `apm.project.md` and `apm.main-agent.md`.

`apm.project.md` is a template the user fills in after `apm init`. Without a shipped template, users have no guidance on what to write — they see only the legacy `agents.md` with its `_Fill in your project's structure here._` placeholder. The new file should have named sections so project-specific documentation is easy to populate and maintain independently of APM version updates.

`apm.main-agent.md` is the role file for the supervisor companion (the Main Agent). Currently the Main Agent role is described inside `agents.md` alongside the Worker role, the ticket format, shell discipline, and session identity content that T1 will emit dynamically. After the redesign, `apm.main-agent.md` must contain only Main Agent-specific behavior (purpose, off-limits actions, supervisor-only transitions, startup sequence) and must reference `apm instructions` as the source of APM system knowledge rather than duplicating it.

### Acceptance criteria

- [ ] `apm-core/src/default/agents/default/apm.project.md` exists with named section headers and `_fill in_`-style placeholder text covering: project name/description, tech stack, repo structure, module responsibilities, and key technical decisions
- [ ] `apm-core/src/default/agents/default/apm.main-agent.md` exists and covers: purpose, off-limits actions, supervisor-only transitions list, override clause, amendment workflow, and startup sequence
- [ ] `apm.main-agent.md` startup sequence instructs the agent to run `apm instructions` first to obtain current state machine, ticket format, shell discipline, and command reference
- [ ] `apm.main-agent.md` does not inline the state machine, ticket format, shell discipline, or session identity content (those are emitted by `apm instructions`)
- [ ] Both files are accessible via `include_str!` constants in `apm-core/src/start.rs` and `cargo build --workspace` succeeds
- [ ] `resolve_builtin_instructions` in `start.rs` returns the `apm.main-agent.md` content for role `"main-agent"` regardless of agent name
- [ ] `cargo test --workspace` passes with no regressions

### Out of scope

- Writing `apm.project.md` and `apm.main-agent.md` during `apm init` — covered by 7ef960f2
- Adding `@apm.project.md` and `@apm.main-agent.md` includes to CLAUDE.md — covered by 7ef960f2
- Wiring `apm.project.md` as layer 2 in `build_system_prompt` — covered by d8e2fa0e
- Rewriting `apm.worker.md` or `apm.spec-writer.md` — covered by 78eeb755 and 34ad9126
- Removing `agents.md` from init or deleting its built-in — covered by 1fce91bd
- Migrating this project's own `.apm/agents/` directory — covered by 7c5c491d
- Creating per-agent claude-specific overrides for either new file
- Updating config.toml default to add a `project` key — covered by d8e2fa0e and 7ef960f2

### Approach

#### 1. Create `apm-core/src/default/agents/default/apm.project.md`

Template file with `_fill in_`-style placeholders. Sections:

- `# Project Context` — title only
- `## What we are building` — one-paragraph placeholder describing the product
- `## Tech stack` — bullet list: language/runtime, key libraries, database, etc.
- `## Repo structure` — directory tree with one-line descriptions per entry
- `## Module responsibilities` — per-module paragraph (what each crate/package owns)
- `## Key technical decisions` — bullet list of non-obvious architectural choices and their rationale

Keep the file under 50 lines. Every placeholder uses `_Fill in: ..._` phrasing so it is visually distinct from real content.

#### 2. Create `apm-core/src/default/agents/default/apm.main-agent.md`

Role file for the supervisor companion. Sections:

- **Title + preamble** — "You are a project-management companion to the supervisor. Run `apm instructions` at the start of every session to load the current state machine, ticket format, shell discipline, and command reference."
- **What you do** — help the supervisor create tickets (with `--context`), manage epics, review specs, answer codebase questions; run `apm` commands at the supervisor's explicit direction
- **What you do NOT do** — spawn workers, push code unsolicited, run `apm start`, amend published git history, make unauthorized state transitions
- **Supervisor-only transitions** — exact list from current `agents.md` (`new → groomed`, `specd → ready/ammend`, `implemented → ready/ammend/closed`, `blocked → ready`, `apm epic close`); then the override clause ("The supervisor can ask you to perform any supervisor-only transition explicitly...")
- **When asked to amend a ticket** — transition `specd → ammend`, add amendment requests with `apm spec --add-task`, stop; do not pick up the ticket yourself
- **Startup sequence**:
  1. `apm instructions` — loads APM system knowledge for this session
  2. `apm sync` — refresh local cache from all `ticket/*` branches
  3. `apm next --json` — find the highest-priority actionable ticket
  4. `apm list --state in_progress` — check for in-progress tickets to resume

Do **not** inline state machine tables, ticket format, shell discipline rules, or session identity instructions — those are covered by `apm instructions` output (T1/4bee5771).

#### 3. Add `include_str!` constants to `apm-core/src/start.rs`

Insert two new `const` declarations immediately after the existing block at lines 7–16:

```rust
const DEFAULT_MAIN_AGENT_MD: &str = include_str!("default/agents/default/apm.main-agent.md");
const DEFAULT_PROJECT_MD: &str = include_str!("default/agents/default/apm.project.md");
```

`DEFAULT_PROJECT_MD` is declared here so T3 (d8e2fa0e) can reference it from `build_system_prompt` without adding a new `include_str!` call. It is not used in this ticket beyond compilation verification.

#### 4. Update `resolve_builtin_instructions` in `apm-core/src/start.rs`

Add one arm before the `_ => None` catch-all:

```rust
(_, "main-agent") => Some(DEFAULT_MAIN_AGENT_MD),
```

The wildcard agent pattern (`_`) matches any agent name, since there are no per-agent overrides for `main-agent`.

#### 5. Verify

Run `cargo test --workspace`. The two new `const` declarations must compile (exercising the `include_str!` paths); existing tests must pass unchanged.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-22T23:22Z | — | new | philippepascal |
| 2026-05-22T23:50Z | new | groomed | philippepascal |
| 2026-05-23T00:09Z | groomed | in_design | philippepascal |
| 2026-05-23T00:13Z | in_design | specd | claude-0523-0009-d620 |
