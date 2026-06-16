+++
id = "9ff929d1"
title = "apm new successfully creates a ticket but outputs Error:"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/9ff929d1-apm-new-successfully-creates-a-ticket-bu"
created_at = "2026-06-16T18:19:39.121805Z"
updated_at = "2026-06-16T18:23:11.032066Z"
+++

## Spec

### Problem

When `apm new` is run without `--no-edit`, it opens `$EDITOR` after creating the ticket. After the editor closes, `open_editor` in `apm/src/cmd/new.rs` calls `commit_to_branch` with the content read back from the temp file. If the user makes no changes (or saves without editing), the content is identical to what `ticket::create` already committed to the branch. `git commit` then exits with code 1 and writes "nothing to commit, working tree clean" to **stdout** — not stderr. The `git_util::run` helper captures only stderr for error messages, so `String::from_utf8_lossy(&out.stderr).trim()` is empty. `anyhow::bail!("{}", "")` then propagates an error with an empty message string, which anyhow formats as `Error: ` (a trailing space). In most terminals this renders as `Error:` on its own line, with no explanation.

The user sees the ticket created successfully (`Created ticket ...`) and then immediately sees `Error:` with no explanation — which is alarming and confusing because the ticket does exist and is valid. The actual operation that failed was a no-op commit attempt. There are two defects: the empty bail message in `git_util::run` (which can affect any git command that writes failure output to stdout), and the unnecessary commit attempt in `open_editor` when content is unchanged.

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
| 2026-06-16T18:19Z | — | new | philippepascal |
| 2026-06-16T18:20Z | new | groomed | philippepascal |
| 2026-06-16T18:23Z | groomed | in_design | philippepascal |