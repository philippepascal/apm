+++
id = "67f83715"
title = "apm list should have a way to filter per epic"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/67f83715-apm-list-should-have-a-way-to-filter-per"
created_at = "2026-06-11T01:09:44.527139Z"
updated_at = "2026-06-11T01:13:49.796268Z"
+++

## Spec

### Problem

`apm list` has no way to scope the output to a single epic. On a project with several epics in flight, running `apm list` shows every ticket regardless of which epic it belongs to. Users working on one epic must mentally filter the noise or grep through the output.

The `epic` field is already stored on each ticket's frontmatter and the `list_filtered` function in `apm-core` is the natural place to add the predicate. The `apm start --epic` and `apm work --epic` flags follow the same pattern; `apm list` is conspicuously missing it.

### Acceptance criteria

- [ ] `apm list --epic <ID>` outputs only tickets whose `epic` frontmatter field starts with `<ID>`
- [ ] `apm list --epic <ID>` with no matching tickets produces no rows and exits 0
- [ ] `apm list --epic <ID>` composes with `--state`: only tickets matching both filters are shown
- [ ] `apm list` without `--epic` behaves identically to before (no regression)
- [ ] `apm list --help` lists `--epic` with a short description and `<ID>` as the value name

### Out of scope

- Filtering `apm next` by epic (separate command with its own flag)
- Filtering `apm list` by epic slug or epic title (ID only)
- A `--no-epic` flag to list tickets that belong to no epic

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-11T01:09Z | — | new | philippepascal |
| 2026-06-11T01:12Z | new | groomed | philippepascal |
| 2026-06-11T01:13Z | groomed | in_design | philippepascal |