+++
id = "4fb7ae94"
title = "apm list includes an epic column"
state = "in_progress"
priority = 0
effort = 2
risk = 2
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4fb7ae94-apm-list-includes-an-epic-column"
created_at = "2026-04-28T00:25:28.853946Z"
updated_at = "2026-04-28T06:01:18.947978Z"
+++

## Spec

### Problem

`apm list` currently renders four columns: ID, state, owner, and title. There is no visibility into a ticket's epic membership or base-branch context. To understand where a ticket fits in the git topology, a user must `apm show` each ticket individually.

Every ticket has an optional `target_branch` field. For epic-member tickets this holds the epic branch (e.g. `epic/8db73240-user-auth`); for standalone tickets the field is absent. When absent, the ticket's implicit base is the project's configured default branch (typically `main`).

Adding an epic/base-branch column to `apm list` exposes this topology at a glance without requiring any per-ticket drill-down.

### Acceptance criteria

- [x] `apm list` output includes a new column between the owner column and the title column
- [x] For a ticket whose `target_branch` frontmatter field is set, the column displays only the epic ID extracted from that value (e.g. `8db73240` from `epic/8db73240-user-auth`)
- [x] For a ticket whose `target_branch` field is absent, the column displays the project's configured default branch (e.g. `main`)
- [x] All rows in a single `apm list` invocation use the same fixed column width so values are left-aligned in a consistent gutter
- [x] Existing snapshot or integration tests for `apm list` pass (updated to include the new column)

### Out of scope

- Filtering `apm list` by epic or by target branch
- Resolving the epic ID to a human-readable epic title in the column
- Showing the ticket's own branch name (distinct from `target_branch`)
- Any changes to `apm show`, `apm epic list`, or other commands

### Approach

**File to change:** `apm/src/cmd/list.rs`

1. **Confirm config availability.** The `run()` function (or its caller) already receives a `Config` value for most commands. Verify that the project config — specifically `config.project.default_branch` — is accessible in the list command entry point. If not, thread it through from the CLI dispatch layer the same way other commands receive it.

2. **Compute the column value per ticket.** In the per-ticket formatting loop, derive the display string:
   ```rust
   let base = match ticket.frontmatter.target_branch.as_deref() {
       Some(branch) => apm_core::epic::epic_id_from_branch(branch).to_owned(),
       None => config.project.default_branch.clone(),
   };
   ```
   `epic_id_from_branch` is already exported from `apm-core/src/epic.rs`. It strips the `epic/` prefix and returns everything before the first `-`, so `epic/8db73240-user-auth` → `8db73240`.

3. **Insert the column into the format string.** The current format is:
   ```
   {id:<8} [{state:<12}] {owner:<16} {title}
   ```
   Change it to:
   ```
   {id:<8} [{state:<12}] {owner:<16} {base:<12} {title}
   ```
   Width `12` comfortably fits an 8-char epic ID and typical default branch names like `main` or `master`.

4. **Update tests.** Find any snapshot tests or integration tests in `apm/tests/` or `testdata/` that assert on `apm list` output and update their expected strings to include the new column. Run `cargo test -p apm` to confirm.

### Open questions


### Amendment requests

- [x] only display the epic id in the column if applicable. in your example 8db73240

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-28T00:25Z | — | new | philippepascal |
| 2026-04-28T00:26Z | new | groomed | philippepascal |
| 2026-04-28T00:57Z | groomed | in_design | philippepascal |
| 2026-04-28T01:02Z | in_design | specd | claude-0428-0057-2c68 |
| 2026-04-28T01:34Z | specd | ammend | philippepascal |
| 2026-04-28T01:38Z | ammend | in_design | philippepascal |
| 2026-04-28T01:41Z | in_design | specd | claude-0428-0138-9198 |
| 2026-04-28T06:00Z | specd | ready | philippepascal |
| 2026-04-28T06:01Z | ready | in_progress | philippepascal |
