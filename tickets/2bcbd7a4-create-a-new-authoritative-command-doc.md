+++
id = "2bcbd7a4"
title = "create a new authoritative command doc"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/2bcbd7a4-create-a-new-authoritative-command-doc"
created_at = "2026-04-07T17:06:49.569239Z"
updated_at = "2026-04-07T17:43:53.349020Z"
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

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-07T17:06Z | — | new | philippepascal |
| 2026-04-07T17:42Z | new | groomed | apm |
| 2026-04-07T17:43Z | groomed | in_design | philippepascal |