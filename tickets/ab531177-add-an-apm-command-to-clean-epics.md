+++
id = "ab531177"
title = "add an apm command to clean epics"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ab531177-add-an-apm-command-to-clean-epics"
created_at = "2026-04-09T05:07:02.660761Z"
updated_at = "2026-04-09T05:18:26.521041Z"
+++

## Spec

### Problem

Epics accumulate over time as a project progresses. Once all tickets in an epic reach a terminal state (`derive_epic_state` returns `"done"`), the epic branch and its `.apm/epics.toml` entry serve no further purpose but remain in the repository indefinitely. There is currently no way to remove them short of manual `git branch -d` and hand-editing `.apm/epics.toml`.

This ticket adds `apm epic clean` — a subcommand that identifies all "done" epics, presents the list to the user, and deletes them (local branch + metadata entry) after confirmation. A `--yes` flag allows non-interactive use, and `--dry-run` lets users preview what would be removed without side effects.

### Acceptance criteria

- [ ] `apm epic clean` with no flags prints the list of "done" epics and prompts "Delete N epic(s)? [y/N]"; entering "y" deletes them
- [ ] `apm epic clean --yes` deletes all "done" epics without prompting
- [ ] `apm epic clean --dry-run` prints what would be deleted and exits without making any changes
- [ ] Epics whose derived state is not `"done"` are not listed and not deleted
- [ ] When no "done" epics exist, the command prints "Nothing to clean." and exits 0
- [ ] After deletion, the epic branch no longer exists locally
- [ ] After deletion, the epic's entry is removed from `.apm/epics.toml` (or the file is left unchanged if the epic had no entry there)
- [ ] Running in a non-interactive terminal without `--yes` skips deletion and prints a message advising the user to use `--yes`
- [ ] Entering anything other than "y" at the prompt leaves all epics untouched

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
| 2026-04-09T05:07Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:18Z | groomed | in_design | philippepascal |