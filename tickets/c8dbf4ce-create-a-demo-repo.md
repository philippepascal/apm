+++
id = "c8dbf4ce"
title = "create a demo repo"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/c8dbf4ce-create-a-demo-repo"
created_at = "2026-04-07T17:01:04.559759Z"
updated_at = "2026-04-07T17:43:43.554278Z"
+++

## Spec

### Problem

APM has no standalone public demo that a new user can clone and explore without first building a project from scratch. The only way to currently "kick the tires" is to run `apm init` on a blank repo (no pre-existing tickets, no context) or wade through the actual APM source tickets (complex, hundreds of entries, opaque to outsiders).

A purpose-built demo repo solves this by giving new users a realistic, self-contained project they can clone and immediately explore. It provides a believable software project with a representative ticket backlog, so every APM command has something meaningful to act on.

The demo must cover the full feature surface: multiple ticket states, epics, cross-ticket dependencies, the `apm-server` web UI, and the README-driven onboarding flow. Without it, the "getting started" story for APM is fragile and requires significant upfront investment from the user.

### Acceptance criteria

- [ ] A public GitHub repository named `apm-demo` exists and is cloneable without authentication
- [ ] The repo contains a Rust CLI project that compiles with `cargo build` without errors
- [ ] Running the compiled binary (e.g. `./jot list`) produces output without panicking
- [ ] The repo contains a `.apm/config.toml` with project name, default branch, and merge strategy configured
- [ ] `apm list` run from the cloned repo shows tickets across at least 8 distinct states
- [ ] At least one epic exists and `apm epic list` shows it
- [ ] At least two tickets have `depends_on` set referencing other tickets in the repo
- [ ] At least one ticket is assigned to the epic (has `epic` field set)
- [ ] `apm show <id>` on a `closed` ticket shows a fully-populated spec (all four sections filled)
- [ ] `apm show <id>` on a ticket in `ammend` state shows a `### Amendment requests` section with at least one unchecked checkbox
- [ ] `apm show <id>` on a ticket in `question` state shows a `### Open questions` section with a pending question
- [ ] `apm next` returns a ticket (the highest-priority actionable one)
- [ ] The README contains a "Getting started" section that covers: cloning, verifying binaries, `apm list`, `apm show`, `apm next`, `apm-server`
- [ ] The README explains the fictional project context so the ticket backlog makes narrative sense
- [ ] All ticket states from the default workflow appear at least once across the ticket set: `new`, `groomed`, `in_design`, `specd`, `question`, `ammend`, `ready`, `in_progress`, `blocked`, `implemented`, `closed`

### Out of scope

- Building or publishing a binary release for the demo CLI
- CI/CD configuration (GitHub Actions, etc.) for the demo repo
- Automated testing of the demo repo itself
- apm-server deployment or hosting of the web UI
- Using `apm register` / `apm sessions` / `apm revoke` (server auth commands) — those require a running server instance and are mentioned in the README as a next step, not demonstrated
- Creating tickets for every single possible state combination — a representative subset is sufficient
- Keeping the demo repo in sync with future APM feature changes (out-of-scope for this ticket; a separate maintenance process is needed)
- The Rust CLI being a genuinely useful piece of software — it only needs to be plausible and compilable

### Approach

## 1. Fictional project: `jot`

The demo Rust CLI is called **`jot`** — a minimal command-line notes tool. This is a thematically apt choice (APM manages tasks; `jot` manages notes), simple enough to understand in 30 seconds, and gives natural ticket material ("add tagging", "search", "delete", "export").

**Working commands (already implemented in the frozen state):**
- `jot add "<text>"` — appends a note to `~/.jot/notes.txt`
- `jot list` — prints all notes with indices

**Stubbed / in-progress commands (exist in code but not finished):**
- `jot delete <n>` — deletes note by index (panics or prints "not yet implemented")
- `jot search <query>` — returns `unimplemented!()`

The Cargo project lives at the repo root. `src/main.rs` is ~80 lines using only `std`.

---

## 2. APM initialisation

Run `apm init` in the repo (or manually create the files). Config:

```toml
# .apm/config.toml
[project]
name = "jot"
description = "A minimal CLI notes tool"
default_branch = "main"

[workflow]
completion = "pr_or_epic_merge"
```

Keep the default workflow (11 states). No custom `workflow.toml` override needed — the defaults demonstrate all states already.

---

## 3. Epic

Create one epic: **"Search feature"** (`apm epic new "Search feature"`).

