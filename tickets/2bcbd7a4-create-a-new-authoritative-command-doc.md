+++
id = "2bcbd7a4"
title = "create a new authoritative command doc"
state = "in_design"
priority = 0
effort = 5
risk = 0
author = "philippepascal"
branch = "ticket/2bcbd7a4-create-a-new-authoritative-command-doc"
created_at = "2026-04-07T17:06:49.569239Z"
updated_at = "2026-04-07T17:47:09.821352Z"
+++

## Spec

### Problem

APM has a rich CLI with ~28 commands, but there is no single authoritative reference document that covers all of them in depth. Existing help text (`--help`) gives one-line descriptions and flag names, but does not explain the internal mechanics—especially the git operations each command performs and why.

Contributors adding new features and users debugging unexpected behaviour have no place to look beyond the source code. A contributor extending `apm sync` needs to understand which `git` calls it already makes and the order they run in; a power user writing a wrapper script needs to know exactly what `apm start` does to a worktree before they can safely automate around it.

The desired outcome is a single Markdown file committed to the repository that serves as the canonical reference for every command: what the command does at a high level, its full argument and flag surface, and a detailed breakdown of every git operation it performs internally with a note on why each one is needed. The format should be inspired by how popular CLI tools like `git` or `curl` document themselves—structured, scannable, and complete enough that reading it once is sufficient to understand the full behaviour.

### Acceptance criteria

- [ ] A file `docs/commands.md` exists in the repository on the ticket branch
- [ ] Every command exposed by `apm --help` has a dedicated section in the document
- [ ] Each command section includes a one-paragraph high-level description of what the command does
- [ ] Each command section includes a SYNOPSIS block showing the exact invocation syntax with arguments and flags
- [ ] Each command section lists every flag and argument with its type, default (if any), and a one-sentence description
- [ ] Each command section that performs git operations includes a "Git internals" subsection listing each git call and a one-sentence explanation of why it is needed
- [ ] Commands that perform no git operations (e.g. `agents`, `register`, `sessions`, `revoke`) explicitly state "No git operations"
- [ ] The document's command list is complete: no command present in the binary is absent from the document
- [ ] The document contains no commands that do not exist in the binary
- [ ] Hidden/internal commands (e.g. `_hook`) are documented in a clearly marked "Internal commands" section rather than the main command list
- [ ] The document includes a top-level introduction section explaining what APM is and how to navigate the reference
- [ ] The document groups commands into logical sections (e.g. Ticket lifecycle, Inspection, Workflow orchestration, Administration, Server)

### Out of scope

- CLI tutorial or getting-started guide (narrative walkthrough; this is a reference only)
- Documentation for apm-server internals or its API endpoints
- Man page generation or HTML output (plain Markdown only)
- Documenting private/internal Rust functions or library APIs (apm_core crate internals)
- Automated doc generation from source (no tooling changes; doc is hand-written)
- Any source code changes to add or modify commands

### Approach

## File location

Create `docs/commands.md` in the repository root. This is a pure documentation commit on the ticket branch; no source code changes.

## Document structure

Top-level sections:

1. **Introduction** — one paragraph on what APM is, where config lives (`.apm/apm.toml`), what git-native means, and a note on aggressive mode (auto-fetch, suppressible with `--no-aggressive`).
2. **Command groups** — each group is an H2; commands within it are H3. Proposed groups:
   - **Ticket lifecycle** — `new`, `state`, `set`, `close`, `assign`
   - **Inspection** — `list`, `show`, `next`, `spec`
   - **Workflow orchestration** — `start`, `work`, `workers`, `sync`, `review`
   - **Epics** — `epic list`, `epic new`, `epic close`, `epic show`
   - **Repository maintenance** — `init`, `verify`, `validate`, `archive`, `clean`, `worktrees`
   - **Server** — `register`, `sessions`, `revoke`
   - **Internal** — `_hook`

## Per-command section template

```
### apm <command>

**<one-line tagline>**

#### Synopsis

    apm <command> [<id>] [--flag <value>] ...

#### Description

One to three paragraphs.  
Mention any notable side-effects (e.g. worktree provisioning, push to remote).

#### Options

| Flag / Arg | Type | Default | Description |
|------------|------|---------|-------------|
| `<id>` | positional | — | ... |
| `--flag` | string | — | ... |

#### Git internals

| Command | Why |
|---------|-----|
| `git fetch` | Sync remote ticket branches before reading state |
| `git show <branch>:<file>` | Read ticket content directly from branch blob without checkout |
| ... | ... |
```

## Source of truth

All command details come directly from the source files inventoried during spec-writing. The implementer must cross-check each section against the corresponding `src/cmd/<command>.rs` and the `apm_core` library to ensure accuracy. Particular attention to:

- Exact flag names (long and short forms) from `main.rs` clap definitions
- Git helper functions in `apm_core::git` — trace each helper to the underlying `git` invocation
- Commands that push to remote only in "aggressive" mode (default on) vs. unconditionally

## Git operations reference

The following `git` primitives are used internally (mapped from `apm_core::git`):

| Helper | Underlying git command | Purpose |
|--------|----------------------|---------|
| `fetch_all` | `git fetch --all --prune` | Pull all remote ticket/epic branches |
| `fetch_branch` | `git fetch origin <branch>` | Pull a single ticket branch |
| `ticket_branches` | `git branch -r --list refs/remotes/origin/ticket/*` + local | Enumerate all ticket branches |
| `read_from_branch` | `git show <branch>:<path>` | Read file content from a branch without checkout |
| `commit_to_branch` | `git commit-tree` / orphan commit chain | Write a new file version onto a branch without checkout |
| `push_branch` | `git push origin <branch>` | Publish local branch to remote |
| `delete_remote_branch` | `git push origin --delete <branch>` | Remove terminal-state branches from remote |
| `merged_into_main` | `git branch --merged <default>` | Detect ticket branches merged into main |
| `find_worktree_for_branch` | `git worktree list --porcelain` | Find existing worktree path for a branch |
| `remove_worktree` | `git worktree remove <path>` | Detach and delete worktree directory |
| `list_files_on_branch` | `git ls-tree -r --name-only <branch> <dir>` | List files in a directory on a branch |

## Constraints

- Document must match implementation as of this commit; it is not aspirational
- No changes to source code as part of this ticket
- The document is Markdown only — no HTML, no generated output
- Commit the file on the ticket branch; merge happens through normal review flow

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T17:06Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:43Z | groomed | in_design | philippepascal |