+++
id = "066d21de"
title = "apm show: open ticket in $EDITOR"
state = "in_design"
priority = 0
effort = 2
risk = 0
author = "apm"
agent = "2429"
branch = "ticket/066d21de-apm-show-open-ticket-in-editor"
created_at = "2026-03-31T04:32:11.186464Z"
updated_at = "2026-03-31T04:37:14.614619Z"
+++

## Spec

### Problem

apm show <id> currently prints ticket content to stdout. Engineers and agents often want to read a ticket in their editor for a better reading experience — especially for long specs. There is no way to open the ticket file directly in $EDITOR without manually locating it.

### Acceptance criteria

- [ ] `apm show <id> --edit` opens the ticket in `$VISUAL` (falling back to `$EDITOR`, then `vi`) instead of printing to stdout
- [ ] Without `--edit`, `apm show` continues to print to stdout unchanged
- [ ] The editor receives a temp file containing the full raw ticket content (frontmatter + body)
- [ ] After the editor exits cleanly, if the content changed, it is committed to the ticket branch via `commit_to_branch` with message `ticket(<id>): edit`
- [ ] If the content is unchanged after editing, no commit is made
- [ ] When `$VISUAL` and `$EDITOR` are both unset, the command falls back to `vi`
- [ ] If the editor exits with a non-zero status, the command prints an error and exits non-zero without committing

### Out of scope

- Making `--edit` the default behaviour (stdout remains the default)
- Editing tickets that do not yet have a branch (new tickets before `apm new`)
- Conflict resolution when the branch was updated remotely during editing

### Approach

Add an `--edit` flag to the `Show` subcommand in `apm/src/main.rs` and thread it through to `apm/src/cmd/show.rs`.

In `show.rs`, after the ticket content is read from the branch blob, if `--edit` is set:

1. Write the raw content to a temp file under `std::env::temp_dir()` (named `apm-<id>.md`)
2. Resolve the editor: `$VISUAL` → `$EDITOR` → `"vi"` (same logic as `cmd/review.rs::open_editor`)
3. Spawn the editor with the temp file path, inheriting stdin/stdout/stderr
4. If the editor exits non-zero, bail with an error
5. Read the temp file back; if it differs from the original, call `apm_core::git::commit_to_branch` with message `ticket(<id>): edit`
6. Delete the temp file

No new dependencies are required — `std::fs` handles temp file I/O and the editor pattern already exists in `review.rs`.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T04:32Z | — | new | apm |
| 2026-03-31T04:32Z | new | in_design | philippepascal |