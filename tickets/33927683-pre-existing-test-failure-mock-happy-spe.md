+++
id = "33927683"
title = "Pre-existing test failure: mock_happy_spec_mode_transitions_to_specd"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/33927683-pre-existing-test-failure-mock-happy-spe"
created_at = "2026-05-04T03:33:27.432606Z"
updated_at = "2026-05-04T04:35:40.291779Z"
+++

## Spec

### Problem

start::tests::mock_happy_spec_mode_transitions_to_specd in apm-core/src/start.rs fails because find_apm_bin() resolves to apm-server via which apm rather than the apm CLI binary. The script calls apm spec but apm-server does not know that subcommand. Pre-dates ticket f8cbd68c — confirmed by git stash.

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
| 2026-05-04T03:33Z | — | new | claude-0503-1430-f8cb|philippepascal |
| 2026-05-04T04:35Z | new | groomed | philippepascal |
| 2026-05-04T04:35Z | groomed | in_design | philippepascal |
