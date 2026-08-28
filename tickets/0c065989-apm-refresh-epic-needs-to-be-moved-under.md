+++
id = "0c065989"
title = "apm refresh-epic needs to be moved under apm epic refresh for consistency"
state = "in_progress"
priority = 3
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/0c065989-apm-refresh-epic-needs-to-be-moved-under"
created_at = "2026-08-28T00:50:30.208076Z"
updated_at = "2026-08-28T17:44:20.371380Z"
+++

## Spec

### Problem

`apm refresh-epic <id>` is a top-level command, even though every other
epic-scoped operation (`new`, `submit`, `close`, `list`, `show`, `set`) lives
under `apm epic <subcommand>`. This is the only epic operation that breaks the
`apm epic ...` convention, which makes the CLI surface harder to discover
(`apm epic --help` doesn't mention it) and inconsistent with the rest of the
command tree documented in `apm main.rs`'s help template and `apm help
commands`.

The command should be relocated to `apm epic refresh <id>` so all epic
operations are grouped under one subcommand namespace, with identical
behaviour and flags. No backward-compatible alias is kept for the old
top-level name, per the project's no-shim convention.

### Acceptance criteria

- [x] `apm epic refresh <id>` (no flags) prints the ahead-count / clean-vs-conflicted status and, when stdout is a terminal, prompts for merge/PR/auto/skip — identical to the current `apm refresh-epic <id>` no-flag behaviour
- [x] `apm epic refresh <id> --merge` performs a local merge of the default branch into the epic branch, identical to current `apm refresh-epic --merge`
- [ ] `apm epic refresh <id> --pr` opens or updates a PR from the default branch into the epic branch, identical to current `apm refresh-epic --pr`
- [ ] `apm epic refresh <id> --auto` merges locally when clean and falls back to a PR on conflict, identical to current `apm refresh-epic --auto`
- [ ] `apm epic refresh <id> --merge --push` and `--merge --no-push` control the post-merge push exactly as `apm refresh-epic` did
- [ ] `apm refresh-epic <id>` no longer exists — running it fails with clap's unknown-subcommand error
- [ ] `apm epic --help` lists `refresh` alongside `new`, `submit`, `close`, `list`, `show`, `set`
- [ ] `apm help commands` no longer lists `refresh-epic` as a top-level command, and lists `refresh` nested under `epic`
- [ ] `cargo test --workspace` passes, including integration tests exercising the relocated command under `apm epic refresh`

### Out of scope

- No change to the refresh logic itself: status printing, quiescence check, interactive prompt, local merge, or PR creation/update behaviour — this is a pure command relocation
- No backward-compatible alias or deprecation warning for the old `apm refresh-epic` top-level invocation
- No changes to other epic subcommands (`new`, `submit`, `close`, `list`, `show`, `set`)
- No changes to the quiescence rules or `epic_is_quiescent` logic (covered by other tickets, e.g. 27439a80)

### Approach

This is a mechanical relocation: move the `RefreshEpic` top-level command
into `EpicCommand` as `Refresh`, with no change to the underlying logic.

#### `apm/src/main.rs`

- Delete the top-level `Command::RefreshEpic { .. }` variant (currently
  around line 826, right after the `Epic { .. }` variant) — including its
  doc comment and all six fields (`id`, `merge`, `pr`, `auto_mode`, `push`,
  `no_push`) with their existing `#[arg(...)]` attributes and
  `conflicts_with_all` groups.
- Add a `Refresh` variant to `enum EpicCommand` (after `Set`), carrying the
  exact same fields/attrs/doc comment that `RefreshEpic` had. Keep the doc
  comment `/// Pull default-branch updates into an epic branch` so it shows
  up correctly in `apm epic --help` and `apm help commands`.
- Update the dispatch match: replace the `Command::RefreshEpic { id, merge,
  pr, auto_mode, push, no_push } => cmd::epic::run_refresh_epic(...)` arm
  with `Command::Epic { command: EpicCommand::Refresh { id, merge, pr,
  auto_mode, push, no_push } } => cmd::epic::run_refresh_epic(&root, &id,
  merge, pr, auto_mode, push, no_push)`, placed alongside the other
  `Command::Epic { command: EpicCommand::... }` arms (~line 1318 onward).
- Remove the `refresh-epic   Pull default-branch updates into an epic
  branch` line from the `Epics:` section of the top-level `help_template`
  (line 43), leaving just `epic  Manage epics`.

#### `apm/src/cmd/epic.rs`

- No behavioural change. `run_refresh_epic` keeps its current signature
  `(root, id_arg, merge, pr, auto_mode, push, no_push) -> Result<()>` — only
  the caller in `main.rs` changes. Renaming the function is optional; if
  renamed (e.g. to `run_refresh`, matching the `run_new`/`run_submit`/
  `run_close`/`run_list`/`run_show`/`run_set` sibling naming), update the
  single call site in `main.rs` to match.

#### `apm/tests/integration.rs`

- Update every test that invokes the command via `.args(["refresh-epic",
  ...])` to `.args(["epic", "refresh", ...])`. Current call sites: the
  `--push`/`--no-push` tests (`refresh_epic_merge_push_flag_pushes_to_origin`,
  `refresh_epic_merge_no_push_flag_skips_push`,
  `refresh_epic_merge_noninteractive_skips_push`) and the no-flag status test
  (`refresh_epic_no_flag_noninteractive_prints_status_and_exits`). Test and
  helper function names (e.g. `setup_refresh_epic_for_push`) can stay as-is —
  only the `args` arrays and any comments referencing the old invocation
  syntax need to change.
- In `help_commands_includes_visible_top_level_commands`, remove
  `"refresh-epic"` from the list of expected top-level command names (it is
  no longer a top-level command; `"epic"` already covers discoverability of
  its subcommands).

#### `docs/strategy-and-dependencies.md`

- Replace the three prose references to `apm refresh-epic <id>` / `apm
  refresh-epic` (in the "Refresh and close: epic must be quiescent" section
  and item 6 of "Implementation rules") with `apm epic refresh <id>` / `apm
  epic refresh`. No other wording changes needed.

#### Verification

- `cargo test --workspace` must pass, in particular the relocated
  integration tests and `help_commands_includes_visible_top_level_commands`.
- Manually run `apm epic --help` and `apm refresh-epic` (expect clap's
  unknown-subcommand error) to sanity-check the CLI surface.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-08-28T00:50Z | — | new | philippepascal |
| 2026-08-28T07:13Z | new | groomed | philippepascal |
| 2026-08-28T07:24Z | groomed | in_design | philippepascal |
| 2026-08-28T07:26Z | in_design | specd | claude |
| 2026-08-28T17:22Z | specd | ready | philippepascal |
| 2026-08-28T17:44Z | ready | in_progress | philippepascal |