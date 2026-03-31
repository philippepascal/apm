+++
id = "651f8a63"
title = "apm-server + apm-ui: worker activity panel (running workers, top of left column)"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "19736"
branch = "ticket/651f8a63-apm-server-apm-ui-worker-activity-panel-"
created_at = "2026-03-31T06:12:27.354130Z"
updated_at = "2026-03-31T06:33:51.720971Z"
+++

## Spec

### Problem

The top half of the left column (WorkerView) in `apm-ui` currently shows a placeholder stub from Step 4. There is no way to see which worker processes are running or which tickets they hold without leaving the browser and using the CLI. This creates an observability gap: supervisors must context-switch to the terminal to assess worker health.

Adding `GET /api/workers` to `apm-server` and wiring up a WorkerActivityPanel component in `apm-ui` gives supervisors an at-a-glance view of running workers, their assigned tickets, current state, agent name, and elapsed time — all without leaving the browser.

**Current state:** Left column top half is a stub/placeholder (from Step 4). `apm-core/src/worker.rs` already implements `read_pid_file`, `is_alive`, and `elapsed_since` — tested and ready to call. Workers write `.apm-worker.pid` files to their worktrees (implemented by ticket 0084).

**Desired state:** The WorkerActivityPanel polls `GET /api/workers` every 5 seconds and renders a list of live and crashed workers with their ticket id, title, state, agent name, and elapsed time.

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
| 2026-03-31T06:12Z | — | new | apm |
| 2026-03-31T06:33Z | new | in_design | philippepascal |