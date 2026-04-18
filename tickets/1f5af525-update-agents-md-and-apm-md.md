+++
id = "1f5af525"
title = "update agents.md and apm.*.md"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/1f5af525-update-agents-md-and-apm-md"
created_at = "2026-04-18T18:42:11.878614Z"
updated_at = "2026-04-18T18:48:08.660125Z"
+++

## Spec

### Problem

The three agent instruction files (`agents.md`, `apm.spec-writer.md`, `apm.worker.md`) exist in two locations: the live project copy under `.apm/` and the default templates under `apm-core/src/default/`. These files have drifted out of sync with each other and with the actual CLI behaviour.

Two concrete bugs affect agents following the default templates today:

1. `apm-core/src/default/apm.agents.md` instructs agents to run `git -C <path> add` and `git -C <path> commit` after every `apm spec --set` or `--mark` call. This is wrong: `apm spec` already calls `git::commit_to_branch` internally, so the manual commits would either fail (nothing to commit) or create spurious empty commits. The live `.apm/agents.md` is correct on this point.

2. `apm-core/src/default/apm.agents.md` instructs agents to take over a ticket with `apm take <id>`. The command `apm take` does not exist in the CLI (`apm --help` shows no such subcommand; `apm assign` is the correct command). The live `.apm/agents.md` correctly uses `apm assign <id> <username>`.

Additionally, `apm-core/src/default/apm.spec-writer.md` is missing the "How to save spec sections" block (explaining `--set` vs `--set-file`) and the explicit `apm state <id> specd` call at the end of "When you are done", both of which are present in the live `.apm/apm.spec-writer.md`.

`apm.worker.md` is identical in both locations and requires no changes.

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
| 2026-04-18T18:42Z | — | new | philippepascal |
| 2026-04-18T18:42Z | new | groomed | philippepascal |
| 2026-04-18T18:48Z | groomed | in_design | philippepascal |