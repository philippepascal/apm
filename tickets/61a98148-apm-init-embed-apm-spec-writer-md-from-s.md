+++
id = "61a98148"
title = "apm init: embed apm.spec-writer.md from source instead of placeholder stub"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
agent = "45409"
branch = "ticket/61a98148-apm-init-embed-apm-spec-writer-md-from-s"
created_at = "2026-04-02T02:09:54.035008Z"
updated_at = "2026-04-02T02:12:02.984057Z"
+++

## Spec

### Problem

When `apm init` sets up a new project, it creates `.apm/apm.spec-writer.md` with a minimal two-line placeholder stub. Every other template file written during init (`apm.worker.md`, `apm.agents.md`) is embedded from a real source file in `apm-core/src/` via `include_str!()`, so new projects get working, complete instructions out of the box. The spec-writer file is the only exception — it ships empty, leaving spec-writer agents with no guidance until a human manually fills it in.

This matters because spec-writer agents run autonomously on `groomed`, `ammend`, and `in_design` state tickets; they depend on `.apm/apm.spec-writer.md` for their instructions. A placeholder produces low-quality or incomplete specs.

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
| 2026-04-02T02:09Z | — | new | apm |
| 2026-04-02T02:10Z | new | groomed | apm |
| 2026-04-02T02:12Z | groomed | in_design | philippepascal |