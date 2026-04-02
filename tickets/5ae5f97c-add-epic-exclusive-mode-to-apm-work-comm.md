+++
id = "5ae5f97c"
title = "Add --epic exclusive mode to apm work command"
state = "in_design"
priority = 6
effort = 0
risk = 0
author = "claude-0401-2145-a8f3"
agent = "77035"
branch = "ticket/5ae5f97c-add-epic-exclusive-mode-to-apm-work-comm"
created_at = "2026-04-01T21:55:49.406819Z"
updated_at = "2026-04-02T00:49:18.843305Z"
+++

## Spec

### Problem

The `apm work` engine currently dispatches any actionable ticket regardless of epic membership. When a supervisor wants to focus a work session exclusively on one epic's tickets — to drive it to completion without interleaving unrelated work — there is no way to restrict the engine to that scope.

The desired behaviour is:

```
apm work --epic ab12cd34
```

Only tickets whose frontmatter contains `epic = "ab12cd34"` are eligible for dispatch. Free tickets (no `epic` field) and tickets from other epics are skipped entirely. Dependency ordering (`depends_on`) still applies within the filtered set.

A config shorthand is also required so persistent epic focus can be set without repeating the flag:

```toml
[work]
epic = "ab12cd34"   # implies exclusive mode every time apm work runs
```

The CLI flag takes precedence over the config value. This is the exclusive mode described in `docs/epics.md` § `apm work` — Exclusive mode. No other scheduling modes (balanced, --and-free, per-epic limits) are supported.

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
| 2026-04-01T21:55Z | — | new | claude-0401-2145-a8f3 |
| 2026-04-01T22:00Z | new | groomed | claude-0401-2145-a8f3 |
| 2026-04-02T00:49Z | groomed | in_design | philippepascal |