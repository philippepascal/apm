+++
id = "4660b156"
title = "Split ticket.rs into ticket_fmt.rs and ticket_util.rs"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4660b156-split-ticket-rs-into-ticket-fmt-rs-and-t"
created_at = "2026-04-12T06:04:17.196705Z"
updated_at = "2026-04-12T06:04:17.196705Z"
epic = "57bce963"
target_branch = "epic/57bce963-refactor-apm-core-module-structure"
+++

## Spec

### Problem

`ticket.rs` is a large file mixing two distinct concerns: (1) file format parsing/serialization (TOML frontmatter, markdown body, checklist parsing, slugification, ID normalization) and (2) ticket manipulation logic (scoring, priority calculation, dependency graphs, filtering, creation, closing). This makes the module hard to navigate and creates unnecessary coupling.

The split into `ticket_fmt.rs` (format) and `ticket_util.rs` (logic) gives each module a clear responsibility. A thin `ticket.rs` re-export hub preserves downstream imports in `apm` and `apm-server`.

See [REFACTOR-CORE.md](../../REFACTOR-CORE.md) section 4 for the full plan.

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
| 2026-04-12T06:04Z | — | new | philippepascal |