+++
id = 34
title = "new-command-take-free-text"
state = "closed"
priority = 0
effort = 3
risk = 1
author = "apm"
agent = "claude-0327-2000-34ee"
branch = "ticket/0034-new-command-take-free-text"
created_at = "2026-03-27T21:11:48.964488Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

After `apm new "title"`, the ticket is created but the supervisor must separately check out the branch, open the file, fill in the spec, commit, and transition state. This friction discourages writing a good spec immediately. When a human supervisor creates a ticket, they usually know the problem and want to write the spec right then. The command should open `$EDITOR` automatically so the spec can be written in one flow.

### Acceptance criteria

- [ ] After `apm new "<title>"`, `$EDITOR` opens with the ticket file pre-populated on the ticket branch
- [ ] When the user saves and closes the editor, the changes are committed to the ticket branch automatically with message `ticket(<id>): write spec`
- [ ] A `--no-edit` flag skips the editor (current behavior, useful for scripts and agents)
- [ ] If `$EDITOR` is unset, `apm new` warns and skips the editor (does not fail)
- [ ] If the editor exits non-zero, the file as written is still committed (editor abort ≠ content loss)

### Out of scope

- Agents using `apm new` interactively (agents pass `--no-edit` by convention)
- Templating beyond the current skeleton structure
- Syntax validation of the spec before commit

### Approach

In `apm/src/cmd/new.rs`, after `commit_to_branch` creates the ticket, check `$EDITOR`. If set and `--no-edit` is not passed: check out the ticket branch, open `$EDITOR <file>` via `std::process::Command::new(editor).arg(&path).status()`, then `git add` + `git commit` the result and check back out to the previous branch. The `--no-edit` flag is added as a boolean arg to the `New` command in `main.rs`.

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T21:11Z | — | new | apm |
| 2026-03-28T01:03Z | new | specd | claude-0327-1757-391b |
| 2026-03-28T01:05Z | specd | ready | apm |
| 2026-03-28T02:09Z | ready | in_progress | claude-0327-2000-34ee |
| 2026-03-28T02:11Z | in_progress | implemented | claude-0327-2000-34ee |
| 2026-03-28T07:31Z | implemented | accepted | apm sync |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |