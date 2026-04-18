+++
id = "29b495c7"
title = "fix ticket history actors"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/29b495c7-fix-ticket-history-actors"
created_at = "2026-04-18T02:20:26.518634Z"
updated_at = "2026-04-18T02:21:54.724243Z"
+++

## Spec

### Problem

Ticket history entries record the wrong actor in three distinct cases, all traceable to actor-resolution logic that is inconsistent across the codebase.

**Case 1 & 2 — `state::transition()` ignores the OS user.**
`apm-core/src/state.rs` line 114 resolves the actor as:
```rust
let actor = std::env::var("APM_AGENT_NAME").unwrap_or_else(|_| "apm".into());
```
This only checks `APM_AGENT_NAME`. When a human user runs `apm state` or `apm review` in a shell where that variable is unset, the fallback fires and records `"apm"` — even though `USER`/`USERNAME` env vars identify the real user. The `apm start` command avoids this by using `resolve_caller_name()`, which chains `APM_AGENT_NAME → USER → USERNAME → "apm"`, but `state::transition()` was never updated to match.

Affected transitions: any state change driven by `apm state` or `apm review` (including automatic grooming and review-to-ready promotions).

**Case 3 — `apm sync` hardcodes `"apm-sync"` as the actor.**
`apm/src/cmd/sync.rs` line 41 passes the literal string `"apm-sync"` to `sync::apply()`. This loses the identity of the human who invoked the command. The desired format is `philippepascal(apm-sync)` — the real caller identity, annotated with `(apm-sync)` to signal that the close was automatic. The server-side sync handler in `apm-server/src/handlers/maintenance.rs` already does this correctly by calling `resolve_identity()` first and passing it through; the CLI path was not aligned with it.

`apm/src/cmd/close.rs` has the same `APM_AGENT_NAME`-only fallback as `state.rs` and should be updated for consistency, even though it is not directly implicated in the reported examples.

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
| 2026-04-18T02:20Z | — | new | philippepascal |
| 2026-04-18T02:21Z | new | groomed | apm |
| 2026-04-18T02:21Z | groomed | in_design | philippepascal |