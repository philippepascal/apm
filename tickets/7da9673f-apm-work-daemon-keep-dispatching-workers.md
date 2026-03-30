+++
id = "7da9673f"
title = "apm work --daemon: keep dispatching workers as tickets become actionable"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "91785"
branch = "ticket/7da9673f-apm-work-daemon-keep-dispatching-workers"
created_at = "2026-03-30T17:27:51.137680Z"
updated_at = "2026-03-30T17:29:35.459784Z"
+++

## Spec

### Problem

Currently `apm work` exits as soon as there are no more actionable tickets — either because all slots are filled or the queue is empty. If the supervisor is away and a worker finishes (freeing a slot) or a ticket transitions to `ready` (new work becomes available), nothing picks it up. The supervisor has to manually re-run `apm work`.

`apm work --daemon` would keep the process alive, polling at a configurable interval, and dispatch new workers as soon as slots open up or actionable tickets appear. This enables fully unattended operation: start the daemon, walk away, come back to completed work.

The daemon should be interruptible with Ctrl-C and should log each dispatch cycle clearly so the supervisor can see what happened while away.

### Acceptance criteria

- [ ] `apm work --daemon` continues running after `apm next` returns null (queue exhausted)
- [ ] When a worker finishes and a slot opens, the daemon immediately re-checks for actionable tickets without waiting for the poll interval
- [ ] When the poll interval elapses with no workers finishing, the daemon re-checks for actionable tickets
- [ ] `apm work --daemon --interval <N>` sets the poll interval to N seconds; default is 30
- [ ] Each dispatch cycle logs a timestamped line: ticket dispatched, worker finished, or no tickets found with seconds until next check
- [ ] Ctrl-C stops the daemon; workers already running continue to completion as independent processes
- [ ] `apm work` without `--daemon` retains existing behaviour: exits when the queue is exhausted and all workers finish
- [ ] `apm work --daemon --dry-run` exits immediately with an error message

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T17:27Z | — | new | philippepascal |
| 2026-03-30T17:29Z | new | in_design | philippepascal |