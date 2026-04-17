+++
id = "3d73a43b"
title = "apm clean fail on epics with work tree"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/3d73a43b-apm-clean-fail-on-epics-with-work-tree"
created_at = "2026-04-17T18:28:11.666627Z"
updated_at = "2026-04-17T18:28:11.666627Z"
+++

## Spec

### Problem

apm clean --epics
Would delete 8 epic(s):
  18dab82d  Ticket Ownership Model
  1b029f52  Refactor Apm Cli Code Organization
  1e706443  Refactor Apm Server Code Organization
  35199c7f  Give Workers Cross Ticket Context
  57bce963  Refactor Apm Core Module Structure
  6062f74f  Consolidate Git Operations Into Git Util
  8db73240  User Mgmt
  ac0fb648  Code Separation And Reuse Cleanup
Delete 8 epic(s)? [y/N] y
error: failed to delete local branch epic/18dab82d-ticket-ownership-model: error: cannot delete branch 'epic/18dab82d-ticket-ownership-model' used by worktree at '/Users/philippepascal/repos/apm--worktrees/apm--worktrees/epic-18dab82d-ticket-ownership-model'
deleted epic/1b029f52-refactor-apm-cli-code-organization
deleted epic/1e706443-refactor-apm-server-code-organization
error: failed to delete local branch epic/35199c7f-give-workers-cross-ticket-context: error: cannot delete branch 'epic/35199c7f-give-workers-cross-ticket-context' used by worktree at '/Users/philippepascal/repos/apm--worktrees/apm--worktrees/epic-35199c7f-give-workers-cross-ticket-context'
deleted epic/57bce963-refactor-apm-core-module-structure
deleted epic/6062f74f-consolidate-git-operations-into-git-util
error: failed to delete local branch epic/8db73240-user-mgmt: error: cannot delete branch 'epic/8db73240-user-mgmt' used by worktree at '/Users/philippepascal/repos/apm--worktrees/epic-8db73240-user-mgmt'
error: failed to delete local branch epic/ac0fb648-code-separation-and-reuse-cleanup: error: cannot delete branch 'epic/ac0fb648-code-separation-and-reuse-cleanup' used by worktree at '/Users/philippepascal/repos/apm--worktrees/epic-ac0fb648-code-separation-and-reuse-cleanup'

### Acceptance criteria

- [ ] At least an error message explaining the user what needs to be done, but better if this can be done automatically

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
| 2026-04-17T18:28Z | — | new | philippepascal |
