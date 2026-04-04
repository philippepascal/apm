+++
id = "e1c4ca76"
title = "UI Workers list sorts tickets by worker status, with crashed and ended last"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm-ui"
branch = "ticket/e1c4ca76-ui-workers-list-sorts-tickets-by-worker-"
created_at = "2026-04-04T03:16:37.369960Z"
updated_at = "2026-04-04T06:40:49.124964Z"
+++

## Spec

### Problem

The Workers list in the UI (WorkerActivityPanel.tsx) renders workers in the order returned by the API — currently unordered. When there are a mix of running, crashed, and ended workers, active workers can appear below idle ones, making it hard to quickly spot what is running.

The desired behaviour is that running workers always appear at the top, and crashed/ended workers are pushed to the bottom. This improves at-a-glance status awareness for the supervisor.

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
| 2026-04-04T03:16Z | — | new | apm-ui |
| 2026-04-04T06:02Z | new | groomed | apm |
| 2026-04-04T06:40Z | groomed | in_design | philippepascal |