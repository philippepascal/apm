+++
id = "6e3f9e91"
title = "Add global max_workers_per_epic config; remove per-epic override"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/6e3f9e91-add-global-max-workers-per-epic-config-r"
created_at = "2026-04-27T20:28:07.069581Z"
updated_at = "2026-04-27T20:47:50.085386Z"
epic = "5ea30227"
target_branch = "epic/5ea30227-strategy-and-dependency-hardening"
+++

## Spec

### Problem

Per-epic concurrency is currently controlled via a per-epic override: `apm epic set <id> max_workers <N>` writes a `max_workers` entry to `.apm/epics.toml`, and both the engine loop (`apm work`) and `apm start --next` read it via `Config::blocked_epics()` to cap concurrent workers per epic. Epics **without** an explicit entry are completely uncapped — any number of workers can be dispatched into the same epic simultaneously.\n\nThe design spec at `docs/strategy-and-dependencies.md` (§ 'Epic concurrency') replaces this model: each epic gets at most one active worker by default, controlled by a single global `max_workers_per_epic` setting (default `1`). Users gain parallelism by creating more epics, not by raising a per-epic cap. This makes epics the atomic parallelism unit and eliminates within-epic merge races.\n\nThe per-epic override mechanism must be removed entirely: `apm epic set <id> max_workers` should become an error, `.apm/epics.toml` should stop being read, and the global limit must be enforced uniformly for every epic — including the `run_next` path (`apm start --next --spawn`), which currently applies no epic concurrency limit at all.

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
| 2026-04-27T20:28Z | — | new | philippepascal |
| 2026-04-27T20:43Z | new | groomed | philippepascal |
| 2026-04-27T20:47Z | groomed | in_design | philippepascal |