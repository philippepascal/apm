+++
id = "f19a6c21"
title = "create a set of install/uninstall scripts for apm on all platforms supported, including brew"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
branch = "ticket/f19a6c21-create-a-set-of-install-uninstall-script"
created_at = "2026-04-07T17:07:48.816446Z"
updated_at = "2026-04-07T17:07:48.816446Z"
+++

## Spec

### Problem

These shell scripts will allow users to install apm straight from the home page of apm. they support linux and mac (aarch64-apple-darwin and x86_64-unknown-linux-musl) (complement brew). They live in the script director of apm,
Typically, with a simple command line, a user can have the script run on their terminal. Running it will download the appropriate binary, put it in the right place ,set the path, etc. uninstall will do the cleanup.

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
| 2026-04-07T17:07Z | — | new | philippepascal |