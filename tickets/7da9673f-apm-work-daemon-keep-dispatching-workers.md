+++
id = "7da9673f"
title = "apm work --daemon: keep dispatching workers as tickets become actionable"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
agent = "philippepascal"
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
