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

- [ ] `apm new "title"` without `--no-edit`, when the user closes the editor without changes, prints `Created ticket ...` and exits 0 with no further output
- [ ] `apm new "title"` without `--no-edit`, when the user edits and saves the ticket, prints `Created ticket ...` and exits 0 with no further output
- [ ] When any `git` command invoked via `git_util::run` fails with empty stderr and non-empty stdout, the error message includes the stdout content rather than being blank
- [ ] `apm new --no-edit "title"` is unaffected: still creates the ticket and exits 0 without opening an editor

### Out of scope

- Changing the default `--no-edit` behaviour; agents should still pass `--no-edit` explicitly
- Fixing `apm show --edit` or `apm review`, which have their own editor flows
- Adding retry logic for genuine commit failures (permission errors, locked index, etc.)

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