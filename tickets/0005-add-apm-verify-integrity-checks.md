+++
id = 5
title = "Add apm verify (integrity checks)"
state = "specd"
priority = 2
effort = 4
risk = 2
created = "2026-03-25"
updated = "2026-03-25"
+++

## Spec

### Problem

Ticket files can drift into inconsistent states: a branch merged but ticket still
`implemented`; agent field set but no branch; ticket `in_progress` with no branch.
`apm verify` provides a way to detect these inconsistencies explicitly, both as a
manual audit tool and as a pre-commit hook.

### Acceptance criteria

- [ ] `apm verify` loads all non-closed tickets and checks each for known inconsistencies
- [ ] Detects: ticket in `in_progress`/`implemented`/`accepted` with no `branch` field
- [ ] Detects: ticket in `in_progress` or `implemented` with `branch` field pointing to a branch merged into main (should have been auto-transitioned by sync)
- [ ] Detects: ticket with `agent` set but state not in Layer 2 (`in_progress`, `implemented`, `accepted`)
- [ ] Each issue is printed as: `#<id> [<state>]: <description of issue>`
- [ ] Exit code 0 if no issues found, 1 if any issues found
- [ ] `apm verify --fix` automatically applies safe fixes (runs `apm sync` logic for merged-but-not-accepted tickets)
- [ ] `apm verify --fix` does not auto-fix issues that require human judgment (missing branch field, unexpected agent assignment)

### Out of scope

- SQLite cache consistency checks
- Checking ticket file format / TOML validity (that's `apm parse`)
- Cross-ticket consistency (e.g. duplicate IDs)

### Approach

New subcommand `apm verify` in `apm/src/cmd/verify.rs`. Walk all tickets, run each
check as a closure returning `Option<String>` (the issue description). Collect all
issues, print, exit with appropriate code. `--fix` mode re-uses sync logic from
`cmd/sync.rs` for the mergeable case.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
