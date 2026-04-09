+++
id = "ab531177"
title = "add an apm command to clean epics"
state = "ammend"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ab531177-add-an-apm-command-to-clean-epics"
created_at = "2026-04-09T05:07:02.660761Z"
updated_at = "2026-04-09T05:27:25.210082Z"
+++

## Spec

### Problem

Epics accumulate over time as a project progresses. Once all tickets in an epic reach a terminal state (`derive_epic_state` returns `"done"`), the epic branch and its `.apm/epics.toml` entry serve no further purpose but remain in the repository indefinitely. There is currently no way to remove them short of manual `git branch -d` and hand-editing `.apm/epics.toml`.

This ticket adds `apm epic clean` — a subcommand that identifies all "done" epics, presents the list to the user, and deletes them (local branch + metadata entry) after confirmation. A `--yes` flag allows non-interactive use, and `--dry-run` lets users preview what would be removed without side effects.

### Acceptance criteria

- [ ] `apm epic clean` with no flags prints the list of "done" epics and prompts "Delete N epic(s)? [y/N]"; entering "y" deletes them
- [ ] `apm epic clean --yes` deletes all "done" epics without prompting
- [ ] `apm epic clean --dry-run` prints what would be deleted and exits without making any changes
- [ ] Epics whose derived state is not `"done"` are not listed and not deleted
- [ ] When no "done" epics exist, the command prints "Nothing to clean." and exits 0
- [ ] After deletion, the epic branch no longer exists locally
- [ ] After deletion, the epic's entry is removed from `.apm/epics.toml` (or the file is left unchanged if the epic had no entry there)
- [ ] Running in a non-interactive terminal without `--yes` skips deletion and prints a message advising the user to use `--yes`
- [ ] Entering anything other than "y" at the prompt leaves all epics untouched

### Out of scope

- Remote branch deletion (can be a follow-on with a --remote flag)\n- Cleaning epics in "empty" or "implemented" state\n- Deleting ticket branches or worktrees belonging to the epic (covered by apm clean)\n- Archiving epic ticket files (covered by apm archive)

### Approach

**Files that change**

- `apm/src/main.rs` — add `Clean` variant to `EpicCommand` with `--yes` and `--dry-run` flags; add dispatch arm in the `Command::Epic` match block
- `apm/src/cmd/epic.rs` — add `run_clean(root, dry_run, yes)` function

**`run_clean` logic**

1. Load config via `CmdContext::load_config_only`.
2. Enumerate all epic branches with `apm_core::git::epic_branches(root)`.
3. Load all tickets with `apm_core::ticket::load_all_from_git`.
4. For each epic branch, parse its 8-char ID, collect its tickets, build `state_configs`, call `apm_core::epic::derive_epic_state`. Keep only branches whose derived state is `"done"`.
5. If the candidate list is empty, print "Nothing to clean." and return.
6. Print the candidate list: `"Would delete N epic(s):"` followed by one `"  <id>  <title>"` line per candidate.
7. If `--dry-run`: print "Dry run — no changes made." and return.
8. Confirmation gate:
   - `--yes` → proceed immediately.
   - Interactive stdout → print `"Delete N epic(s)? [y/N] "`, read a line; if the trimmed input is not `"y"` (case-insensitive), print "Aborted." and return.
   - Non-interactive stdout, no `--yes` → print "Skipping — non-interactive terminal. Use --yes to confirm." and return.
9. For each candidate, run `git branch -d <branch>` and print `"deleted <branch>"`. Surface errors if git refuses (e.g. branch not merged).
10. Remove each deleted epic's ID from `.apm/epics.toml` using `toml_edit`: read the file if it exists, remove the matching top-level table key, write back. If the file is absent or the ID has no entry, skip silently.

**`EpicCommand::Clean` variant**

```rust
/// Remove local branches for "done" epics
Clean {
    /// Preview what would be deleted without making changes
    #[arg(long)]
    dry_run: bool,
    /// Skip confirmation prompt
    #[arg(long)]
    yes: bool,
},
```

Dispatch arm in the `Command::Epic` match:

```rust
Command::Epic { command: EpicCommand::Clean { dry_run, yes } } =>
    cmd::epic::run_clean(&root, dry_run, yes),
```

**Constraints**

- No new crate dependencies; reuse `toml_edit` (already used in `run_set`) for `.apm/epics.toml` writes.
- Remote branch deletion is out of scope; use local `git branch -d` only.
- Use `std::io::IsTerminal` (already imported in `cmd/clean.rs`) for the terminal check.

### Files that change

- `apm/src/main.rs` — add `Clean` variant to `EpicCommand` with `--yes` and `--dry-run` flags; add dispatch arm in the `Command::Epic` match block
- `apm/src/cmd/epic.rs` — add `run_clean(root, dry_run, yes)` function

### `run_clean` logic

1. Load config (`CmdContext::load_config_only`).
2. Enumerate all epic branches with `apm_core::git::epic_branches(root)`.
3. Load all tickets with `apm_core::ticket::load_all_from_git`.
4. For each epic branch, derive its state (`apm_core::epic::derive_epic_state`) using the same logic as `run_list`. Keep only those whose derived state is `"done"`.
5. If the candidate list is empty, print "Nothing to clean." and return.
6. Print the list:
   ```
   Would delete N epic(s):
     <id>  <title>
     ...
   ```
7. **dry-run path** (`--dry-run`): stop here, print "Dry run — no changes made." and return.
8. **Confirmation**:
   - If `--yes`: proceed.
   - Else if `stdout.is_terminal()`: print `"Delete N epic(s)? [y/N] "`, read a line; proceed only if the trimmed input is `"y"` (case-insensitive). Otherwise print "Aborted." and return.
   - Else (non-interactive, no `--yes`): print "Skipping — non-interactive terminal. Use --yes to confirm." and return.
9. For each candidate:
   a. Delete the local branch with `git branch -d <branch>` (use `-D` only if the branch is not merged — but since state is `"done"`, the PR should be merged; use `-d` and surface the error if git refuses).
   b. Print `"deleted epic/<id>-<slug>"`.
10. Remove each deleted epic's ID from `.apm/epics.toml`: read the file, drop the matching TOML table, write back. If the file doesn't exist or the ID has no entry, skip silently.

### EpicCommand variant (main.rs)

```rust
/// Remove local branches for "done" epics
Clean {
    /// Preview what would be deleted without making changes
    #[arg(long)]
    dry_run: bool,
    /// Skip confirmation prompt
    #[arg(long)]
    yes: bool,
},
```

Dispatch:
```rust
Command::Epic { command: EpicCommand::Clean { dry_run, yes } } =>
    cmd::epic::run_clean(&root, dry_run, yes),
```

### Constraints

- No new dependencies; use the same `toml_edit` already used in `run_set` for `.apm/epics.toml` mutations.
- Do not delete remote branches — keep scope minimal. Remote cleanup can be a follow-on.
- The function must compile on both interactive and non-interactive stdout; use `std::io::IsTerminal` (already imported in `clean.rs`).

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-09T05:07Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:18Z | groomed | in_design | philippepascal |
| 2026-04-09T05:22Z | in_design | specd | claude-0409-0518-22b8 |
| 2026-04-09T05:27Z | specd | ammend | apm |
