+++
id = 27
title = "missing-state-machine-definitino"
state = "new"
priority = 0
effort = 0
risk = 0
author = "apm"
branch = "ticket/0027-missing-state-machine-definitino"
created_at = "2026-03-27T05:28:59.591031Z"
updated_at = "2026-03-27T06:05:58.827874Z"
+++

## Spec

### Problem

apm.agents.md refer to files in init-spec. Nothing in init-spec should be referenced since it won't be present when apm gets installed. instead, apm init creates a state machine definition in a file, and that file is referenced by apm.agents.md. users can then customize that file.

### Acceptance criteria

### Out of scope

### Approach
## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-27T05:28Z | — | new | apm |
| 2026-03-27T05:38Z | new | question | claude-0326-2222-8071 |
| 2026-03-27T06:05Z | question | new | apm |
