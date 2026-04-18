+++
id = "5473a0e6"
title = "UI clean mod window cancel ->close"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5473a0e6-ui-clean-mod-window-cancel-close"
created_at = "2026-04-17T20:18:54.917961Z"
updated_at = "2026-04-18T01:02:48.834421Z"
+++

## Spec

### Problem

The Clean worktrees modal has two minor UI issues in CleanModal.tsx. First, the dismiss button is labelled Cancel, which implies the action was aborted; since closing without running a clean is a neutral action, the label should read Close. Second, the untracked checkbox defaults to false; removing untracked files is the common case, so it should be pre-checked to reduce unnecessary clicks.

### Acceptance criteria

- [x] The dismiss button in the Clean worktrees modal displays the label Close\n- [x] Clicking Close dismisses the modal without performing any clean action\n- [x] The Untracked checkbox is checked by default when the modal opens\n- [x] The Untracked checkbox can still be unchecked by the user before running the clean

### Out of scope

- Any changes to the clean action logic itself\n- Renaming or restyling other buttons in the modal\n- Persisting the untracked checkbox state across modal sessions

### Approach

Both changes are in apm-ui/src/components/CleanModal.tsx.\n\n1. Change the untracked initial state (line ~21) from useState(false) to useState(true).\n2. Change the button label text (line ~139) from Cancel to Close.\n\nNo other files require changes.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-17T20:18Z | — | new | philippepascal |
| 2026-04-17T20:23Z | new | groomed | apm |
| 2026-04-17T20:30Z | groomed | in_design | philippepascal |
| 2026-04-17T20:31Z | in_design | specd | claude-0417-2030-6728 |
| 2026-04-17T21:45Z | specd | ready | apm |
| 2026-04-17T21:45Z | ready | in_progress | philippepascal |
| 2026-04-17T21:48Z | in_progress | implemented | claude-0417-2145-a258 |
| 2026-04-18T01:02Z | implemented | closed | philippepascal |
