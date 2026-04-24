+++
id = "b7003852"
title = "apm init should print next-step tips on completion"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/b7003852-apm-init-should-print-next-step-tips-on-"
created_at = "2026-04-24T06:28:19.582833Z"
updated_at = "2026-04-24T07:14:25.822187Z"
+++

## Spec

### Problem

After `apm init` completes, the only output is a bare `"apm initialized."` line (`apm/src/cmd/init.rs:62`). New users receive no cue on what to do next: whether to commit the generated `.apm/` files, how to create a first ticket, that a web UI (`apm-server`) exists, or where to find the full command reference.

The desired behaviour is to print a short tips block immediately after `"apm initialized."` that surfaces the four most useful next steps. The block should be suppressed automatically when stdout is not a TTY (so CI pipelines stay clean) and should also respect a `--quiet` flag, consistent with the pattern already established by `apm sync`.

### Acceptance criteria

- [ ] After `apm init` on a TTY without `--quiet`, a tips block is printed after `"apm initialized."` containing suggestions to commit `.apm/` files, run `apm new`, try `apm-server`, and check `apm --help`
- [ ] When stdout is not a TTY, the tips block is suppressed and only `"apm initialized."` is printed
- [ ] `apm init --quiet` suppresses the tips block even when run on a TTY
- [ ] `apm init --quiet` does not suppress `"apm initialized."` (the confirmation line always prints)
- [ ] `apm init --quiet` is accepted by the CLI without error (the flag is wired up end-to-end)
- [ ] `apm init --help` documents the `--quiet` flag with a description

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
| 2026-04-24T06:28Z | — | new | philippepascal |
| 2026-04-24T07:12Z | new | groomed | philippepascal |
| 2026-04-24T07:14Z | groomed | in_design | philippepascal |