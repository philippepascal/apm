+++
id = "67f83715"
title = "apm list should have a way to filter per epic"
state = "in_progress"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/67f83715-apm-list-should-have-a-way-to-filter-per"
created_at = "2026-06-11T01:09:44.527139Z"
updated_at = "2026-06-11T05:29:44.743999Z"
+++

## Spec

### Problem

`apm list` has no way to scope the output to a single epic. On a project with several epics in flight, running `apm list` shows every ticket regardless of which epic it belongs to. Users working on one epic must mentally filter the noise or grep through the output.

The `epic` field is already stored on each ticket's frontmatter and the `list_filtered` function in `apm-core` is the natural place to add the predicate. The `apm start --epic` and `apm work --epic` flags follow the same pattern; `apm list` is conspicuously missing it.

### Acceptance criteria

- [x] `apm list --epic <ID>` outputs only tickets whose `epic` frontmatter field starts with `<ID>`
- [x] `apm list --epic <ID>` with no matching tickets produces no rows and exits 0
- [x] `apm list --epic <ID>` composes with `--state`: only tickets matching both filters are shown
- [x] `apm list` without `--epic` behaves identically to before (no regression)
- [x] `apm list --help` lists `--epic` with a short description and `<ID>` as the value name

### Out of scope

- Filtering `apm next` by epic (separate command with its own flag)
- Filtering `apm list` by epic slug or epic title (ID only)
- A `--no-epic` flag to list tickets that belong to no epic

### Approach

#### 1. `apm-core/src/ticket/ticket_util.rs` — extend `list_filtered`

Add `epic_filter: Option<&str>` as the last parameter of `list_filtered`. Inside the existing filter closure add:

```rust
let epic_ok = epic_filter.is_none_or(|id| {
    fm.epic.as_deref().is_some_and(|e| e.starts_with(id))
});
```

Include `epic_ok` in the final `&&` chain. Update every in-file call site in the test section (add `None` as the new final argument). Add one new unit test `list_filtered_by_epic` that constructs tickets with and without an `epic` field, passes a 4-char prefix, and asserts only the matching ticket is returned.

#### 2. `apm/src/cmd/list.rs` — thread the parameter

Add `epic: Option<String>` to `run`'s signature (after `owner`). Pass `epic.as_deref()` as the new last argument to `list_filtered`.

#### 3. `apm/src/main.rs` — expose the CLI flag

In the `List` variant of `Command`:
- Add a new field:
  ```rust
  /// Show only tickets in this epic (4–8 char hex prefix)
  #[arg(long, value_name = "ID")]
  epic: Option<String>,
  ```
- Add `epic` to the destructuring pattern and to the `cmd::list::run(...)` call.
- Add an example line to the long_about string:
  ```
  apm list --epic 57bce963      # only tickets in this epic
  ```

No changes to `apm-core`'s public API surface beyond the added parameter; callers outside this ticket (none exist) would need updating, but the only callers are inside `apm/src/cmd/list.rs` and the inline tests.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-06-11T01:09Z | — | new | philippepascal |
| 2026-06-11T01:12Z | new | groomed | philippepascal |
| 2026-06-11T01:13Z | groomed | in_design | philippepascal |
| 2026-06-11T01:16Z | in_design | specd | claude |
| 2026-06-11T05:29Z | specd | ready | philippepascal |
| 2026-06-11T05:29Z | ready | in_progress | philippepascal |