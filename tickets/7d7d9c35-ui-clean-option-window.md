+++
id = "7d7d9c35"
title = "UI: clean option window"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/7d7d9c35-ui-clean-option-window"
created_at = "2026-04-09T05:15:30.189617Z"
updated_at = "2026-04-09T05:23:11.521685Z"
+++

## Spec

### Problem

The clean command exposes seven flags (--dry-run, --force, --branches, --remote, --older-than, --untracked, --yes) and produces detailed log output (per-worktree, per-branch, per-remote-branch messages plus warnings). The existing Clean button in the SupervisorView header fires a hard-coded POST /api/clean with no options and discards all log output — the server handler ignores every flag and returns only a removed count.

Users who need to preview what clean will do (dry-run), delete local branches alongside worktrees (--branches), clean remote branches (--remote --older-than), force-remove unmerged worktrees (--force), or remove untracked files first (--untracked) must fall back to the CLI. Log output produced during dry-run is completely invisible in the UI.

The fix is to replace the one-click Clean button with a modal window that exposes all options and displays the command log output, making dry-run a first-class UI workflow.

### Acceptance criteria

- [ ] Clicking the Clean button in the SupervisorView header opens the Clean modal instead of immediately running clean
- [ ] The Clean modal displays a Dry run checkbox
- [ ] The Clean modal displays a Branches checkbox (also remove local ticket/* branches)
- [ ] The Clean modal displays a Force checkbox (bypass merge checks)
- [ ] The Clean modal displays an Untracked checkbox (remove untracked files from worktrees before removal)
- [ ] The Clean modal displays a Remote checkbox
- [ ] When Remote is checked, an Older than text field appears and is required before Run is enabled
- [ ] When Remote is unchecked, the Older than field is hidden
- [ ] The Run button is disabled when Remote is checked and Older than is empty
- [ ] The modal has a scrollable log output area that is empty on open
- [ ] Clicking Run calls POST /api/clean with the selected options and displays the returned log lines in the output area
- [ ] While the request is in-flight the Run button shows a spinner and is disabled
- [ ] When Dry run is checked the Run button label reads "Dry run"; otherwise it reads "Run"
- [ ] After a successful non-dry-run execution the tickets query is invalidated (board refreshes)
- [ ] After a successful dry-run execution the tickets query is NOT invalidated
- [ ] Pressing Escape closes the modal
- [ ] Clicking outside the modal (backdrop) closes the modal
- [ ] The modal can be reopened cleanly after being closed (state resets: checkboxes unchecked, older-than cleared, log output cleared)
- [ ] POST /api/clean accepts an optional JSON body with fields: dry_run, force, branches, remote, older_than, untracked
- [ ] POST /api/clean returns a JSON object with a log field (string) containing all output lines and a removed field (number)
- [ ] When --remote is used with --older-than, the server passes both to the clean logic and remote candidates appear in the log

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
| 2026-04-09T05:15Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:23Z | groomed | in_design | philippepascal |