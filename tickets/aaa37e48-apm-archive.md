+++
id = "aaa37e48"
title = "apm archive"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/aaa37e48-apm-archive"
created_at = "2026-04-03T00:33:18.924269Z"
updated_at = "2026-04-04T06:25:35.886928Z"
+++

## Spec

### Problem

As tickets are closed over time, the `tickets/` directory on `main` accumulates stale files indefinitely. While `apm list` hides terminal-state tickets by default, the files remain on disk and clutter the working directory for anyone browsing the repository. There is no automated way to sweep closed ticket files into a separate archive location.

This ticket adds `apm archive`, a command that moves closed ticket files from the active `tickets/` directory to a configurable archive directory on `main`. It also adds the `archive_dir` config key to `[tickets]` in `config.toml`, and extends the `apm show` fallback path so that archived tickets (whose per-ticket branch was later deleted by `apm clean --branches`) remain discoverable.

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
| 2026-04-03T00:33Z | — | new | apm |
| 2026-04-04T06:01Z | new | groomed | apm |
| 2026-04-04T06:25Z | groomed | in_design | philippepascal |