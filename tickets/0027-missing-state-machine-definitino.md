+++
id = 27
title = "missing-state-machine-definitino"
state = "specd"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/0027-missing-state-machine-definitino"
created_at = "2026-03-27T05:28:59.591031Z"
updated_at = "2026-03-27T06:21:10.856086Z"
+++

## Spec

### Problem

apm.agents.md refer to files in init-spec. Nothing in init-spec should be referenced since it won't be present when apm gets installed. instead, apm.tomle is referenced by apm.agents.md. 

do advise if some elements of the specs are missing from apm.agents.md for clarity.

### Acceptance criteria

- [ ] `apm.agents.md` (the template installed by `apm init`) contains no references to `initial_specs/`
- [ ] State machine reference updated to point to `apm.toml` (present in every project)
- [ ] Ticket document format is documented inline in `apm.agents.md` so agents don't need an external file
- [ ] The "Repo structure" section either removed or replaced with a generic placeholder agents can customize — it currently describes the APM source repo, not a user's project
- [ ] `apm agents` (which prints `apm.agents.md`) works correctly in a freshly-initialized project

### Out of scope

- Changing the ticket file format itself
- Generating separate documentation files via `apm init`
- Modifying `initial_specs/` content (those docs can stay for APM developers)

### Approach

Edit `apm.agents.md` at the repo root (the template shipped via `include_str!` in `init.rs`):
1. Replace "State machine reference: `initial_specs/STATE-MACHINE.md`" with "State machine: configured in `apm.toml` under `[[workflow.states]]`"
2. Replace "Ticket document format: `initial_specs/TICKET-SPEC.md`" with an inline summary of the ticket format (frontmatter fields, required spec sections)
3. Replace or remove the APM-source-specific "Repo structure" section — either strip it entirely or leave a placeholder comment for users to fill in
4. Verify no other `initial_specs/` paths remain in the template
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T05:28Z | — | new | apm |
| 2026-03-27T05:38Z | new | question | claude-0326-2222-8071 |
| 2026-03-27T06:05Z | question | new | apm |
| 2026-03-27T06:21Z | new | specd | claude-0326-2222-8071 |
