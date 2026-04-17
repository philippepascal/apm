+++
id = "5473a0e6"
title = "UI clean mod window cancel ->close"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/5473a0e6-ui-clean-mod-window-cancel-close"
created_at = "2026-04-17T20:18:54.917961Z"
updated_at = "2026-04-17T20:30:13.322446Z"
+++

## Spec

### Problem

The Clean worktrees modal has two minor UI issues in CleanModal.tsx. First, the dismiss button is labelled Cancel, which implies the action was aborted; since closing without running a clean is a neutral action, the label should read Close. Second, the untracked checkbox defaults to false; removing untracked files is the common case, so it should be pre-checked to reduce unnecessary clicks.

### Acceptance criteria

- [ ] The dismiss button in the Clean worktrees modal displays the label Close\n- [ ] Clicking Close dismisses the modal without performing any clean action\n- [ ] The Untracked checkbox is checked by default when the modal opens\n- [ ] The Untracked checkbox can still be unchecked by the user before running the clean

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
| 2026-04-17T20:18Z | — | new | philippepascal |
| 2026-04-17T20:23Z | new | groomed | apm |
| 2026-04-17T20:30Z | groomed | in_design | philippepascal |