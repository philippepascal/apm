+++
id = "295ff9ba"
title = "Add mock_happy demo script for GIF recording"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/295ff9ba-add-mock-happy-demo-script-for-gif-recor"
created_at = "2026-05-04T16:48:42.740876Z"
updated_at = "2026-05-04T16:55:48.609633Z"
epic = "65af2998"
target_branch = "epic/65af2998-apm-demo-enhancements"
depends_on = ["e04e1b3f"]
+++

## Spec

### Problem

The APM project has no demo GIF showing `apm work` orchestrating mock workers across tickets. Producing one manually requires a live Claude session and a GitHub repository, making it slow, expensive, and non-reproducible. `mock-happy` is a built-in APM worker that completes tickets deterministically without any API credentials, making it ideal for scripted demos — but no script exists that drives a representative `apm work` session suitable for recording.

Without such a script, every attempt to produce the README GIF is a manual process: set up a repo, create tickets, wire up mock-happy, record, discard. The script that should encapsulate this setup does not exist.

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
| 2026-05-04T16:48Z | — | new | philippepascal |
| 2026-05-04T16:50Z | new | groomed | philippepascal |
| 2026-05-04T16:55Z | groomed | in_design | philippepascal |