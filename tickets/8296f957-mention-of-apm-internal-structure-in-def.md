+++
id = "8296f957"
title = "mention of apm internal structure in default claude apm.*.md"
state = "closed"
priority = 0
effort = 1
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/8296f957-mention-of-apm-internal-structure-in-def"
created_at = "2026-05-28T02:19:13.637222Z"
updated_at = "2026-05-28T06:43:23.479685Z"
+++

## Spec

### Problem

The default Claude coder template shipped with APM (`apm-core/src/default/agents/claude/apm.coder.md`) has three APM-specific lines hardcoded in its `## Tests and finishing` section:

```
- Unit tests inline in each crate (`apm-core/src/`) or in `apm-core/tests/`
- Integration tests in `apm/tests/integration.rs` — temp git repos, no fixtures
- Run `cargo test --workspace` — all tests must pass
```

When a new (non-APM) project runs `apm init`, it receives this file verbatim. The coder agent assigned to any ticket in that project is then told to look for `apm-core/src/`, `apm/tests/integration.rs`, and to run `cargo test --workspace` — paths and commands that do not exist in their repo. Any project using APM with a coder agent gets misleading test instructions unless they manually edit the file after init.

### Acceptance criteria

- [x] The template file `apm-core/src/default/agents/claude/apm.coder.md` contains no reference to `apm-core/src/`, `apm-core/tests/`, `apm/tests/integration.rs`, or `cargo test`
- [x] The `## Tests and finishing` section in the template gives generic guidance that applies to any project, instructing the agent to consult `apm.project.md` for project-specific test conventions and commands
- [x] All other sections of the template file are unchanged

### Out of scope

- Updating `.apm/agents/claude/apm.coder.md` — APM's live copy, which correctly references APM's own test structure and is not a template
- Updating test guidance in the debug, mock, phi4, or pi agent variants
- Adding project-type detection or conditional content to the template
- Changing the CLAUDE.md scaffold or `apm.project.md` template

### Approach

Edit `apm-core/src/default/agents/claude/apm.coder.md`. In the `## Tests and finishing` section, replace the three APM-specific bullet points:

```
- Unit tests inline in each crate (`apm-core/src/`) or in `apm-core/tests/`
- Integration tests in `apm/tests/integration.rs` — temp git repos, no fixtures
- Run `cargo test --workspace` — all tests must pass
```

with generic alternatives:

```
- Follow the test conventions described in `apm.project.md`
- Run the project's test suite — all tests must pass
```

No other files change. No Rust code is touched, so no `cargo test` run is needed.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-05-28T02:19Z | — | new | philippepascal |
| 2026-05-28T06:09Z | new | groomed | philippepascal |
| 2026-05-28T06:14Z | groomed | in_design | philippepascal |
| 2026-05-28T06:16Z | in_design | specd | claude |
| 2026-05-28T06:27Z | specd | ready | philippepascal |
| 2026-05-28T06:37Z | ready | in_progress | philippepascal |
| 2026-05-28T06:38Z | in_progress | implemented | claude |
| 2026-05-28T06:43Z | implemented | closed | philippepascal(apm-sync) |
