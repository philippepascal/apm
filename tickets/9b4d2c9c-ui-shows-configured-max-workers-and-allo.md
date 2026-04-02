+++
id = "9b4d2c9c"
title = "UI shows configured max workers and allow override"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm-ui"
agent = "48994"
branch = "ticket/9b4d2c9c-ui-shows-configured-max-workers-and-allo"
created_at = "2026-04-02T19:20:21.647921Z"
updated_at = "2026-04-02T19:22:24.726022Z"
+++

## Spec

### Problem

The Work Engine Controls UI does not display the currently configured `agents.max_concurrent` value from `.apm/config.toml`. Users have no way to see how many workers the engine will spawn, and no way to change that number without manually editing the config file.

The problem has two parts: (1) the UI omits the value entirely, and (2) even if a UI control existed, there is no API endpoint to persist a change back to the config file. `post_work_start` reads `config.agents.max_concurrent` fresh on each start — so a runtime override that does not write to the file has no effect on the next start.

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
| 2026-04-02T19:20Z | — | new | apm-ui |
| 2026-04-02T19:20Z | new | groomed | apm |
| 2026-04-02T19:22Z | groomed | in_design | philippepascal |