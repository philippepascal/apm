+++
id = 5
title = "Add apm verify (integrity checks)"
state = "closed"
priority = 2
effort = 4
risk = 2
updated_at = "2026-03-27T00:06:00.362807Z"
+++

## Spec

### Amendment requests
- [x] verify should also look at formatting of ticket documents, there might be other areas where a manual change cause inconsistencies.

  Addressed: added document format checks to acceptance criteria and approach below.
  Checks cover: required sections present, state value is known, frontmatter id matches filename.

### Problem

Ticket files can drift into inconsistent states: a branch merged but ticket still
`implemented`; agent field set but no branch; ticket `in_progress` with no branch.
`apm verify` provides a way to detect these inconsistencies explicitly, both as a
manual audit tool and as a pre-commit hook.

### Acceptance criteria

- [ ] `apm verify` loads all non-closed tickets and checks each for known inconsistencies
- [ ] Detects: ticket in `in_progress`/`implemented`/`accepted` with no `branch` field
- [ ] Detects: ticket in `in_progress` or `implemented` with `branch` field pointing to a branch merged into main (should have been auto-transitioned by sync)
- [ ] Detects: ticket with `agent` set but state not in `in_progress`, `implemented`, or `accepted`
- [ ] Detects: ticket file missing `## Spec` section
- [ ] Detects: ticket file missing `## History` section
- [ ] Detects: state value not in the configured `[[workflow.states]]` list
- [ ] Detects: frontmatter `id` does not match the numeric prefix in the filename
- [ ] Each issue is printed as: `#<id> [<state>]: <description of issue>`
- [ ] Exit code 0 if no issues found, 1 if any issues found
- [ ] `apm verify --fix` automatically transitions merged-but-not-accepted tickets (same logic as `apm sync`)
- [ ] `apm verify --fix` does not auto-fix issues requiring human judgment (missing branch, unexpected agent)

### Out of scope

- SQLite cache consistency checks
- Full TOML parse validation (parse failure is already surfaced as a warning in `load_all`)
- Cross-ticket consistency (e.g. duplicate IDs)

### Approach

New subcommand `apm verify` in `apm/src/cmd/verify.rs`. Walk all tickets, run each
check as a closure returning `Option<String>` (the issue description). Collect all
issues, print, exit with appropriate code.

`--fix` mode: for tickets with merged-but-not-accepted issue, apply the same
transition as `cmd/sync.rs` — update state + history, call `git::commit_to_branch`
to `main`.

## History

| Date | Actor | Transition | Note |
|------|-------|------------|------|
| 2026-03-25 | manual | new → specd | |
| 2026-03-25 | manual | specd → ammend | |
| 2026-03-25 | manual | ammend → specd | |
| 2026-03-25 | manual | specd → ready | |
| 2026-03-25 | manual | ready → ready | |
| 2026-03-26 | manual | ready → ready | Respec: --fix uses commit_to_branch pattern |
| 2026-03-26 | manual | ready → specd | |
| 2026-03-26 | manual | specd → ready | |
| 2026-03-27T00:06Z | ready | closed | apm |