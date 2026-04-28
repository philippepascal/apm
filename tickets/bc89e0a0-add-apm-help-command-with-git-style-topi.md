+++
id = "bc89e0a0"
title = "Add apm help command with git-style topic dispatch"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/bc89e0a0-add-apm-help-command-with-git-style-topi"
created_at = "2026-04-28T19:27:00.760945Z"
updated_at = "2026-04-28T19:33:33.568010Z"
epic = "e3b24cb9"
target_branch = "epic/e3b24cb9-apm-help-auto-derived-git-style-topic-he"
+++

## Spec

### Problem

There is no unified `apm help` command today. Users discover apm surface area by running `apm <subcommand> --help` for each command individually and reading source for config schemas. A git-style `apm help [topic]` entry point would give users a single landing point to orient themselves across commands, config, and workflow concepts.

This ticket adds CLI plumbing only: the `Help` subcommand variant in the clap `Command` enum, dispatch wiring in `main()`, and a new `cmd::help` module with four stub renderer functions. No real content is produced here; topic content arrives in sibling tickets within this epic.

### Acceptance criteria

- [ ] `apm help` (no topic) exits 0 and prints a short description of the help system
- [ ] `apm help` (no topic) lists all available topics (`commands`, `config`, `workflow`, `ticket`) with a one-line summary each
- [ ] `apm help commands` exits 0 and prints a non-empty placeholder string referencing ticket 3665e017
- [ ] `apm help config` exits 0 and prints a non-empty placeholder string referencing ticket d486d183
- [ ] `apm help workflow` exits 0 and prints a non-empty placeholder string referencing ticket 7ba021e8
- [ ] `apm help ticket` exits 0 and prints a non-empty placeholder string referencing ticket 14214305
- [ ] `apm help <unknown-topic>` exits non-zero
- [ ] `apm help <unknown-topic>` prints an error message that names the unknown topic and lists the valid topics
- [ ] `apm --help` lists `help` as a subcommand in the clap-generated usage output

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
| 2026-04-28T19:27Z | — | new | philippepascal |
| 2026-04-28T19:32Z | new | groomed | philippepascal |
| 2026-04-28T19:33Z | groomed | in_design | philippepascal |