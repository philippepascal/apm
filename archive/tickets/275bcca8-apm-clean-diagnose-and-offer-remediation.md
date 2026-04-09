+++
id = "275bcca8"
title = "apm clean: diagnose and offer remediation for dirty worktrees"
state = "closed"
priority = 0
effort = 5
risk = 3
author = "philippepascal"
agent = "33221"
branch = "ticket/275bcca8-apm-clean-diagnose-and-offer-remediation"
created_at = "2026-03-30T18:12:35.205840Z"
updated_at = "2026-03-30T19:54:45.114218Z"
+++

## Spec

### Problem

When `apm clean` encounters a worktree with uncommitted changes, it silently skips it with a one-line warning. The user gets no actionable information: not what the files are, not whether they matter, not what to do.

In practice, dirty worktrees fall into a few distinct categories that each warrant a different response:

- **Untracked temp files** (`pr-body.md`, `.apm-worker.pid`, `ac.txt`, etc.) — leftover worker artifacts that are safe to delete. The worktree is effectively clean for the purposes of branch removal.
- **Stale PID files / log files** (`.apm-worker.pid`, `.apm-worker.log`) — the process is gone; the file is noise.
- **Untracked user files** — possibly intentional; user should decide.
- **Modified tracked files** — real uncommitted work; definitely needs user attention before cleaning.

The current behaviour conflates all of these into "has uncommitted changes — skipping", which:
1. Doesn't distinguish safe-to-clean from risky cases
2. Gives no remediation path
3. Forces the user to manually inspect and clean each worktree before `apm clean` will proceed

The desired behaviour: `apm clean` diagnoses each blocked worktree, explains what it found (categorised), proposes a concrete action (e.g. "remove 3 untracked temp files"), and asks the user to confirm or skip. Modified tracked files are always left for the user to handle manually.

### Acceptance criteria

- [x] When `apm clean` finds a dirty worktree whose only dirty files are known temp files (`pr-body.md`, `body.md`, `ac.txt`, `.apm-worker.pid`, `.apm-worker.log`), it lists those files and prompts `Remove N file(s) and clean? [y/N]`
- [x] Confirming the prompt deletes the listed temp files, then removes the worktree and branch as normal
- [x] Declining the prompt skips that worktree with a one-line "skipping" message
- [x] When a dirty worktree contains modified tracked files, `apm clean` lists each file with an `M` prefix, prints "manual cleanup required — skipping", and does not prompt
- [x] When a dirty worktree has both modified tracked files and untracked files, the modified-tracked gate applies: no prompt, skip with message
- [x] When a dirty worktree contains untracked files not in the known-temp list, `apm clean` lists them labelled `[user]`, distinct from known-temp files labelled `[temp]`, and includes all of them in the removal prompt
- [x] In `--dry-run` mode, `apm clean` prints a categorised diagnosis of each dirty worktree (file labels and names) without prompting or removing anything
- [x] A `--yes` flag auto-confirms all removal prompts without reading stdin, enabling scripted use

### Out of scope

- Removing modified tracked file content — always left for the user to handle manually
- Making the known-temp filename list configurable via `apm.toml` — it is a hardcoded constant
- Selective per-file confirmation — removal is all-or-nothing per worktree
- Staged (indexed) changes — treated as modified tracked files and block auto-cleaning

### Approach

**Core changes (`apm-core/src/clean.rs`):**

1. Add a `DirtyWorktree` struct capturing the diagnosis for one blocked worktree:
   ```rust
   pub struct DirtyWorktree {
       pub ticket_id: String,
       pub ticket_title: String,
       pub branch: String,
       pub path: PathBuf,
       pub local_branch_exists: bool,
       pub known_temp: Vec<PathBuf>,
       pub other_untracked: Vec<PathBuf>,
       pub modified_tracked: Vec<PathBuf>,
   }
   ```

2. Add a `const KNOWN_TEMP_FILES: &[&str]` with the safe filenames:
   `"pr-body.md"`, `"body.md"`, `"ac.txt"`, `".apm-worker.pid"`, `".apm-worker.log"`.

3. Add `diagnose_worktree(path, ticket_id, ticket_title, branch, local_branch_exists) -> Result<DirtyWorktree>`:
   - Runs `git status --porcelain` in the worktree
   - Parses each output line: `XY <file>`
     - `??` prefix → untracked: filename in `KNOWN_TEMP_FILES` → `known_temp`, else → `other_untracked`
     - Any other XY → `modified_tracked`

4. Change `candidates()` return type from `Result<Vec<CleanCandidate>>` to `Result<(Vec<CleanCandidate>, Vec<DirtyWorktree>)>`.
   Replace the current dirty-worktree `eprintln!(...); continue;` block with a call to `diagnose_worktree()` that pushes into the second vec.

5. Add `remove_untracked(wt_path: &Path, files: &[PathBuf]) -> Result<()>` that deletes each listed file.

**Command layer (`apm/src/cmd/clean.rs`):**

6. Update `run(root, dry_run, yes)` to receive both vecs from `candidates()`.

7. For each `DirtyWorktree`:
   - If `modified_tracked` non-empty:
     - Print each file as `  M <file>` then `warning: <branch> has modified tracked files — manual cleanup required — skipping`
   - Else (only untracked files):
     - Print each `known_temp` file as `  [temp] <file>`
     - Print each `other_untracked` file as `  [user] <file>`
     - In `--dry-run`: print `would remove N file(s) — re-run without --dry-run to be prompted`
     - Not dry-run, `yes=true`: call `remove_untracked` then construct a `CleanCandidate` from the `DirtyWorktree` fields and call `remove()`
     - Not dry-run, `yes=false`, interactive (stdout is a terminal): prompt `Remove N file(s) and clean? [y/N]`; on "y": remove and clean; on anything else: print `skipping <branch>`
     - Not dry-run, non-interactive: print `skipping <branch> — untracked files present (use --yes to auto-remove)`

**CLI (`apm/src/main.rs`):**

8. Add `--yes` / `-y` boolean flag to the `Clean` subcommand. Pass it through to `cmd::clean::run`.

**Tests (`apm/tests/integration.rs`):**

9. `clean_yes_removes_known_temp_files_and_cleans`: create a merged/closed worktree, drop a `pr-body.md` into it, run `apm clean --yes`, assert worktree and branch are gone.
10. `clean_skips_modified_tracked_files`: create a merged/closed worktree, modify a tracked file without committing, run `apm clean --yes`, assert worktree and branch still exist and stdout contains "manual cleanup required".
11. `clean_dry_run_diagnoses_dirty_worktree`: create a merged/closed worktree with a `pr-body.md`, run `apm clean --dry-run`, assert output mentions `[temp]` and no files removed.
12. `clean_yes_removes_other_untracked_files`: create a merged/closed worktree with an unrecognised untracked file (e.g. `notes.txt`), run `apm clean --yes`, assert worktree and branch are gone.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T18:12Z | — | new | philippepascal |
| 2026-03-30T19:16Z | new | in_design | philippepascal |
| 2026-03-30T19:23Z | in_design | specd | claude-0330-1920-b7f2 |
| 2026-03-30T19:24Z | specd | ready | apm |
| 2026-03-30T19:26Z | ready | in_progress | philippepascal |
| 2026-03-30T19:33Z | in_progress | implemented | claude-0330-1930-w8x2 |
| 2026-03-30T19:47Z | implemented | accepted | apm-sync |
| 2026-03-30T19:54Z | accepted | closed | apm-sync |