+++
id = "02f36f58"
title = "apm show: display depends_on, epic, and target_branch frontmatter fields"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/02f36f58-apm-show-display-depends-on-epic-and-tar"
created_at = "2026-04-04T00:27:04.270671Z"
updated_at = "2026-04-04T06:36:51.413654Z"
+++

## Spec

### Problem

When running `apm show <id>`, the output header displays `state`, `priority`, `effort`, `risk`, and `branch` — but three optional frontmatter fields are silently omitted: `epic`, `target_branch`, and `depends_on`.

These fields are fully parsed and stored in the `Frontmatter` struct (see `apm-core/src/ticket.rs`), and they carry meaningful context: which epic a ticket belongs to, which branch it targets, and which other tickets must complete before it can start. Without them in `apm show`, an agent or developer reading a ticket must look at the raw file to discover dependencies or epic membership — defeating the purpose of the command.

### Acceptance criteria

- [ ] `apm show <id>` prints an `epic:` line when the ticket's `epic` frontmatter field is set
- [ ] `apm show <id>` prints a `target_branch:` line when the ticket's `target_branch` frontmatter field is set
- [ ] `apm show <id>` prints a `depends_on:` line when the ticket's `depends_on` frontmatter field is set and non-empty
- [ ] `apm show <id>` omits the `epic:` line entirely when the field is absent
- [ ] `apm show <id>` omits the `target_branch:` line entirely when the field is absent
- [ ] `apm show <id>` omits the `depends_on:` line entirely when the field is absent or empty
- [ ] The three new lines appear in the header block (before the blank line that separates frontmatter from the body), after the existing `branch:` line

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
| 2026-04-04T00:27Z | — | new | apm |
| 2026-04-04T06:01Z | new | groomed | apm |
| 2026-04-04T06:36Z | groomed | in_design | philippepascal |