Tickets that belong to this epic set `epic = "<search-epic-id>"` and `target_branch = "epic/<search-epic-id>-search-feature"`.

---

## 4. Ticket set (12–14 tickets)

Design the backlog to cover every default workflow state at least once, with realistic narrative:

| # | Title | State | Epic | depends_on | Notes |
|---|-------|-------|------|-----------|-------|
| 1 | Initial CLI scaffold | `closed` | — | — | Full spec, all AC checked |
| 2 | Add note to file (`jot add`) | `closed` | — | 1 | Full spec |
| 3 | List notes command (`jot list`) | `implemented` | — | 2 | PR open, not merged |
| 4 | Delete note command (`jot delete`) | `in_progress` | — | 3 | Spec complete, being coded |
| 5 | Add full-text search | `in_progress` | search-epic | 3 | Being implemented |
| 6 | Search result highlighting | `ready` | search-epic | 5 | Spec approved, not started |
| 7 | Export notes to markdown | `specd` | — | — | Spec written, awaiting review |
| 8 | Note tagging support | `in_design` | — | — | Spec being written |
| 9 | Configuration file (`~/.jot/config.toml`) | `groomed` | — | — | Groomed, not yet in design |
| 10 | Pagination for long note lists | `new` | — | — | Just filed |
| 11 | Interactive TUI mode | `question` | — | — | Has open question about TUI framework choice |
| 12 | Fuzzy search fallback | `ammend` | search-epic | 5 | Spec needs revision |
| 13 | Fix list command index off-by-one | `blocked` | — | — | Blocked on design decision re: 0- vs 1-indexed |
| 14 | Add `--count` flag to `jot list` | `ready` | — | 3 | Low-priority polish |

**Dependency chain:** 1 → 2 → 3 → 4 and 3 → 5 → 6 and 5 → 12 give a multi-level dependency graph to demo `depends_on`.

**Priority spread:** tickets 4 and 5 get priority 5; ticket 1-3 (closed/implemented) can be 0; the rest vary 1–3.

---

## 5. Ticket content quality

- Tickets 1–3 (`closed`/`implemented`): fully filled spec with all AC checked `[x]`
- Ticket 11 (`question`): has `### Open questions` with an unanswered Q
- Ticket 12 (`ammend`): has `### Amendment requests` with one unchecked checkbox
- Ticket 13 (`blocked`): problem and approach filled; AC present; `### Open questions` explains what the supervisor must decide
- All other tickets: at minimum `### Problem` and partial `### Acceptance criteria`

---

## 6. README structure

```
# jot — a minimal notes CLI

> Demo repository for [APM](https://github.com/philippepascal/apm)

## About this repo
<1-paragraph description of jot and why it's frozen mid-development>

## Prerequisites
- `apm` and `apm-server` installed (see APM installation guide)
- Rust toolchain (for building jot)

## Build & run jot
cargo build
./target/debug/jot list

## Explore with APM

### See all tickets
apm list

### Inspect a ticket
apm show <id>

### Find the next actionable ticket
apm next

### Transition a ticket (example: start working)
apm state <id> in_progress

### Browse epics
apm epic list
apm epic show <epic-id>

### Launch the web UI
apm-server
# then open http://localhost:3000

## Next steps
- Run `apm help` to see all commands
- Try `apm work` to auto-dispatch agents
- Register with apm-server for team features: `apm register`
```

---

## 7. Repository setup

1. Create new public GitHub repo `apm-demo` under the `philippepascal` account (or `apm-project` org if preferred)
2. Init git, add Cargo project files (`Cargo.toml`, `src/main.rs`)
3. Run `apm init` (or manually place `.apm/` files) — answer prompts with project name "jot"
4. Create tickets using `apm new` with appropriate metadata, then `apm spec` to fill sections, then `apm state` to advance each to its target state
5. For `closed` / `implemented` tickets, use `apm state <id> --force closed` since there is no actual branch to merge
6. Write README.md
7. Push to GitHub

**Note on git history:** the repo should have a plausible commit history (a few commits). The implementer may squash into a single "initial commit" if preferred — realism of history is not an AC.

**Note on worktrees:** the demo repo does not need actual worktrees — the `branch` field on tickets is sufficient for demonstration. Do not run `apm start` for in-progress tickets (it would try to create worktrees); instead set `branch` manually with `apm set <id> branch "ticket/..."`.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T17:01Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:43Z | groomed | in_design | philippepascal |