+++
id = "8c5dc732"
title = "apm edit"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "philippepascal"
agent = "14429"
branch = "ticket/8c5dc732-apm-edit"
created_at = "2026-03-30T16:11:04.170762Z"
updated_at = "2026-03-30T16:21:32.322788Z"
+++

## Spec

### Problem

The master/delegator agent needs to update ticket spec sections programmatically
without having to enter a ticket worktree. Worktree access from agent subprocesses
triggers permission errors in Claude Code (the `apm--worktrees/` path is blocked
for Edit/Write tool calls in subagents).

`apm spec <id> --section <name> --set <value>` can write inline content, but for
multi-line content it requires `--set -` (read from stdin) which forces a shell
pipe — a compound command pattern that also bypasses the allow list.

There is no `--from-file` equivalent in any current apm command.

### Acceptance criteria

- [ ] `apm edit <id> --section <name> --set <value>` writes inline content to the named section and commits it to the ticket branch
- [ ] `apm edit <id> --section <name> --from-file <path>` reads the file at `<path>` and writes its content to the named section
- [ ] Exactly one of `--set` or `--from-file` must be provided; providing neither or both is an error
- [ ] Section type formatting (tasks, qa, free) is applied based on the `[[ticket.sections]]` config in apm.toml — same logic as `apm spec`
- [ ] Unknown section names (not in config, or not in the built-in list when no config sections are defined) are rejected with a clear error
- [ ] Works from the main repo working directory without touching any worktree; reads and writes the ticket file via git branch blobs
- [ ] Prints confirmation on success: `ticket #<id>: section "<name>" updated`

### Out of scope

- Reading/printing section content (use `apm spec --section`)
- Opening an interactive editor (use `apm review`)
- Marking checklist items (use `apm spec --mark`)
- Validating spec completeness (use `apm spec --check`)
- Updating multiple sections in a single command invocation

### Approach

Add a new `Edit` subcommand to `apm/src/main.rs` and implement it in a new file
`apm/src/cmd/edit.rs`.

The command signature:
```
apm edit <id> --section <name> (--set <value> | --from-file <path>)
```

Implementation in `edit.rs`:
1. Resolve ticket branch from id (same as `spec.rs`)
2. Read ticket content from the branch blob via `git::read_from_branch`
3. Determine content: read from `<path>` if `--from-file`, else use `--set` value
4. Look up the section in `config.ticket.sections`; apply `apply_section_type` to format content
5. Write the formatted content into the section using the existing `set_section_doc` /
   `set_section_body` helpers (extract these from `spec.rs` into a shared location or
   duplicate minimally)
6. Commit the updated ticket back to its branch via `git::commit_to_branch`

The formatting and section-write logic already exists in `spec.rs`. Extract the
relevant helpers (`apply_section_type`, `set_section_doc`, `set_section_body`,
`is_doc_field`) into a small `cmd/spec_helpers.rs` module shared by both `spec.rs`
and `edit.rs`, or simply re-expose them from `spec.rs` as `pub(super)` functions.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T16:11Z | — | new | philippepascal |
| 2026-03-30T16:12Z | new | in_design | philippepascal |
| 2026-03-30T16:15Z | in_design | specd | claude-0330-1615-b7e2 |
| 2026-03-30T16:21Z | specd | closed | philippepascal |