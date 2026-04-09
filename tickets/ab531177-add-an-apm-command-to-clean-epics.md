+++
id = "ab531177"
title = "add an apm command to clean epics"
state = "specd"
priority = 0
effort = 2
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/ab531177-add-an-apm-command-to-clean-epics"
created_at = "2026-04-09T05:07:02.660761Z"
updated_at = "2026-04-09T05:41:33.472129Z"
+++

## Spec

### Problem

Epics accumulate over time as a project progresses. Once all tickets in an epic reach a terminal state (`derive_epic_state` returns `"done"`), the epic branch and its `.apm/epics.toml` entry serve no further purpose but remain in the repository indefinitely. There is currently no way to remove them short of manual `git branch -d` and hand-editing `.apm/epics.toml`.

This ticket extends the existing `apm clean` command with an `--epics` flag. When passed, `apm clean --epics` identifies all "done" epics, presents the list, and deletes them (local branch + `.apm/epics.toml` entry) after confirmation. The existing `--yes` and `--dry-run` flags on `apm clean` apply to the epic cleanup as well.

### Acceptance criteria

- [ ] `apm clean --epics` with no other flags prints the list of "done" epics and prompts "Delete N epic(s)? [y/N]"; entering "y" deletes them
- [ ] `apm clean --epics --yes` deletes all "done" epics without prompting
- [ ] `apm clean --epics --dry-run` prints what would be deleted and exits without making any changes
- [ ] Epics whose derived state is not `"done"` are not listed and not deleted
- [ ] When no "done" epics exist, the command prints "Nothing to clean." and exits 0
- [ ] After deletion, the epic branch no longer exists locally
- [ ] After deletion, the epic's entry is removed from `.apm/epics.toml` (or the file is left unchanged if the epic had no entry there)
- [ ] Running `apm clean --epics` in a non-interactive terminal without `--yes` skips epic deletion and prints a message advising the user to use `--yes`
- [ ] Entering anything other than "y" at the prompt leaves all epics untouched
- [ ] `apm clean --epics` can be combined with other `apm clean` flags (e.g. `--branches`, `--dry-run`); epic cleanup runs after ticket cleanup in the same invocation

### Out of scope

- Remote epic branch deletion (can be a follow-on with a `--remote` flag or by extending the existing `--remote` behaviour)
- Cleaning epics in "empty" or "implemented" state
- Deleting ticket branches or worktrees belonging to the epic (covered by the existing `apm clean` logic)
- Archiving epic ticket files (covered by `apm archive`)
- A standalone `apm epic clean` subcommand

### Approach

**Files that change**

- `apm/src/main.rs` — add `--epics` flag (`epics: bool`) to the `Clean` variant; pass it through to `cmd::clean::run()`
- `apm/src/cmd/clean.rs` — add `epics: bool` parameter to `run()`; at the end of the function, when `epics` is `true`, execute the epic-cleanup block below

**`--epics` flag in `main.rs`**

Add to the `Clean` variant:

```rust
/// Also clean local branches for "done" epics
#[arg(long)]
epics: bool,
```

Add `epics` to the existing dispatch arm:

```rust
Command::Clean { dry_run, yes, force, branches, remote, older_than, untracked, epics } =>
    cmd::clean::run(&root, dry_run, yes, force, branches, remote, older_than, untracked, epics),
```

**Epic-cleanup block in `cmd/clean.rs`**

At the end of `run()`, after all existing ticket-cleanup logic, insert:

```rust
if epics {
    run_epic_clean(root, &config, dry_run, yes)?;
}
```

Implement `run_epic_clean(root, config, dry_run, yes)` as a private function in the same file:

1. Call `apm_core::git::epic_branches(root)` — returns all locally-visible epic branch names (already deduplicates local+remote).
2. Filter to local-only branches: skip any name that was derived from `origin/` in `epic_branches`. Because `epic_branches` already strips the `origin/` prefix during dedup, determine local existence by running `git branch --list <branch>` for each candidate (or reuse the local subset: the function iterates local before remote, so the first dedup pass covers local branches; a simpler filter is `git branch --list 'epic/*'` directly).
   - **Simpler approach**: call `git branch --list 'epic/*'` directly from `run_epic_clean` to get only local branches.
3. Load all tickets with `apm_core::ticket::load_all_from_git(root)`.
4. Load `config.states` to build `state_configs`.
5. For each local epic branch:
   a. Parse the 8-char epic ID from the branch name (`epic/<id>-<slug>`).
   b. Collect tickets whose `epic` field matches this ID.
   c. Build `&[&StateConfig]` by mapping each ticket's state through `config.states`.
   d. Call `apm_core::epic::derive_epic_state(&state_configs)`.
   e. Keep only branches where the result is `"done"`.
6. If no candidates: print `"No done epics to clean."` and return.
7. Print:
   ```
   Would delete N epic(s):
     <id>  <title-or-branch-name>
   ```
8. If `dry_run`: print `"Dry run — no changes made."` and return.
9. Confirmation gate (mirrors the existing pattern in `run()`):
   - `yes` → proceed.
   - `std::io::stdout().is_terminal()` → print `"Delete N epic(s)? [y/N] "`, flush, read a line; proceed only if the trimmed input equals `"y"` (case-insensitive). Otherwise print `"Aborted."` and return.
   - Non-interactive, no `--yes` → print `"Skipping — non-interactive terminal. Use --yes to confirm."` and return.
10. For each candidate:
    a. Run `git branch -d <branch>`. If git refuses (branch not fully merged), surface the error and continue to the next candidate.
    b. Print `"deleted <branch>"`.
    c. Remove the epic's ID key from `.apm/epics.toml` using `toml_edit`: read the file if it exists, drop the top-level table keyed by the ID, write back. If the file is absent or the key is missing, skip silently.

**Title lookup for the candidate list**

Epic title is stored in `.apm/epics.toml` under the epic's ID key (field `title`). Load the TOML file once at the start of `run_epic_clean`; fall back to the branch name when the entry is absent.

**Constraints**

- No new crate dependencies. `toml_edit` is already used in `run_set`; `std::io::IsTerminal` is already imported in `clean.rs`.
- Only local epic branches are targeted (`git branch --list 'epic/*'`). Remote cleanup is out of scope.
- The function signature of `run()` gains one parameter (`epics: bool`) — update the call site in `main.rs` accordingly.

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

- [x] include this in the apm clean command instead of the apm epic
- [ ] there is no point cleaning the epic branches only locally
- [ ] apm clean --epics only clean epics, and does it to local and remote
- [ ] apm clean --remote, does the same clean as now and in addition (in a subsequent step) cleans the epics just like apm clean --epics would do

### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-09T05:07Z | — | new | philippepascal |
| 2026-04-09T05:17Z | new | groomed | apm |
| 2026-04-09T05:18Z | groomed | in_design | philippepascal |
| 2026-04-09T05:22Z | in_design | specd | claude-0409-0518-22b8 |
| 2026-04-09T05:27Z | specd | ammend | apm |
| 2026-04-09T05:37Z | ammend | in_design | philippepascal |
| 2026-04-09T05:41Z | in_design | specd | claude-0409-0537-f100 |